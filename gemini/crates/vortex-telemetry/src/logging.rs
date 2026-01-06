//! Structured JSON Logging

/// Configure logging for different environments
pub struct LoggingConfig {
    pub json: bool,
    pub span_events: bool,
    pub file: bool,
    pub line_number: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            json: true,
            span_events: true,
            file: true,
            line_number: true,
        }
    }
}

impl LoggingConfig {
    pub fn development() -> Self {
        Self {
            json: false,
            span_events: false,
            file: true,
            line_number: true,
        }
    }
    
    pub fn production() -> Self {
        Self::default()
    }
}
