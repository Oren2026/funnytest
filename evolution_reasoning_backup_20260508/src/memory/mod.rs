//! 長期記憶系統（Long-term Memory）
//!
//! v0.5 新增：持久化的用戶記憶系統。
//!
//! 負責：
//! - 用戶基本資料和偏好（profile.md）
//! - 歷史討論摘要（history.md）
//! - 已探索過的主題和結論（topics.md）
//!
//! # 檔案位置
//!
//! ```ignore
//! ~/.evolution_reasoning/workspace/memory/
//! ├── profile.md   # 用戶基本資料、偏好
//! ├── history.md   # 歷史討論摘要
//! └── topics.md    # 已探索過的主題和結論
//! ```

use std::path::PathBuf;
use chrono::{DateTime, Local};

/// Memory 資料夾名稱
const MEMORY_DIR: &str = "memory";
/// Memory 文件名稱
const MEMORY_PROFILE: &str = "profile.md";
const MEMORY_HISTORY: &str = "history.md";
const MEMORY_TOPICS: &str = "topics.md";

/// 用戶基本資料結構
#[derive(Debug, Clone, Default)]
pub struct UserProfile {
    /// 用戶名稱（可選）
    pub name: Option<String>,
    /// 偏好設定（JSON 格式字串）
    pub preferences: String,
    /// 創建時間
    pub created_at: DateTime<Local>,
    /// 最後更新時間
    pub updated_at: DateTime<Local>,
}

impl UserProfile {
    /// 建立新的空白 Profile
    pub fn new() -> Self {
        let now = Local::now();
        UserProfile {
            name: None,
            preferences: "{}".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 從 Markdown 內容解析 Profile
    ///
    /// 格式：
    /// ```markdown
    /// # User Profile
    ///
    /// - name: [optional name]
    /// - preferences: { ... JSON ... }
    /// - created_at: 2026-05-07T10:00:00+08:00
    /// - updated_at: 2026-05-07T10:00:00+08:00
    /// ```
    pub fn from_md(content: &str) -> Self {
        let mut profile = UserProfile::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("- name:") {
                let name = line.trim_start_matches("- name:").trim();
                if !name.is_empty() && name != "null" {
                    profile.name = Some(name.to_string());
                }
            } else if line.starts_with("- preferences:") {
                let prefs = line.trim_start_matches("- preferences:").trim();
                if !prefs.is_empty() {
                    profile.preferences = prefs.to_string();
                }
            } else if line.starts_with("- created_at:") {
                let ts = line.trim_start_matches("- created_at:").trim();
                if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                    profile.created_at = dt.with_timezone(&Local);
                }
            } else if line.starts_with("- updated_at:") {
                let ts = line.trim_start_matches("- updated_at:").trim();
                if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                    profile.updated_at = dt.with_timezone(&Local);
                }
            }
        }

        profile
    }

    /// 轉換為 Markdown 格式
    pub fn to_md(&self) -> String {
        let mut md = String::from("# User Profile\n\n");
        md.push_str(&format!("- name: {}\n", self.name.as_deref().unwrap_or("null")));
        md.push_str(&format!("- preferences: {}\n", self.preferences));
        md.push_str(&format!("- created_at: {}\n", self.created_at.to_rfc3339()));
        md.push_str(&format!("- updated_at: {}\n", self.updated_at.to_rfc3339()));
        md
    }
}

/// 歷史討論記錄結構
#[derive(Debug, Clone, Default)]
pub struct HistoryEntry {
    /// 日期時間
    pub datetime: DateTime<Local>,
    /// 主題
    pub topic: String,
    /// 摘要
    pub summary: String,
    /// 節點數量
    pub node_count: usize,
}

impl HistoryEntry {
    /// 建立新的歷史記錄
    pub fn new(topic: impl Into<String>, summary: impl Into<String>, node_count: usize) -> Self {
        HistoryEntry {
            datetime: Local::now(),
            topic: topic.into(),
            summary: summary.into(),
            node_count,
        }
    }

    /// 從 Markdown 格式解析（單一區塊）
    pub fn from_md_block(block: &str) -> Self {
        let mut entry = HistoryEntry::new("", "", 0);

        for line in block.lines() {
            let line = line.trim();
            if line.starts_with("- datetime:") {
                let dt_str = line.trim_start_matches("- datetime:").trim();
                if let Ok(dt) = DateTime::parse_from_rfc3339(dt_str) {
                    entry.datetime = dt.with_timezone(&Local);
                }
            } else if line.starts_with("- topic:") {
                entry.topic = line.trim_start_matches("- topic:").trim().to_string();
            } else if line.starts_with("- node_count:") {
                let count_str = line.trim_start_matches("- node_count:").trim();
                entry.node_count = count_str.parse().unwrap_or(0);
            } else if line.starts_with("- summary:") {
                entry.summary = line.trim_start_matches("- summary:").trim().to_string();
            }
        }

        entry
    }

    /// 轉換為 Markdown 區塊格式
    pub fn to_md_block(&self) -> String {
        format!(
            "- datetime: {}\n- topic: {}\n- node_count: {}\n- summary: {}",
            self.datetime.to_rfc3339(),
            self.topic,
            self.node_count,
            self.summary
        )
    }
}

