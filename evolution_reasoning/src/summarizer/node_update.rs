//! NodeUpdate - 節點更新結構
//!
//! 用於解析 gemma4 回應中的 <node_update> 結構

use serde::{Deserialize, Serialize};

/// 節點更新資料結構（從 XML 解析）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeUpdate {
    /// 目標節點 ID（None 表示當前活躍節點）
    pub node_id: Option<String>,
    /// 重要發現（最多5條）
    pub findings: Vec<String>,
    /// 最終結論（None 表示尚未形成結論）
    pub conclusion: Option<String>,
    /// 相關主題
    pub topics: Vec<String>,
}

impl NodeUpdate {
    /// 空值判斷
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty() && self.conclusion.is_none() && self.topics.is_empty()
    }

    /// 是否有效（有實質內容）
    pub fn has_content(&self) -> bool {
        !self.findings.is_empty() || self.conclusion.is_some()
    }

    /// 從 XML 字串解析（完整 <node_update>...</node_update> 區塊）
    pub fn from_xml(xml_str: &str) -> Option<Self> {
        let trimmed = xml_str.trim();
        
        // 移除 code block 標記（如果有）
        let content = trimmed
            .strip_prefix("```xml").unwrap_or(trimmed)
            .strip_prefix("```").unwrap_or(trimmed)
            .trim();
        
        // 找 <node_update ...> ... </node_update>
        let start_tag = content.find("<node_update")?;
        let after_tag = &content[start_tag..];
        
        // 找第一個 >
        let close_bracket_pos = after_tag.find('>')?;
        let after_open = &after_tag[close_bracket_pos + 1..];
        
        // 找 </node_update>
        let end_tag_pos = after_open.find("</node_update>")?;
        let inner = &after_open[..end_tag_pos];
        
        let mut node_id = None;
        let mut findings = Vec::new();
        let mut conclusion = None;
        let mut topics = Vec::new();
        
        // 解析 node_id 屬性
        let open_tag = &after_tag[..close_bracket_pos + 1];
        if let Some(start) = open_tag.find("node_id=\"") {
            let after = &open_tag[start + 9..];
            if let Some(end) = after.find('"') {
                node_id = Some(after[..end].to_string());
            }
        }
        
        // 解析 <findings>...</findings>
        if let Some(f_start) = inner.find("<findings>") {
            let f_after = &inner[f_start + 10..];
            if let Some(f_end) = f_after.find("</findings>") {
                let findings_inner = &f_after[..f_end];
                let mut pos = 0;
                while let Some(item_start) = findings_inner[pos..].find("<item>") {
                    let after = &findings_inner[pos + item_start + 6..];
                    if let Some(item_end) = after.find("</item>") {
                        let text = after[..item_end].trim().to_string();
                        if !text.is_empty() {
                            findings.push(text);
                        }
                        pos += item_start + 6 + item_end;
                    } else {
                        break;
                    }
                }
            }
        }
        
        // 解析 <conclusion>...</conclusion> 或 <conclusion null="true"/>
        if let Some(c_start) = inner.find("<conclusion") {
            let c_after = &inner[c_start..];
            if c_after.starts_with("<conclusion null=\"true\"/>") || c_after.starts_with("<conclusion null='true'/>") {
                conclusion = None;
            } else if let Some(c_close) = c_after.find('>') {
                let c_inner = &c_after[c_close + 1..];
                if let Some(c_end) = c_inner.find("</conclusion>") {
                    let text = c_inner[..c_end].trim().to_string();
                    if !text.is_empty() {
                        conclusion = Some(text);
                    }
                }
            }
        }
        
        // 解析 <topics>...</topics>
        if let Some(t_start) = inner.find("<topics>") {
            let t_after = &inner[t_start + 8..];
            if let Some(t_end) = t_after.find("</topics>") {
                let topics_inner = &t_after[..t_end];
                let mut pos = 0;
                while let Some(topic_start) = topics_inner[pos..].find("<topic>") {
                    let after = &topics_inner[pos + topic_start + 7..];
                    if let Some(topic_end) = after.find("</topic>") {
                        let text = after[..topic_end].trim().to_string();
                        if !text.is_empty() {
                            topics.push(text);
                        }
                        pos += topic_start + 7 + topic_end;
                    } else {
                        break;
                    }
                }
            }
        }
        
        Some(NodeUpdate {
            node_id,
            findings,
            conclusion,
            topics,
        })
    }

    /// 從 JSON 字串解析
    pub fn from_json(json_str: &str) -> Option<Self> {
        let trimmed = json_str.trim();
        
        // 移除 ```json 包裝
        let content = trimmed
            .strip_prefix("```json").unwrap_or(trimmed)
            .strip_prefix("```").unwrap_or(trimmed)
            .strip_suffix("```").unwrap_or(trimmed)
            .trim();

        let json: serde_json::Value = serde_json::from_str(content).ok()?;

        let node_id = json.get("node_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let findings = json.get("findings")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let conclusion = json.get("conclusion")
            .and_then(|v| {
                if v.is_null() { None }
                else { v.as_str().map(|s| s.to_string()) }
            });

        let topics = json.get("topics")
            .or_else(|| json.get("relevant_topics"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Some(NodeUpdate {
            node_id,
            findings,
            conclusion,
            topics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_xml() {
        let xml = r#"<node_update node_id="abc123">
          <findings>
            <item>用戶偏好：悠閒、戰地歷史、在地美食</item>
            <item>時間：5天4夜</item>
          </findings>
          <conclusion null="true"/>
          <topics>
            <topic>金門</topic>
            <topic>戰地歷史</topic>
          </topics>
        </node_update>"#;
        let update = NodeUpdate::from_xml(xml).unwrap();
        assert_eq!(update.node_id, Some("abc123".to_string()));
        assert_eq!(update.findings.len(), 2);
        assert!(update.conclusion.is_none());
        assert_eq!(update.topics.len(), 2);
    }

    #[test]
    fn test_parse_empty_conclusion() {
        let xml = r#"<node_update><findings><item>發現1</item></findings><conclusion null="true"/></node_update>"#;
        let update = NodeUpdate::from_xml(xml).unwrap();
        assert!(update.conclusion.is_none());
    }

    #[test]
    fn test_parse_with_conclusion() {
        let xml = r#"<node_update><findings/><conclusion>這是最好的方案</conclusion></node_update>"#;
        let update = NodeUpdate::from_xml(xml).unwrap();
        assert_eq!(update.conclusion, Some("這是最好的方案".to_string()));
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{"findings":["發現1","發現2"],"conclusion":null,"topics":["金門","美食"]}"#;
        let update = NodeUpdate::from_json(json).unwrap();
        assert_eq!(update.findings.len(), 2);
        assert!(update.conclusion.is_none());
        assert_eq!(update.topics.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let empty = NodeUpdate::default();
        assert!(empty.is_empty());

        let with_finding = NodeUpdate {
            node_id: None,
            findings: vec!["test".to_string()],
            conclusion: None,
            topics: vec![],
        };
        assert!(with_finding.has_content());
    }
}