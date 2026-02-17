use std::path::Path;
use rustpython_parser::{ast, parser};

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

        // Parse Python code into AST
        let ast = match parser::parse(content, parser::Mode::Module, filename) {
            Ok(ast) => ast,
            Err(e) => {
                findings.push(Finding {
                    code: "SEC-PARSE".into(),
                    severity: Severity::High,
                    file: filename.into(),
                    line: e.location.row(),
                    description: format!("Failed to parse Python code: {}", e),
                });
                return findings;
            }
        };

        // Traverse AST using a visitor
        if let ast::Mod::Module(module) = ast {
            for stmt in &module.body {
                self.visit_statement(stmt, filename, &mut findings);
            }
        }

        findings
    }

    fn visit_statement(&self, stmt: &ast::Stmt, filename: &str, findings: &mut Vec<Finding>) {
        match &stmt.node {
            ast::StmtKind::Import { names } => {
                for name in names {
                    if self.is_dangerous_module(&name.node.name) {
                        findings.push(Finding {
                            code: "SEC-IMPT".into(),
                            severity: Severity::Critical,
                            file: filename.into(),
                            line: stmt.location.row(),
                            description: format!("Dangerous import: {}", name.node.name),
                        });
                    }
                }
            }
            ast::StmtKind::ImportFrom { module, .. } => {
                if let Some(mod_name) = module {
                    if self.is_dangerous_module(mod_name) {
                        findings.push(Finding {
                            code: "SEC-IMPT".into(),
                            severity: Severity::Critical,
                            file: filename.into(),
                            line: stmt.location.row(),
                            description: format!("Dangerous from-import: {}", mod_name),
                        });
                    }
                }
            }
            ast::StmtKind::FunctionDef { body, .. } | ast::StmtKind::AsyncFunctionDef { body, .. } => {
                for inner_stmt in body {
                    self.visit_statement(inner_stmt, filename, findings);
                }
            }
            ast::StmtKind::ClassDef { body, .. } => {
                for inner_stmt in body {
                    self.visit_statement(inner_stmt, filename, findings);
                }
            }
            ast::StmtKind::If { body, orelse, .. } => {
                for inner_stmt in body {
                    self.visit_statement(inner_stmt, filename, findings);
                }
                for inner_stmt in orelse {
                    self.visit_statement(inner_stmt, filename, findings);
                }
            }
            ast::StmtKind::While { body, orelse, .. } | ast::StmtKind::For { body, orelse, .. } => {
                for inner_stmt in body {
                    self.visit_statement(inner_stmt, filename, findings);
                }
                for inner_stmt in orelse {
                    self.visit_statement(inner_stmt, filename, findings);
                }
            }
            ast::StmtKind::Try { body, orelse, finalbody, .. } => {
                for inner_stmt in body {
                    self.visit_statement(inner_stmt, filename, findings);
                }
                for inner_stmt in orelse {
                    self.visit_statement(inner_stmt, filename, findings);
                }
                for inner_stmt in finalbody {
                    self.visit_statement(inner_stmt, filename, findings);
                }
            }
            ast::StmtKind::Expr { value } => {
                self.visit_expression(value, filename, findings);
            }
            _ => {}
        }
    }

    fn visit_expression(&self, expr: &ast::Expr, filename: &str, findings: &mut Vec<Finding>) {
        match &expr.node {
            ast::ExprKind::Call { func, args, keywords } => {
                self.check_call(func, filename, expr.location.row(), findings);
                for arg in args {
                    self.visit_expression(arg, filename, findings);
                }
                for kw in keywords {
                    self.visit_expression(&kw.node.value, filename, findings);
                }
            }
            ast::ExprKind::Attribute { value, .. } => {
                self.visit_expression(value, filename, findings);
            }
            ast::ExprKind::BinOp { left, right, .. } => {
                self.visit_expression(left, filename, findings);
                self.visit_expression(right, filename, findings);
            }
            ast::ExprKind::BoolOp { values, .. } => {
                for val in values {
                    self.visit_expression(val, filename, findings);
                }
            }
            ast::ExprKind::List { elts, .. } | ast::ExprKind::Tuple { elts, .. } | ast::ExprKind::Set { elts, .. } => {
                for elt in elts {
                    self.visit_expression(elt, filename, findings);
                }
            }
            ast::ExprKind::Dict { keys, values } => {
                for key in keys.iter().flatten() {
                    self.visit_expression(key, filename, findings);
                }
                for val in values {
                    self.visit_expression(val, filename, findings);
                }
            }
            _ => {}
        }
    }

    fn check_call(&self, func: &ast::Expr, filename: &str, line: usize, findings: &mut Vec<Finding>) {
        let call_name = match &func.node {
            ast::ExprKind::Name { id, .. } => Some(id.to_string()),
            ast::ExprKind::Attribute { value, attr, .. } => {
                if let ast::ExprKind::Name { id, .. } = &value.node {
                    Some(format!("{}.{}", id, attr))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(name) = call_name {
            if self.is_dangerous_call(&name) {
                findings.push(Finding {
                    code: "SEC-CALL".into(),
                    severity: Severity::Critical,
                    file: filename.into(),
                    line,
                    description: format!("Dangerous function call: {}", name),
                });
            }
        }
    }

    fn is_dangerous_module(&self, name: &str) -> bool {
        matches!(name, "os" | "subprocess" | "socket" | "requests" | "urllib" | "shutil" | "pty")
    }

    fn is_dangerous_call(&self, name: &str) -> bool {
        matches!(name, "os.system" | "os.popen" | "subprocess.run" | "subprocess.Popen" | "subprocess.call" | "subprocess.check_call" | "subprocess.check_output" | "eval" | "exec" | "open")
    }
}

impl Default for AstScanner {
    fn default() -> Self {
        Self::new()
    }
}