/// 已探索主題結構
#[derive(Debug, Clone, Default)]
pub struct ExploredTopic {
    /// 主題名稱
    pub name: String,
    /// 標籤（多個用逗號分隔）
    pub tags: Vec<String>,
    /// 結論摘要
    pub conclusion: String,
    /// 探索時間
    pub explored_at: DateTime<Local>,
}

impl ExploredTopic {
    /// 建立新的已探索主題
    pub fn new(name: impl Into<String>, tags: Vec<String>, conclusion: impl Into<String>) -> Self {
        ExploredTopic {
            name: name.into(),
            tags,
            conclusion: conclusion.into(),
            explored_at: Local::now(),
        }
    }

    /// 從 Markdown 區塊解析
    pub fn from_md_block(block: &str) -> Self {
        let mut topic = ExploredTopic::new("", vec![], "");

        for line in block.lines() {
            let line = line.trim();
            if line.starts_with("- name:") {
                topic.name = line.trim_start_matches("- name:").trim().to_string();
            } else if line.starts_with("- tags:") {
                let tags_str = line.trim_start_matches("- tags:").trim();
                topic.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
            } else if line.starts_with("- explored_at:") {
                let dt_str = line.trim_start_matches("- explored_at:").trim();
                if let Ok(dt) = DateTime::parse_from_rfc3339(dt_str) {
                    topic.explored_at = dt.with_timezone(&Local);
                }
            } else if line.starts_with("- conclusion:") {
                topic.conclusion = line.trim_start_matches("- conclusion:").trim().to_string();
            }
        }

        topic
    }

    /// 轉換為 Markdown 區塊格式
    pub fn to_md_block(&self) -> String {
        format!(
            "- name: {}\n- tags: {}\n- explored_at: {}\n- conclusion: {}",
            self.name,
            self.tags.join(", "),
            self.explored_at.to_rfc3339(),
            self.conclusion
        )
    }
}

/// 長期記憶管理器
///
/// 負責讀寫三個記憶檔案：
/// - profile.md：用戶基本資料
/// - history.md：歷史討論摘要
/// - topics.md：已探索過的主題
#[derive(Debug, Clone)]
pub struct MemoryManager {
    /// Memory 目錄路徑
    root: PathBuf,
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryManager {
    /// 建立新的 Memory Manager
    ///
    /// 使用預設路徑：`~/.evolution_reasoning/workspace/memory/`
    pub fn new() -> Self {
        let root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".evolution_reasoning")
            .join("workspace")
            .join(MEMORY_DIR);

        MemoryManager { root }
    }

