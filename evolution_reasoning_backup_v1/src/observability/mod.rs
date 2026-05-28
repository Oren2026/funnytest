//! 可觀測性系統（Observability System）
//!
//! v0.6 新增：對系統運作進行全面觀察和記錄。
//! v0.7 新增：Session 產出模板、決策樹格式化。
//!
//! 負責：
//! - 對話日誌（Conversation Log）
//! - 階段轉換日誌（Phase Transition Log）
//! - 約束變化日誌（Constraint Change Log）
//! - 節點圖快照（Graph Snapshot）
//! - Session 摘要（Session Summary）
//! - Session 產出模板（Output Template）

pub mod conversation;
pub mod constraint_log;
pub mod output_template;
pub mod phase_transition;
pub mod session_summary;
pub mod snapshot;

use std::fs;
use std::path::PathBuf;
use chrono::Local;

/// Logs 目錄名稱
const LOGS_DIR: &str = "logs";
/// Snapshots 子目錄名稱
const SNAPSHOTS_DIR: &str = "snapshots";

/// Observability Logger
///
/// 統一管理所有可觀測性日誌的寫入。
#[derive(Debug, Clone)]
pub struct ObservableLogger {
    /// Workspace 根目錄
    root: PathBuf,
    /// Logs 目錄路徑
    logs_dir: PathBuf,
    /// Snapshots 目錄路徑
    snapshots_dir: PathBuf,
}

impl Default for ObservableLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservableLogger {
    /// 建立新的 ObservableLogger
    ///
    /// 使用預設路徑：`~/.evolution_reasoning/workspace/`
    pub fn new() -> Self {
        let root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".evolution_reasoning")
            .join("workspace");

        let logs_dir = root.join(LOGS_DIR);
        let snapshots_dir = logs_dir.join(SNAPSHOTS_DIR);

        ObservableLogger {
            root,
            logs_dir,
            snapshots_dir,
        }
    }

    /// 確保所有目錄存在
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.logs_dir)?;
        fs::create_dir_all(&self.snapshots_dir)?;
        Ok(())
    }

    /// 取得 logs 目錄路徑
    pub fn logs_dir(&self) -> &PathBuf {
        &self.logs_dir
    }

    /// 取得 snapshots 目錄路徑
    pub fn snapshots_dir(&self) -> &PathBuf {
        &self.snapshots_dir
    }

    /// 產生時間戳記字串（用於檔名）
    pub fn timestamp_filename(&self) -> String {
        Local::now().format("%Y%m%d_%H%M%S").to_string()
    }

    /// 產生時間戳記字串（用於檔案內容）
    pub fn timestamp_content(&self) -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observable_logger_new() {
        let logger = ObservableLogger::new();
        assert!(logger.logs_dir().ends_with("logs"));
        assert!(logger.snapshots_dir().ends_with("snapshots"));
    }

    #[test]
    fn test_timestamp_filename() {
        let logger = ObservableLogger::new();
        let ts = logger.timestamp_filename();
        assert!(!ts.is_empty());
        assert!(ts.contains("_"));
    }

    #[test]
    fn test_timestamp_content() {
        let logger = ObservableLogger::new();
        let ts = logger.timestamp_content();
        assert!(!ts.is_empty());
        assert!(ts.contains("-"));
        assert!(ts.contains(":"));
    }
}
