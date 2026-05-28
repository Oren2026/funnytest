//! 約束條件管理器（Constraint Manager）
//!
//! v0.5 新增：動態約束條件系統。
//!
//! 負責：
//! - 管理約束條件列表
//! - 從 gemma4 回覆中自動萃取新的約束
//! - 約束條件持久化到 constraints.xml

use chrono::{DateTime, Local};
use uuid::Uuid;

/// 約束條件來源
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintSource {
    /// 用戶手動輸入
    User,
    /// 從 gemma4 回覆自動萃取
    Gemma,
}

impl std::fmt::Display for ConstraintSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintSource::User => write!(f, "user"),
            ConstraintSource::Gemma => write!(f, "gemma"),
        }
    }
}

/// 約束條件
#[derive(Debug, Clone)]
pub struct Constraint {
    /// 唯一識別碼
    pub id: String,
    /// 約束內容
    pub content: String,
    /// 來源
    pub source: ConstraintSource,
    /// 創建時間
    pub created_at: DateTime<Local>,
}

impl Constraint {
    /// 建立新的約束條件
    pub fn new(content: impl Into<String>, source: ConstraintSource) -> Self {
        Constraint {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            source,
            created_at: Local::now(),
        }
    }

    /// 建立來源為用戶的約束
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(content, ConstraintSource::User)
    }

    /// 建立來源為 gemma 的約束
    pub fn gemma(content: impl Into<String>) -> Self {
        Self::new(content, ConstraintSource::Gemma)
    }
}

/// 約束條件管理器
///
/// 負責管理約束條件的萃取、儲存和讀取。
#[derive(Debug, Clone)]
pub struct ConstraintManager {
    /// 約束條件列表
    constraints: Vec<Constraint>,
}

impl Default for ConstraintManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintManager {
    /// 建立新的約束條件管理器
    pub fn new() -> Self {
        ConstraintManager {
            constraints: Vec::new(),
        }
    }

    /// 從 XML 內容載入約束條件
    ///
    /// # 格式
    /// ```xml
    /// <constraints>
    ///   <constraint id="..." source="user|gemma" created_at="...">
    ///     <content><![CDATA[...]]></content>
    ///   </constraint>
    /// </constraints>
    /// ```
    pub fn from_xml(xml: &str) -> Self {
        let mut constraints = Vec::new();

        for chunk in xml.split("<constraint ").skip(1) {
            let id = extract_attr(chunk, "id").unwrap_or_else(|| Uuid::new_v4().to_string());
            let source_str = extract_attr(chunk, "source").unwrap_or_default();
            let source = if source_str == "gemma" {
                ConstraintSource::Gemma
            } else {
                ConstraintSource::User
            };
            let created_at_str = extract_attr(chunk, "created_at").unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now());

            let content = chunk
                .split("<content><![CDATA[")
                .nth(1)
                .and_then(|s| s.split("]]></content>").next())
                .unwrap_or("")
                .to_string();

            constraints.push(Constraint {
                id,
                content,
                source,
                created_at,
            });
        }

        ConstraintManager { constraints }
    }

    /// 將約束條件轉換為 XML
    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<constraints>\n");

        for c in &self.constraints {
            xml.push_str(&format!(
                "  <constraint id=\"{}\" source=\"{}\" created_at=\"{}\">\n",
                c.id,
                c.source,
                c.created_at.to_rfc3339()
            ));
            xml.push_str(&format!("    <content><![CDATA[{}]]></content>\n", c.content));
            xml.push_str("  </constraint>\n");
        }

        xml.push_str("</constraints>\n");
        xml
    }

    /// 新增約束條件
    pub fn add(&mut self, constraint: Constraint) {
        // 檢查是否已有相同內容的約束
        if !self.constraints.iter().any(|c| c.content == constraint.content) {
            self.constraints.push(constraint);
        }
    }

    /// 從用戶輸入新增約束
    pub fn add_user_constraint(&mut self, content: impl Into<String>) {
        self.add(Constraint::user(content));
    }

    /// 從 gemma4 回覆萃取約束條件
    ///
    /// 當 gemma4 說出「根據...」或「因為...」時，這些可以成為新的約束。
    ///
    /// # 萃取模式
    /// - 「根據 OO，所以...」 -> 萃取「OO」
    /// - 「因為 XX，所以...」 -> 萃取「XX」
    pub fn extract_from_gemma_response(&mut self, response: &str) -> Vec<String> {
        let mut extracted = Vec::new();

        // 萃取「根據...所以」的模式
        for pattern in &["根據", "因為", "依據", "基於"] {
            if let Some(pos) = response.find(pattern) {
                let start = pos + pattern.len();
                // 找到下一個句號、逗號或換行
                let end = response[start..]
                    .find(|c| c == '。' || c == '，' || c == '\n')
                    .map(|i| start + i)
                    .unwrap_or(response.len());

                let reason = response[start..end].trim().to_string();
                if !reason.is_empty() && reason.len() > 2 {
                    extracted.push(reason);
                }
            }
        }

        // 新增萃取到的約束
        for content in &extracted {
            self.add(Constraint::gemma(content));
        }

        extracted
    }

    /// 取得所有約束條件
    pub fn get_all(&self) -> &[Constraint] {
        &self.constraints
    }

    /// 取得約束條件數量
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// 檢查是否為空
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    /// 格式化為字串列表（用於顯示給 gemma4）
    pub fn format_for_prompt(&self) -> String {
        if self.constraints.is_empty() {
            return "（目前無約束條件）".to_string();
        }

        let mut result = String::from("當前約束條件：\n");
        for (i, c) in self.constraints.iter().enumerate() {
            let source_marker = match c.source {
                ConstraintSource::User => "[用戶]",
                ConstraintSource::Gemma => "[自動]",
            };
            result.push_str(&format!("{}. {} {}\n", i + 1, source_marker, c.content));
        }
        result
    }

    /// 清除所有約束
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.constraints.clear();
    }

    /// 移除指定 ID 的約束
    #[allow(dead_code)]
    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.constraints.len();
        self.constraints.retain(|c| c.id != id);
        self.constraints.len() < initial_len
    }
}

