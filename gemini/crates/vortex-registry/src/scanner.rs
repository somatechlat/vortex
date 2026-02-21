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
        for (idx, line) in content.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            if let Some(module_name) = self.parse_import_module(trimmed) {
                if self.is_dangerous_module(module_name) {
                    findings.push(Finding {
                        code: "SEC-IMPT".into(),
                        severity: Severity::Critical,
                        file: filename.into(),
                        line: line_no,
                        description: format!("Dangerous import: {}", module_name),
                    });
                }
            }

            for call in self.dangerous_calls() {
                if self.line_contains_call(trimmed, call) {
                    findings.push(Finding {
                        code: "SEC-CALL".into(),
                        severity: Severity::Critical,
                        file: filename.into(),
                        line: line_no,
                        description: format!("Dangerous function call: {}", call),
                    });
                }
            }
        }

        findings
    }

    fn parse_import_module<'a>(&self, line: &'a str) -> Option<&'a str> {
        if let Some(rest) = line.strip_prefix("import ") {
            let first = rest.split(',').next()?.trim();
            return Some(first.split_whitespace().next()?);
        }

        if let Some(rest) = line.strip_prefix("from ") {
            let module = rest.split_whitespace().next()?;
            return Some(module);
        }

        None
    }

    fn dangerous_calls(&self) -> &'static [&'static str] {
        &[
            "os.system",
            "os.popen",
            "subprocess.run",
            "subprocess.Popen",
            "subprocess.call",
            "subprocess.check_call",
            "subprocess.check_output",
            "eval",
            "exec",
            "open",
        ]
    }

    fn line_contains_call(&self, line: &str, call: &str) -> bool {
        let needle = format!("{call}(");
        line.contains(&needle)
    }

    fn is_dangerous_module(&self, name: &str) -> bool {
        matches!(name, "os" | "subprocess" | "socket" | "requests" | "urllib" | "shutil" | "pty")
    }
}

impl Default for AstScanner {
    fn default() -> Self {
        Self::new()
    }
}
