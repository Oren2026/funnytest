//! 對話日誌（Conversation Log）
//!
//! 記錄每個對話回合的輸入、輸出和工具呼叫。

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

/// 對話回合記錄
#[derive(Debug, Clone)]
pub struct ConversationRound {
    /// 回合編號（從 1 開始）
    pub round: usize,
    /// 使用者輸入
    pub user_input: String,
    /// gemma4 回覆
    pub gemma_response: String,
    /// 工具呼叫（名稱和參數）
    pub tool_calls: Vec<ToolCallRecord>,
}

/// 工具呼叫記錄
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    /// 工具名稱
    pub name: String,
    /// 工具參數（JSON 字串）
    pub arguments: String,
}

/// Conversation Logger
///
/// 負責寫入對話日誌到 markdown 檔案。
#[derive(Debug, Clone)]
pub struct ConversationLogger {
    /// 日誌檔案路徑
    log_path: PathBuf,
}

impl ConversationLogger {
    /// 建立新的 ConversationLogger
    ///
    /// 每次建立都會建立新的時間戳記檔案。
    pub fn new(logs_dir: &PathBuf) -> std::io::Result<Self> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("conversation_{}.md", timestamp);
        let log_path = logs_dir.join(&filename);

        // 建立檔案並寫入標頭
        let mut file = File::create(&log_path)?;
        writeln!(file, "# 對話日誌")?;
        writeln!(file, "")?;
        writeln!(file, "時間：{}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file, "")?;

        Ok(ConversationLogger { log_path })
    }

    /// 建立有指定主題的 ConversationLogger
    pub fn with_topic(logs_dir: &PathBuf, topic: &str) -> std::io::Result<Self> {
        let logger = Self::new(logs_dir)?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&logger.log_path)?;

        writeln!(file, "主題：{}\n", topic)?;
        writeln!(file, "---\n")?;

        Ok(logger)
    }

    /// 記錄一個對話回合
    pub fn log_round(&self, round: &ConversationRound) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "## 回合 {}", round.round)?;
        writeln!(file, "")?;
        writeln!(file, "**使用者：** {}", escape_markdown(&round.user_input))?;
        writeln!(file, "")?;
        writeln!(file, "**gemma4：** {}", escape_markdown(&round.gemma_response))?;
        writeln!(file, "")?;

        if !round.tool_calls.is_empty() {
            writeln!(file, "**工具呼叫：**")?;
            writeln!(file, "")?;
            for tc in &round.tool_calls {
                writeln!(file, "- `{}({})`", tc.name, tc.arguments)?;
            }
            writeln!(file, "")?;
        }

        writeln!(file, "---\n")?;

        Ok(())
    }

    /// 取得日誌檔案路徑
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

/// 逸出 Markdown 特殊字元
fn escape_markdown(s: &str) -> String {
    // 注意：Markdown 中 backtick 需要用雙 backtick 包裹或其他方式處理
    // 這裡我們將 backtick 替換為 HTML entity
    s.replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "&#96;")
        .replace('#', "\\#")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        let input = "**bold** and `code`";
        let escaped = escape_markdown(input);
        assert!(!escaped.contains("**"));
        assert!(!escaped.contains("`"));
    }

    #[test]
    fn test_conversation_round() {
        let round = ConversationRound {
            round: 1,
            user_input: "測試輸入".to_string(),
            gemma_response: "測試回覆".to_string(),
            tool_calls: vec![],
        };
        assert_eq!(round.round, 1);
    }

    #[test]
    fn test_tool_call_record() {
        let tc = ToolCallRecord {
            name: "diverge".to_string(),
            arguments: r#"{"root": "node1", "content": "test", "count": 3}"#.to_string(),
        };
        assert_eq!(tc.name, "diverge");
    }
}
