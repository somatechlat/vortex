use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::process::{Command, Child, ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use async_trait::async_trait;
use serde_json::{Value, json};
use crate::error::McpError;
use crate::McpResult;

/// MCP Client interface for tool discovery and execution.
#[async_trait]
pub trait McpClient: Send + Sync {
    /// List tools provided by this MCP server.
    async fn list_tools(&self) -> McpResult<Vec<McpTool>>;

    /// Call a tool with parameters.
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Value>;
}

/// A concrete MCP client that communicates over stdio with a child process.
pub struct StdioMcpClient {
    stdin: Arc<Mutex<ChildStdin>>,
    reader: Arc<Mutex<BufReader<ChildStdout>>>,
    request_id: AtomicU64,
    _child: Arc<Mutex<Child>>, // Hold child to prevent dropping
}

impl StdioMcpClient {
    pub fn spawn(command: &str, args: &[&str]) -> McpResult<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| McpError::Connection(format!("Failed to spawn MCP server: {}", e)))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| McpError::Connection("Failed to open stdin".into()))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| McpError::Connection("Failed to open stdout".into()))?;

        Ok(Self {
            stdin: Arc::new(Mutex::new(stdin)),
            reader: Arc::new(Mutex::new(BufReader::new(stdout))),
            request_id: AtomicU64::new(1),
            _child: Arc::new(Mutex::new(child)),
        })
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn list_tools(&self) -> McpResult<Vec<McpTool>> {
        let id = self.next_id();
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "list_tools",
            "params": {}
        });

        let response = self.send_request(request).await?;

        let tools: Vec<McpTool> = serde_json::from_value(response.get("result").cloned().unwrap_or_default())
            .map_err(|e| McpError::Schema(format!("Invalid list_tools response: {}", e)))?;

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Value> {
        let id = self.next_id();
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "call_tool",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let response = self.send_request(request).await?;
        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl StdioMcpClient {
    async fn send_request(&self, request: Value) -> McpResult<Value> {
        let json = serde_json::to_string(&request)? + "\n";

        // 1. Send request
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(json.as_bytes()).await
                .map_err(|e| McpError::Connection(format!("Failed to write to stdin: {}", e)))?;
            stdin.flush().await
                .map_err(|e| McpError::Connection(format!("Failed to flush stdin: {}", e)))?;
        }

        // 2. Read response
        let mut line = String::new();
        {
            let mut reader = self.reader.lock().await;
            reader.read_line(&mut line).await
                .map_err(|e| McpError::Connection(format!("Failed to read from stdout: {}", e)))?;
        }

        if line.is_empty() {
            return Err(McpError::Connection("MCP server closed connection unexpectedly".into()));
        }

        let response: Value = serde_json::from_str(&line)?;

        if let Some(error) = response.get("error") {
            return Err(McpError::Protocol(format!("MCP Error: {}", error)));
        }

        Ok(response)
    }
}

/// MCP Tool definition (matches MCP Spec)
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}