    /// 建立有自訂路徑的 Memory Manager
    #[allow(dead_code)]
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        MemoryManager { root: path.into() }
    }

    /// 取得 memory 目錄路徑
    pub fn memory_path(&self) -> &PathBuf {
        &self.root
    }

    /// 確保 memory 目錄存在
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)
    }

    /// 讀取用戶 profile
    ///
    /// 回傳 (存在, 內容)
    pub fn read_profile(&self) -> (bool, String) {
        let path = self.root.join(MEMORY_PROFILE);
        if path.exists() {
            (true, std::fs::read_to_string(path).unwrap_or_default())
        } else {
            (false, String::new())
        }
    }

    /// 寫入用戶 profile
    pub fn write_profile(&self, content: &str) -> std::io::Result<()> {
        self.ensure_dir()?;
        let path = self.root.join(MEMORY_PROFILE);
        std::fs::write(path, content)
    }

    /// 讀取歷史討論摘要
    pub fn read_history(&self) -> (bool, String) {
        let path = self.root.join(MEMORY_HISTORY);
        if path.exists() {
            (true, std::fs::read_to_string(path).unwrap_or_default())
        } else {
            (false, String::new())
        }
    }

    /// 附加歷史討論（追加模式）
    pub fn append_history(&self, content: &str) -> std::io::Result<()> {
        self.ensure_dir()?;
        let path = self.root.join(MEMORY_HISTORY);
        let existing = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };
        let new_content = if existing.is_empty() {
            content.to_string()
        } else {
            format!("{}\n\n---\n\n{}", existing, content)
        };
        std::fs::write(path, new_content)
    }

    /// 讀取已探索主題
    pub fn read_topics(&self) -> (bool, String) {
        let path = self.root.join(MEMORY_TOPICS);
        if path.exists() {
            (true, std::fs::read_to_string(path).unwrap_or_default())
        } else {
            (false, String::new())
        }
    }

    /// 寫入已探索主題（覆蓋模式）
    pub fn write_topics(&self, content: &str) -> std::io::Result<()> {
        self.ensure_dir()?;
        let path = self.root.join(MEMORY_TOPICS);
        std::fs::write(path, content)
    }

    /// 讀取所有長期記憶（所有三個檔案）
    ///
    /// 回傳 (profile, history, topics)
    pub fn read_all(&self) -> (String, String, String) {
        let (_, profile) = self.read_profile();
        let (_, history) = self.read_history();
        let (_, topics) = self.read_topics();
        (profile, history, topics)
    }

    /// 格式化為系統提示詞字串
    ///
    /// 將長期記憶格式化為 gemma4 可理解的提示詞片段。
    pub fn format_for_prompt(&self) -> String {
        let (profile_exists, profile) = self.read_profile();
        let (_, history) = self.read_history();
        let (_, topics) = self.read_topics();

        let mut result = String::from("\n\n=== 長期記憶 ===\n");

        if profile_exists && !profile.is_empty() {
            result.push_str("【用戶資料】\n");
            result.push_str(&profile);
            result.push_str("\n");
        }

        if !history.is_empty() {
            result.push_str("【歷史討論】\n");
            // 只顯示最後 3 筆
            let all_entries: Vec<&str> = history.split("\n\n---\n\n").collect();
            let entries: Vec<&str> = all_entries.iter().rev().take(3).map(|s| *s).collect();
            for entry in entries {
                if !entry.trim().is_empty() {
                    result.push_str("- ");
                    result.push_str(entry.trim());
                    result.push_str("\n");
                }
            }
            result.push_str("\n");
        }

        if !topics.is_empty() {
            result.push_str("【已探索主題】\n");
            // 只顯示前 5 個
            let topic_blocks: Vec<&str> = topics.split("\n\n").filter(|s| !s.trim().is_empty()).take(5).collect();
            for block in topic_blocks {
                result.push_str("- ");
                result.push_str(block.trim());
                result.push_str("\n");
            }
            result.push_str("\n");
        }

        result.push_str("=== 長期記憶結束 ===\n");

        // 如果沒有任何記憶，回傳空白
        if !profile_exists && history.is_empty() && topics.is_empty() {
            String::new()
        } else {
            result
        }
    }

    /// 新增歷史記錄
    pub fn add_history_entry(&self, topic: &str, summary: &str, node_count: usize) -> std::io::Result<()> {
        let entry = HistoryEntry::new(topic, summary, node_count);
        self.append_history(&entry.to_md_block())
    }

    /// 新增已探索主題
    pub fn add_explored_topic(&self, name: &str, tags: Vec<String>, conclusion: &str) -> std::io::Result<()> {
        let (_, existing) = self.read_topics();
        let mut topics: Vec<String> = existing
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        let new_topic = ExploredTopic::new(name, tags, conclusion);
        topics.push(new_topic.to_md_block());

        // 保留最新的 20 個主題
        if topics.len() > 20 {
            topics = topics.into_iter().rev().take(20).rev().collect();
        }

        let content = topics.join("\n\n");
        self.write_topics(&content)
    }

    /// 更新用戶名稱
    pub fn set_user_name(&self, name: &str) -> std::io::Result<()> {
        let (exists, content) = self.read_profile();
        let mut profile = if exists && !content.is_empty() {
            UserProfile::from_md(&content)
        } else {
            UserProfile::new()
        };

        profile.name = Some(name.to_string());
        profile.updated_at = Local::now();
        self.write_profile(&profile.to_md())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_new() {
        let profile = UserProfile::new();
        assert!(profile.name.is_none());
        assert_eq!(profile.preferences, "{}");
    }

    #[test]
    fn test_user_profile_to_md() {
        let profile = UserProfile::new();
        let md = profile.to_md();
        assert!(md.contains("# User Profile"));
        assert!(md.contains("- name: null"));
    }

    #[test]
    fn test_user_profile_from_md() {
        let md = r#"# User Profile

- name: 測試用戶
- preferences: {"theme": "dark"}
- created_at: 2026-05-07T10:00:00+08:00
- updated_at: 2026-05-07T10:00:00+08:00
"#;
        let profile = UserProfile::from_md(md);
        assert_eq!(profile.name, Some("測試用戶".to_string()));
        assert!(profile.preferences.contains("dark"));
    }

    #[test]
    fn test_history_entry_new() {
        let entry = HistoryEntry::new("測試主題", "這是摘要", 5);
        assert_eq!(entry.topic, "測試主題");
        assert_eq!(entry.summary, "這是摘要");
        assert_eq!(entry.node_count, 5);
    }

    #[test]
    fn test_history_entry_to_md_block() {
        let entry = HistoryEntry::new("測試主題", "摘要", 3);
        let block = entry.to_md_block();
        assert!(block.contains("測試主題"));
        assert!(block.contains("3"));
    }

    #[test]
    fn test_explored_topic_new() {
        let topic = ExploredTopic::new("生涯規劃", vec!["工作".to_string(), "生活".to_string()], "結論");
        assert_eq!(topic.name, "生涯規劃");
        assert_eq!(topic.tags.len(), 2);
    }

    #[test]
    fn test_memory_manager_new() {
        let mgr = MemoryManager::new();
        assert!(mgr.memory_path().ends_with("memory"));
    }

    #[test]
    fn test_memory_manager_format_for_prompt_empty() {
        let mgr = MemoryManager::new();
        let formatted = mgr.format_for_prompt();
        // 空的時候回傳空白字串
        assert_eq!(formatted, String::new());
    }
}
