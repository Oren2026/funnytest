//! 約束變化日誌（Constraint Change Log）
//!
//! 記錄約束條件的新增、刪除等變化。

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use crate::engine::constraint::{Constraint, ConstraintSource};

/// 約束變化類型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintChangeType {
    /// 新增
    Added,
    /// 移除
    Removed,
}

/// 約束變化記錄
#[derive(Debug, Clone)]
pub struct ConstraintChange {
    /// 發生時間
    pub timestamp: String,
    /// 變化類型
    pub change_type: ConstraintChangeType,
    /// 約束內容
    pub content: String,
    /// 約束來源
    pub source: ConstraintSource,
    /// 約束 ID
    pub constraint_id: String,
}

/// Constraint Change Logger
///
/// 負責寫入約束變化日誌。
/// 檔案位置：`workspace/logs/constraints.md`
#[derive(Debug, Clone)]
pub struct ConstraintChangeLogger {
    /// 日誌檔案路徑
    log_path: PathBuf,
    /// 是否已初始化（已有標頭）
    initialized: bool,
}

impl ConstraintChangeLogger {
    /// 建立新的 ConstraintChangeLogger
    ///
    /// 如果檔案已存在，會附加到現有檔案。
    pub fn new(logs_dir: &PathBuf) -> std::io::Result<Self> {
        let filename = "constraints.md";
        let log_path = logs_dir.join(filename);

        let initialized = log_path.exists();

        // 如果檔案不存在，建立並寫入標頭
        if !initialized {
            let mut file = File::create(&log_path)?;
            writeln!(file, "# 約束變化記錄\n")?;
            writeln!(file, "| 時間 | 類型 | 內容 | 來源 | ID |")?;
            writeln!(file, "|------|------|------|------|----|")?;
        }

        Ok(ConstraintChangeLogger {
            log_path,
            initialized: true,
        })
    }

    /// 記錄約束新增
    pub fn log_added(&self, constraint: &Constraint) -> std::io::Result<()> {
        let change = ConstraintChange {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            change_type: ConstraintChangeType::Added,
            content: constraint.content.clone(),
            source: constraint.source.clone(),
            constraint_id: constraint.id.clone(),
        };
        self.log_change(&change)
    }

    /// 記錄約束移除
    pub fn log_removed(&self, constraint: &Constraint) -> std::io::Result<()> {
        let change = ConstraintChange {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            change_type: ConstraintChangeType::Removed,
            content: constraint.content.clone(),
            source: constraint.source.clone(),
            constraint_id: constraint.id.clone(),
        };
        self.log_change(&change)
    }

    /// 記錄一次約束變化
    fn log_change(&self, change: &ConstraintChange) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.log_path)?;

        let change_type_str = match change.change_type {
            ConstraintChangeType::Added => "新增",
            ConstraintChangeType::Removed => "移除",
        };

        let source_str = match &change.source {
            ConstraintSource::User => "用戶",
            ConstraintSource::Gemma => "自動萃取",
        };

        // 內容太長時截斷
        let content_preview = if change.content.len() > 50 {
            format!("{}...", &change.content[..50])
        } else {
            change.content.clone()
        };

        writeln!(
            file,
            "| {} | {} | {} | {} | {} |",
            change.timestamp,
            change_type_str,
            escape_markdown(&content_preview),
            source_str,
            &change.constraint_id[..8]
        )?;

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
    fn test_change_type_name() {
        assert_eq!(
            format!("{:?}", ConstraintChangeType::Added),
            "Added"
        );
        assert_eq!(
            format!("{:?}", ConstraintChangeType::Removed),
            "Removed"
        );
    }

    #[test]
    fn test_escape_markdown() {
        let input = "**bold** and `code`";
        let escaped = escape_markdown(input);
        assert!(!escaped.contains("**"));
        assert!(!escaped.contains("`"));
    }

    #[test]
    fn test_constraint_change() {
        let change = ConstraintChange {
            timestamp: "2026-05-07 10:30:00".to_string(),
            change_type: ConstraintChangeType::Added,
            content: "測試約束".to_string(),
            source: ConstraintSource::User,
            constraint_id: "test-id-123".to_string(),
        };
        assert_eq!(change.change_type, ConstraintChangeType::Added);
    }
}
