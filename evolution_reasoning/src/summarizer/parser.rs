//! NodeUpdateParser - 節點更新解析器
//!
//! 從 gemma4 回應中解析 <node_update> XML 或 JSON 區塊
//! 並自動更新目標節點的欄位

use std::sync::{Arc, Mutex};
use crate::models::{Graph, Node, NodeStatus};
use super::node_update::NodeUpdate;

/// 節點更新解析器
pub struct NodeUpdateParser;

impl NodeUpdateParser {
    /// 從回應文字中解析 NodeUpdate（先 XML，再 JSON）
    pub fn extract(content: &str) -> Option<NodeUpdate> {
        // 先嘗試 XML 格式
        if let Some(update) = Self::extract_xml(content) {
            return Some(update);
        }
        
        // 再嘗試 JSON 格式
        Self::extract_json(content)
    }

    /// 從文字中提取 XML 格式的 <node_update>
    fn extract_xml(content: &str) -> Option<NodeUpdate> {
        // 找 ```xml ... ``` 或直接的 <node_update>...</node_update>
        
        // 先找 ```xml block
        let xml_block = content.find("```xml")
            .or_else(|| content.find("``` xml"));
        
        let xml_content = if let Some(start) = xml_block {
            let after_start = &content[start + 6..];
            let end = after_start.find("```").unwrap_or(after_start.len());
            Some(after_start[..end].trim().to_string())
        } else {
            // 直接找 <node_update>
            content.find("<node_update>").map(|_| {
                content[content.find("<node_update>").unwrap()..].to_string()
            })
        }?;

        NodeUpdate::from_xml(&xml_content)
    }

    /// 從文字中提取 JSON 格式
    fn extract_json(content: &str) -> Option<NodeUpdate> {
        // 找 ```json ... ```
        let json_block = content.find("```json")
            .or_else(|| content.find("``` json"));
        
        let json_content = if let Some(start) = json_block {
            let after_start = &content[start + 7..];
            let end = after_start.find("```").unwrap_or(after_start.len());
            Some(after_start[..end].trim().to_string())
        } else {
            // 找 { "findings": ... } 格式
            let first_brace = content.find('{')?;
            let last_brace = content.rfind('}')?;
            if first_brace < last_brace {
                Some(content[first_brace..=last_brace].to_string())
            } else {
                None
            }
        }?;

        NodeUpdate::from_json(&json_content)
    }

    /// 找到「最活躍」的節點（用於 node_id = None 時）
    pub fn find_active_node(graph: &Graph) -> Option<String> {
        let nodes = graph.get_all_nodes();
        
        // 優先：有 key_findings 的活躍節點
        for node in &nodes {
            if (node.status == NodeStatus::Active || node.status == NodeStatus::Draft) 
                && !node.key_findings.is_empty() {
                return Some(node.id.clone());
            }
        }
        
        // 次之：任何活躍節點
        for node in &nodes {
            if node.status == NodeStatus::Active || node.status == NodeStatus::Draft {
                return Some(node.id.clone());
            }
        }
        
        // 最末：返回第一個節點
        nodes.first().map(|n| n.id.clone())
    }

    /// 應用 NodeUpdate 到圖中的節點
    pub fn apply(graph: &mut Graph, update: &NodeUpdate) -> Result<String, String> {
        // 決定目標節點
        let target_id = if let Some(ref nid) = update.node_id {
            nid.clone()
        } else {
            Self::find_active_node(graph).ok_or("找不到目標節點")?
        };

        // 找到節點
        let node = graph.get_node_mut(&target_id).ok_or("節點不存在")?;

        // 更新 findings（最多5條，超出的話只取前5個）
        let findings: Vec<String> = update.findings.iter()
            .take(5)
            .cloned()
            .collect();
        
        for finding in findings {
            if node.key_findings.len() < 5 && !node.key_findings.contains(&finding) {
                node.key_findings.push(finding);
            }
        }

        // 更新 conclusion
        if let Some(ref conclusion) = update.conclusion {
            if !conclusion.is_empty() {
                node.conclusion = Some(conclusion.clone());
                // 有結論 → 節點鎖定
                node.status = NodeStatus::Locked;
            }
        }

        // 更新 topics
        for topic in &update.topics {
            if !node.relevant_topics.contains(topic) {
                node.relevant_topics.push(topic.clone());
            }
        }

        Ok(format!("節點 {} 更新成功：{} findings, {} topics", 
            target_id, 
            node.key_findings.len(),
            node.relevant_topics.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_xml_block() {
        let content = r#"
        這是 gemma4 的回覆
        
        ```xml
        <node_update node_id="test123">
          <findings>
            <item>發現1</item>
            <item>發現2</item>
          </findings>
          <conclusion null="true"/>
          <topics>
            <topic>旅遊</topic>
          </topics>
        </node_update>
        ```
        
        以上就是我的分析。
        "#;
        let update = NodeUpdateParser::extract(content).unwrap();
        assert_eq!(update.node_id, Some("test123".to_string()));
        assert_eq!(update.findings.len(), 2);
        assert_eq!(update.topics.len(), 1);
    }

    #[test]
    fn test_extract_json_block() {
        let content = r#"
        分析結果：
        
        ```json
        {"findings":["發現A","發現B"],"conclusion":"這是結論","topics":["主題1"]}
        ```
        "#;
        let update = NodeUpdateParser::extract(content).unwrap();
        assert_eq!(update.findings.len(), 2);
        assert_eq!(update.conclusion, Some("這是結論".to_string()));
    }

    #[test]
    fn test_extract_direct_xml() {
        let content = "有些回覆直接帶 <node_update><findings><item>test</item></findings></node_update> 在文字裡";
        let update = NodeUpdateParser::extract(content).unwrap();
        assert_eq!(update.findings.len(), 1);
    }
}