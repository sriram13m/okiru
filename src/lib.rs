mod monitor;
mod ollama;
mod storage;

pub use monitor::{AppInfo, MonitorConfig, MonitorError, get_active_window, start_monitoring};
pub use ollama::{AlignmentEvaluation, OllamaClient, OllamaError};
pub use storage::{ActivityLogger, StorageError};
