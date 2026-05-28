//! 邊（Edge）資料結構
//!
//! Edge 連接兩個節點，代表節點之間的關係。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 邊的類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    /// 推理關係
    Reasoning,
    /// 約束關係
    Constraint,
    /// 分叉關係
    Divergence,
}

impl Default for EdgeType {
    fn default() -> Self {
        EdgeType::Reasoning
    }
}

/// 邊（Edge）
///
/// 連接兩個節點，代表它們之間的關係。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// 唯一識別碼（UUID）
    pub id: String,
    /// 起始節點 ID
    pub from: String,
    /// 目標節點 ID
    pub to: String,
    /// 邊的類型
    pub edge_type: EdgeType,
    /// 權重
    pub weight: f64,
}

impl Edge {
    /// 建立新的邊
    ///
    /// # 引數
    /// - `from`: 起始節點 ID
    /// - `to`: 目標節點 ID
    /// - `edge_type`: 邊的類型
    ///
    /// # 範例
    /// ```
    /// let edge = Edge::new("node1".to_string(), "node2".to_string(), EdgeType::Reasoning);
    /// ```
    pub fn new(from: String, to: String, edge_type: EdgeType) -> Self {
        Edge {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            edge_type,
            weight: 1.0,
        }
    }

    /// 建立新的邊並指定權重
    pub fn new_with_weight(from: String, to: String, edge_type: EdgeType, weight: f64) -> Self {
        Edge {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            edge_type,
            weight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new() {
        let edge = Edge::new("node1".to_string(), "node2".to_string(), EdgeType::Reasoning);
        assert_eq!(edge.from, "node1");
        assert_eq!(edge.to, "node2");
        assert_eq!(edge.edge_type, EdgeType::Reasoning);
        assert_eq!(edge.weight, 1.0);
        assert!(!edge.id.is_empty());
    }

    #[test]
    fn test_edge_with_weight() {
        let edge = Edge::new_with_weight(
            "node1".to_string(),
            "node2".to_string(),
            EdgeType::Divergence,
            0.5,
        );
        assert_eq!(edge.weight, 0.5);
        assert_eq!(edge.edge_type, EdgeType::Divergence);
    }

    #[test]
    fn test_edge_type_default() {
        let edge_type = EdgeType::default();
        assert_eq!(edge_type, EdgeType::Reasoning);
    }
}
