//! AST-based Security Scanner
//!
//! Scans Python code for dangerous patterns.

use std::path::Path;

/// Security finding severity
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// A security finding
#[derive(Debug)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub file: String,
    pub line: usize,
    pub description: String,
}

/// Security scan report
#[derive(Debug)]
pub struct SecurityReport {
    pub scanned_files: usize,
    pub elapsed_ms: u64,
    pub findings: Vec<Finding>,
    pub passed: bool,
}

/// AST Security Scanner
pub struct AstScanner {
    patterns: Vec<DangerPattern>,
}

struct DangerPattern {
    code: &'static str,
    severity: Severity,
    description: &'static str,
    // Function that checks if an AST node matches
    // TODO: Implement with rustpython-parser
}

impl AstScanner {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }
    
    fn default_patterns() -> Vec<DangerPattern> {
        vec![
            DangerPattern {
                code: "SEC-RCE",
                severity: Severity::Critical,
                description: "Remote code execution via shell command",
            },
            DangerPattern {
                code: "SEC-NET",
                severity: Severity::Critical,
                description: "Network socket access",
            },
            DangerPattern {
                code: "SEC-FILE",
                severity: Severity::High,
                description: "Suspicious file access",
            },
        ]
    }
    
    /// Scan a package directory
    pub fn scan_package(&self, path: &Path) -> Result<SecurityReport, std::io::Error> {
        let start = std::time::Instant::now();
        let mut findings = Vec::new();
        let mut files_scanned = 0;
        
        // Walk directory and scan .py files
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let file_path = entry.path();
                
                if file_path.extension().is_some_and(|ext| ext == "py") {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        files_scanned += 1;
                        let file_findings = self.scan_file(&content, &file_path.display().to_string());
                        findings.extend(file_findings);
                    }
                }
            }
        }
        
        let passed = !findings.iter().any(|f| f.severity == Severity::Critical);
        
        Ok(SecurityReport {
            scanned_files: files_scanned,
            elapsed_ms: start.elapsed().as_millis() as u64,
            findings,
            passed,
        })
    }
    
    fn scan_file(&self, content: &str, filename: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Simple pattern matching (will be replaced with AST analysis)
        for (line_num, line) in content.lines().enumerate() {
            // Check for dangerous imports/calls
            if line.contains("os.system") || line.contains("subprocess") {
                findings.push(Finding {
                    code: "SEC-RCE".into(),
                    severity: Severity::Critical,
                    file: filename.into(),
                    line: line_num + 1,
                    description: "Shell command execution detected".into(),
                });
            }
            
            if line.contains("import socket") || line.contains("from socket") {
                findings.push(Finding {
                    code: "SEC-NET".into(),
                    severity: Severity::Critical,
                    file: filename.into(),
                    line: line_num + 1,
                    description: "Network socket import detected".into(),
                });
            }
        }
        
        findings
    }
}

impl Default for AstScanner {
    fn default() -> Self {
        Self::new()
    }
}
