//! 節點（Node）資料結構
//!
//! Node 是離散思考單位，代表推理圖中的一個思考節點。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 節點狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 草稿（未確認）
    Draft,
    /// 活躍（可編輯）
    Active,
    /// 已刪除（被收斂移除）
    Pruned,
    /// 鎖定（已確認）
    Locked,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Draft
    }
}

/// 節點（Node）
///
/// 代表推理圖中的一個離散思考單位。
/// 每個節點有唯一的 ID、內容、權重、信心度等屬性。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 唯一識別碼（UUID）
    pub id: String,
    /// 步驟編號（1, 2, 3, 4, 5...）
    pub step: i32,
    /// 節點內容描述
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
}

impl Node {
    /// 建立新的草稿節點
    ///
    /// # 引數
    /// - `content`: 節點內容
    /// - `step`: 步驟編號
    ///
    /// # 範例
    /// ```
    /// let node = Node::new("這是我的想法".to_string(), 1);
    /// assert_eq!(node.status, NodeStatus::Draft);
    /// ```
    pub fn new(content: String, step: i32) -> Self {
        Node {
            id: Uuid::new_v4().to_string(),
            step,
            content,
            weight: 1.0,
            confidence: 0.5,
            complexity: 0.0,
            parent_edges: Vec::new(),
            child_edges: Vec::new(),
            status: NodeStatus::Draft,
        }
    }

    /// 建立新節點並指定所有屬性
    pub fn new_with(
        content: String,
        step: i32,
        weight: f64,
        confidence: f64,
        complexity: f64,
    ) -> Self {
        Node {
            id: Uuid::new_v4().to_string(),
            step,
            content,
            weight,
            confidence,
            complexity,
            parent_edges: Vec::new(),
            child_edges: Vec::new(),
            status: NodeStatus::Draft,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_new() {
        let node = Node::new("測試節點".to_string(), 1);
        assert_eq!(node.status, NodeStatus::Draft);
        assert_eq!(node.step, 1);
        assert_eq!(node.content, "測試節點");
        assert!(!node.id.is_empty());
    }

    #[test]
    fn test_node_score() {
        let node = Node::new_with("測試".to_string(), 1, 0.8, 0.7, 1.0);
        // 分數 = weight * confidence = 0.8 * 0.7 = 0.56
        assert!((node.score() - 0.56).abs() < 0.001);
    }

    #[test]
    fn test_node_child_edges() {
        let mut node = Node::new("測試".to_string(), 1);
        node.add_child_edge("edge1".to_string());
        node.add_child_edge("edge2".to_string());
        assert_eq!(node.child_edges.len(), 2);
        node.remove_child_edge("edge1");
        assert_eq!(node.child_edges.len(), 1);
        assert!(node.child_edges.contains(&"edge2".to_string()));
    }

    #[test]
    fn test_node_is_editable() {
        let mut node = Node::new("測試".to_string(), 1);
        assert!(node.is_editable());

        node.status = NodeStatus::Locked;
        assert!(!node.is_editable());

        node.status = NodeStatus::Pruned;
        assert!(!node.is_editable());
    }
}
