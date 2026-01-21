use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use crate::domain::{Metadata, RiskAssessment, Identity};

/// A normalized log event for Cyber-Retrieval governance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub trace_id: String,
    pub user_did: String,
    pub cmd: String,
    pub params: serde_json::Value,
    pub result_ref: Option<String>,
    pub timestamp: SystemTime,
    pub metadata: Metadata,
    pub risk: RiskAssessment,
    pub authorship: Identity,
}

/// Append-only log sink trait.
pub trait LogSink: Send + Sync {
    fn append(&self, event: &LogEvent) -> Result<(), LogError>;
}

#[derive(Debug)]
pub enum LogError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
}

/// Simple file-backed append-only sink (one JSON per line).
pub struct FileLogSink {
    path: std::path::PathBuf,
}

impl FileLogSink {
    pub fn new<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl LogSink for FileLogSink {
    fn append(&self, event: &LogEvent) -> Result<(), LogError> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let serialized = serde_json::to_string(event).map_err(LogError::Serialization)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(LogError::Io)?;
        writeln!(file, "{}", serialized).map_err(LogError::Io)?;
        Ok(())
    }
}
