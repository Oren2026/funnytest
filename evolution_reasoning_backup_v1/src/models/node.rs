//! 節點（Node）資料結構
//!
//! Node 是離散思考單位，代表推理圖中的一個認知節點。
//! 每個節點有固定結構，不因層數深度而被二次壓縮。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 節點狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 草稿（分析中）
    Draft,
    /// 活躍（已確認）
    Active,
    /// 已刪除（被收斂移除）
    Pruned,
    /// 鎖定（已得出結論）
    Locked,
    /// 執行失敗
    Failed,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Draft
    }
}

/// 節點（Node）
///
/// 代表推理圖中的一個離散認知單元。
///
/// # 固定欄位（不壓縮）
///
/// 每個節點的資訊不會因為深度而被二次壓縮。
/// 每次送給 gemma4 的都是完整的節點資訊。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 唯一識別碼（UUID）
    pub id: String,
    /// 步驟編號（1, 2, 3, 4, 5...）
    pub step: i32,
    /// 這個節點要回答的核心問題
    pub question: String,
    /// 重要發現（不超過 5 條，避免過度複雜）
    pub key_findings: Vec<String>,
    /// 最終結論（決策類節點才有）
    pub conclusion: Option<String>,
    /// 關聯主題（用於快速檢索相關節點）
    pub relevant_topics: Vec<String>,
    /// 完整描述（這個節點的「為什麼」）
    pub content: String,
    /// 權重（影響上游）
    pub weight: f64,
    /// AI 信心度（0.0 ~ 1.0）
    pub confidence: f64,
    /// 該節點的複雜度貢獻
    pub complexity: f64,
    /// 連入的邊 ID
    pub parent_edges: Vec<String>,
    /// 連出的邊 ID
    pub child_edges: Vec<String>,
    /// 狀態
    pub status: NodeStatus,
    /// 執行回饋結果
    pub feedback_result: Option<String>,
    /// 執行回饋時間戳
    pub feedback_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Node {
    /// 建立新的認知節點
    ///
    /// # 引數
    /// - `question`: 這個節點要回答的問題
    /// - `step`: 步驟編號
    ///
    pub fn new(question: String, step: i32) -> Self {
        Node {
            id: Uuid::new_v4().to_string(),
            step,
            question: question.clone(),
            key_findings: Vec::new(),
            conclusion: None,
            relevant_topics: Vec::new(),
            content: question,
            weight: 1.0,
            confidence: 0.5,
            complexity: 0.0,
            parent_edges: Vec::new(),
            child_edges: Vec::new(),
            status: NodeStatus::Draft,
            feedback_result: None,
            feedback_timestamp: None,
        }
    }

    /// 建立新節點並指定所有屬性
    pub fn new_with(
        question: String,
        step: i32,
        content: String,
        weight: f64,
        confidence: f64,
        complexity: f64,
    ) -> Self {
        Node {
            id: Uuid::new_v4().to_string(),
            step,
            question,
            key_findings: Vec::new(),
            conclusion: None,
            relevant_topics: Vec::new(),
            content,
            weight,
            confidence,
            complexity,
            parent_edges: Vec::new(),
            child_edges: Vec::new(),
            status: NodeStatus::Draft,
            feedback_result: None,
            feedback_timestamp: None,
        }
    }

    /// 建立根節點（第一個節點，承載初始任務）
    pub fn root(question: String) -> Self {
        let content = format!("初始任務：{}", question);
        Node {
            id: Uuid::new_v4().to_string(),
            step: 1,
            question: question.clone(),
            key_findings: Vec::new(),
            conclusion: None,
            relevant_topics: Vec::new(),
            content,
            weight: 1.0,
            confidence: 1.0,
            complexity: 0.0,
            parent_edges: Vec::new(),
            child_edges: Vec::new(),
            status: NodeStatus::Active,
            feedback_result: None,
            feedback_timestamp: None,
        }
    }

    /// 新增一條重要發現（最多 5 條）
    pub fn add_finding(&mut self, finding: String) {
        if self.key_findings.len() < 5 && !self.key_findings.contains(&finding) {
            self.key_findings.push(finding);
        }
    }

    /// 設定結論（會將節點狀態改為 Locked）
    pub fn set_conclusion(&mut self, conclusion: String) {
        self.conclusion = Some(conclusion);
        self.status = NodeStatus::Locked;
    }

    /// 新增關聯主題
    pub fn add_topic(&mut self, topic: String) {
        if !self.relevant_topics.contains(&topic) {
            self.relevant_topics.push(topic);
        }
    }

    /// 設定執行回饋結果
    pub fn set_feedback(&mut self, result: String) {
        self.feedback_result = Some(result);
        self.feedback_timestamp = Some(chrono::Utc::now());
    }

    /// 根據執行結果設定節點狀態
    pub fn apply_execution_result(&mut self, success: bool) {
        if !success {
            self.status = NodeStatus::Failed;
        }
    }

    /// 加入子節點的邊 ID
    pub fn add_child_edge(&mut self, edge_id: String) {
        if !self.child_edges.contains(&edge_id) {
            self.child_edges.push(edge_id);
        }
    }

    /// 加入父節點的邊 ID
    pub fn add_parent_edge(&mut self, edge_id: String) {
        if !self.parent_edges.contains(&edge_id) {
            self.parent_edges.push(edge_id);
        }
    }

    /// 移除子節點的邊 ID
    pub fn remove_child_edge(&mut self, edge_id: &str) {
        self.child_edges.retain(|e| e != edge_id);
    }

    /// 移除父節點的邊 ID
    pub fn remove_parent_edge(&mut self, edge_id: &str) {
        self.parent_edges.retain(|e| e != edge_id);
    }

    /// 檢查節點是否可編輯
    pub fn is_editable(&self) -> bool {
        self.status == NodeStatus::Draft || self.status == NodeStatus::Active
    }

    /// 檢查節點是否已刪除
    pub fn is_pruned(&self) -> bool {
        self.status == NodeStatus::Pruned
    }

    /// 計算節點分數（用於收斂判斷）
    /// 分數 = weight * confidence
    pub fn score(&self) -> f64 {
        self.weight * self.confidence
    }

    /// 取得節點的簡要摘要（用於 System Prompt）
    /// 格式固定，每次都是完整資訊，不因深度而壓縮
    pub fn to_prompt_summary(&self) -> String {
        let mut lines = vec![
            format!("## 節點 [{}]", self.id),
            format!("問題：{}", self.question),
        ];

        if !self.key_findings.is_empty() {
            lines.push("發現：".to_string());
            for (i, f) in self.key_findings.iter().enumerate() {
                lines.push(format!("  {}. {}", i + 1, f));
            }
        }

        if let Some(ref c) = self.conclusion {
            lines.push(format!("結論：{}", c));
        }

        if !self.relevant_topics.is_empty() {
            lines.push(format!("主題：{}", self.relevant_topics.join(", ")));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_new() {
        let node = Node::new("金門5天行程如何規劃？".to_string(), 1);
        assert_eq!(node.status, NodeStatus::Draft);
        assert_eq!(node.step, 1);
        assert_eq!(node.question, "金門5天行程如何規劃？");
        assert!(node.key_findings.is_empty());
        assert!(node.conclusion.is_none());
    }

    #[test]
    fn test_node_root() {
        let node = Node::root("測試任務".to_string());
        assert_eq!(node.status, NodeStatus::Active);
        assert_eq!(node.step, 1);
        assert!(node.content.contains("測試任務"));
    }

    #[test]
    fn test_add_finding() {
        let mut node = Node::new("問題".to_string(), 1);
        node.add_finding("發現1".to_string());
        node.add_finding("發現2".to_string());
        assert_eq!(node.key_findings.len(), 2);
        // 測試不重複
        node.add_finding("發現1".to_string());
        assert_eq!(node.key_findings.len(), 2);
    }

    #[test]
    fn test_set_conclusion() {
        let mut node = Node::new("問題".to_string(), 1);
        node.set_conclusion("這是結論".to_string());
        assert!(node.conclusion.is_some());
        assert_eq!(node.status, NodeStatus::Locked);
    }

    #[test]
    fn test_add_topic() {
        let mut node = Node::new("問題".to_string(), 1);
        node.add_topic("旅行".to_string());
        node.add_topic("金門".to_string());
        assert_eq!(node.relevant_topics.len(), 2);
        // 不重複
        node.add_topic("旅行".to_string());
        assert_eq!(node.relevant_topics.len(), 2);
    }

    #[test]
    fn test_to_prompt_summary() {
        let mut node = Node::new("金門行程規劃".to_string(), 1);
        node.add_finding("用戶喜歡歷史古蹟".to_string());
        node.add_finding("偏好深度遊".to_string());
        node.set_conclusion("選擇時間軸深度遊".to_string());
        node.add_topic("金門".to_string());
        node.add_topic("歷史".to_string());

        let summary = node.to_prompt_summary();
        assert!(summary.contains("問題：金門行程規劃"));
        assert!(summary.contains("用戶喜歡歷史古蹟"));
        assert!(summary.contains("結論：選擇時間軸深度遊"));
        assert!(summary.contains("金門, 歷史"));
    }

    #[test]
    fn test_node_score() {
        let node = Node::new_with(
            "問題".to_string(),
            1,
            "內容".to_string(),
            0.8,
            0.7,
            1.0,
        );
        // 分數 = weight * confidence = 0.8 * 0.7 = 0.56
        assert!((node.score() - 0.56).abs() < 0.001);
    }

    #[test]
    fn test_node_child_edges() {
        let mut node = Node::new("問題".to_string(), 1);
        node.add_child_edge("edge1".to_string());
        node.add_child_edge("edge2".to_string());
        assert_eq!(node.child_edges.len(), 2);
        node.remove_child_edge("edge1");
        assert_eq!(node.child_edges.len(), 1);
        assert!(node.child_edges.contains(&"edge2".to_string()));
    }

    #[test]
    fn test_node_is_editable() {
        let mut node = Node::new("問題".to_string(), 1);
        assert!(node.is_editable());

        node.status = NodeStatus::Locked;
        assert!(!node.is_editable());

        node.status = NodeStatus::Pruned;
        assert!(!node.is_editable());
    }
}