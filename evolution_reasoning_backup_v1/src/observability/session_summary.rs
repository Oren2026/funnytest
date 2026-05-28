//! Session 摘要（Session Summary）
//!
//! 在每次 session 結束時自動產生摘要報告。

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

/// Session 統計資料
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    /// 討論主題
    pub topic: String,
    /// 總回合數
    pub total_rounds: usize,
    /// 新增節點數
    pub nodes_added: usize,
    /// 約束變化次數
    pub constraints_changed: usize,
    /// 階段轉換次數
    pub phase_transitions: usize,
    /// 提問總數（估算）
    pub questions_asked: usize,
    /// 最終節點數
    pub final_node_count: usize,
    /// 最終邊數
    pub final_edge_count: usize,
    /// 最終複雜度
    pub final_complexity: f64,
    /// 最終階段
    pub final_phase: String,
}

impl SessionStats {
    /// 建立新的 SessionStats（預設值）
    pub fn new(topic: &str) -> Self {
        SessionStats {
            topic: topic.to_string(),
            ..Default::default()
        }
    }
}

/// Session Summary Logger
///
/// 負責寫入 Session 摘要。
/// 檔案位置：`workspace/logs/session_{timestamp}.md`
#[derive(Debug, Clone)]
pub struct SessionSummaryLogger {
    /// 統計資料
    stats: SessionStats,
    /// 日誌檔案路徑
    log_path: PathBuf,
}

impl SessionSummaryLogger {
    /// 建立新的 SessionSummaryLogger
    pub fn new(logs_dir: &PathBuf, topic: &str) -> std::io::Result<Self> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("session_{}.md", timestamp);
        let log_path = logs_dir.join(&filename);

        let stats = SessionStats::new(topic);