/// 從 XML 屬性字串中提取值
fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    xml.find(&pattern)
        .and_then(|pos| {
            let start = pos + pattern.len();
            xml[start..].find('"').map(|end| xml[start..start + end].to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_new() {
        let c = Constraint::new("測試約束", ConstraintSource::User);
        assert_eq!(c.content, "測試約束");
        assert_eq!(c.source, ConstraintSource::User);
        assert!(!c.id.is_empty());
    }

    #[test]
    fn test_constraint_user() {
        let c = Constraint::user("用戶約束");
        assert_eq!(c.source, ConstraintSource::User);
    }

    #[test]
    fn test_constraint_gemma() {
        let c = Constraint::gemma("gemma 約束");
        assert_eq!(c.source, ConstraintSource::Gemma);
    }

    #[test]
    fn test_constraint_manager_new() {
        let mgr = ConstraintManager::new();
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_add_constraint() {
        let mut mgr = ConstraintManager::new();
        mgr.add(Constraint::user("約束1"));
        assert_eq!(mgr.len(), 1);

        // duplicate should not be added
        mgr.add(Constraint::user("約束1"));
        assert_eq!(mgr.len(), 1);

        mgr.add(Constraint::user("約束2"));
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn test_extract_from_gemma_response() {
        let mut mgr = ConstraintManager::new();

        // 測試「根據...所以」模式
        let response = "根據你的價值觀是誠實，所以我們應該選擇透明的方式。";
        let extracted = mgr.extract_from_gemma_response(response);
        assert!(extracted.contains(&"你的價值觀是誠實".to_string()));

        // 測試「因為...所以」模式
        let response2 = "因為時間有限，所以我們需要優先處理重要的事項。";
        let extracted2 = mgr.extract_from_gemma_response(response2);
        assert!(extracted2.contains(&"時間有限".to_string()));

        // 確認約束已新增
        assert!(mgr.len() >= 2);
    }

    #[test]
    fn test_to_xml() {
        let mut mgr = ConstraintManager::new();
        mgr.add(Constraint::user("約束1"));
        mgr.add(Constraint::gemma("約束2"));

        let xml = mgr.to_xml();
        assert!(xml.contains("<constraints>"));
        assert!(xml.contains("約束1"));
        assert!(xml.contains("約束2"));
        assert!(xml.contains("user"));
        assert!(xml.contains("gemma"));
    }

    #[test]
    fn test_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<constraints>
  <constraint id="test-id-1" source="user" created_at="2026-05-07T10:00:00+08:00">
    <content><![CDATA[用戶約束]]></content>
  </constraint>
  <constraint id="test-id-2" source="gemma" created_at="2026-05-07T11:00:00+08:00">
    <content><![CDATA[gemma約束]]></content>
  </constraint>
</constraints>"#;

        let mgr = ConstraintManager::from_xml(xml);
        assert_eq!(mgr.len(), 2);
        assert_eq!(mgr.get_all()[0].source, ConstraintSource::User);
        assert_eq!(mgr.get_all()[1].source, ConstraintSource::Gemma);
    }

    #[test]
    fn test_format_for_prompt() {
        let mut mgr = ConstraintManager::new();
        assert!(mgr.format_for_prompt().contains("無約束"));

        mgr.add(Constraint::user("約束1"));
        mgr.add(Constraint::gemma("約束2"));

        let formatted = mgr.format_for_prompt();
        assert!(formatted.contains("[用戶]"));
        assert!(formatted.contains("[自動]"));
    }
}