        Ok(SessionSummaryLogger { stats, log_path })
    }

    /// 更新統計資料
    pub fn update_stats(&mut self, stats: SessionStats) {
        self.stats = stats;
    }

    /// 增加回合數
    pub fn increment_rounds(&mut self) {
        self.stats.total_rounds += 1;
    }

    /// 增加節點數
    pub fn add_nodes(&mut self, count: usize) {
        self.stats.nodes_added += count;
    }

    /// 增加約束變化
    pub fn add_constraint_change(&mut self) {
        self.stats.constraints_changed += 1;
    }

    /// 增加階段轉換
    pub fn add_phase_transition(&mut self) {
        self.stats.phase_transitions += 1;
    }

    /// 增加提問數
    pub fn add_question(&mut self) {
        self.stats.questions_asked += 1;
    }

    /// 設定最終狀態
    pub fn set_final_state(
        &mut self,
        node_count: usize,
        edge_count: usize,
        complexity: f64,
        phase: &str,
    ) {
        self.stats.final_node_count = node_count;
        self.stats.final_edge_count = edge_count;
        self.stats.final_complexity = complexity;
        self.stats.final_phase = phase.to_string();
    }

    /// 寫入 Session 摘要到檔案
    pub fn write_summary(&self) -> std::io::Result<PathBuf> {
        // 確保父目錄存在
        if let Some(parent) = self.log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&self.log_path)?;

        writeln!(file, "# Session 摘要\n")?;
        writeln!(file, "**時間：** {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file, "")?;
        writeln!(file, "---\n")?;

        // 基本資訊
        writeln!(file, "## 基本資訊\n")?;
        writeln!(file, "- **討論主題：** {}", self.stats.topic)?;
        writeln!(file, "- **總回合數：** {}", self.stats.total_rounds)?;
        writeln!(file, "- **新增節點數：** {}", self.stats.nodes_added)?;
        writeln!(file, "- **約束變化次數：** {}", self.stats.constraints_changed)?;
        writeln!(file, "- **階段轉換次數：** {}", self.stats.phase_transitions)?;
        writeln!(file, "- **提問總數（估算）：** {}", self.stats.questions_asked)?;
        writeln!(file, "")?;

        // 最終狀態
        writeln!(file, "## 最終狀態\n")?;
        writeln!(file, "- **節點數：** {}", self.stats.final_node_count)?;
        writeln!(file, "- **邊數：** {}", self.stats.final_edge_count)?;
        writeln!(file, "- **總複雜度：** {:.2}", self.stats.final_complexity)?;
        writeln!(file, "- **最終階段：** {}", self.stats.final_phase)?;
        writeln!(file, "")?;

        // 估算提問密度（每回合提問數）
        let question_density = if self.stats.total_rounds > 0 {
            self.stats.questions_asked as f64 / self.stats.total_rounds as f64
        } else {
            0.0
        };
        writeln!(file, "## 提問習慣分析\n")?;
        writeln!(
            file,
            "- **平均每回合提問數：** {:.1}",
            question_density
        )?;
        writeln!(
            file,
            "- **提問密度評估：** {}",
            if question_density >= 1.0 {
                "高（符合探索期特徵）"
            } else if question_density >= 0.5 {
                "中（正常對話）"
            } else {
                "低（可能進入成熟期）"
            }
        )?;
        writeln!(file, "")?;

        writeln!(file, "---\n")?;
        writeln!(file, "*此摘要由 Evolution Reasoning Tool v0.6 自動產生*\n")?;

        Ok(self.log_path.clone())
    }

    /// 取得統計資料的參考
    pub fn stats(&self) -> &SessionStats {
        &self.stats
    }

    /// 取得日誌檔案路徑
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_stats_new() {
        let stats = SessionStats::new("測試主題");
        assert_eq!(stats.topic, "測試主題");
        assert_eq!(stats.total_rounds, 0);
    }

    #[test]
    fn test_session_summary_logger() {
        let temp_path = std::env::temp_dir().join("test_logs");
        let logger = SessionSummaryLogger::new(&temp_path, "測試主題").unwrap();

        assert!(logger.log_path().to_str().unwrap().contains("session_"));
        assert_eq!(logger.stats().topic, "測試主題");
    }

    #[test]
    fn test_increment_stats() {
        let temp_path = std::env::temp_dir().join("test_logs");
        let mut logger = SessionSummaryLogger::new(&temp_path, "測試").unwrap();

        logger.increment_rounds();
        logger.add_nodes(3);
        logger.add_constraint_change();
        logger.add_phase_transition();
        logger.add_question();

        assert_eq!(logger.stats().total_rounds, 1);
        assert_eq!(logger.stats().nodes_added, 3);
        assert_eq!(logger.stats().constraints_changed, 1);
        assert_eq!(logger.stats().phase_transitions, 1);
        assert_eq!(logger.stats().questions_asked, 1);
    }

    #[test]
    fn test_set_final_state() {
        let temp_path = std::env::temp_dir().join("test_logs");
        let mut logger = SessionSummaryLogger::new(&temp_path, "測試").unwrap();

        logger.set_final_state(10, 15, 42.5, "成熟期");

        assert_eq!(logger.stats().final_node_count, 10);
        assert_eq!(logger.stats().final_edge_count, 15);
        assert!((logger.stats().final_complexity - 42.5).abs() < 0.001);
        assert_eq!(logger.stats().final_phase, "成熟期");
    }

    #[test]
    fn test_write_summary() {
        let temp_path = std::env::temp_dir().join("test_logs");
        let mut logger = SessionSummaryLogger::new(&temp_path, "測試主題").unwrap();

        logger.increment_rounds();
        logger.increment_rounds();
        logger.add_nodes(5);
        logger.add_question();
        logger.add_question();
        logger.set_final_state(5, 4, 25.0, "發展期");

        let result = logger.write_summary();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Session 摘要"));
        assert!(content.contains("測試主題"));
        // 檢查包含關鍵字（因為有 markdown 格式，所以只檢查部分內容）
        assert!(content.contains("總回合數"));
        assert!(content.contains("發展期"));

        // cleanup
        let _ = std::fs::remove_file(&path);
    }
}
