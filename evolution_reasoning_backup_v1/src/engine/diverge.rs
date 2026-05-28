//! 發散引擎（Diverge Engine）
//!
//! 負責發散推理：針對一個節點生成多個可能的子節點。

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use uuid::Uuid;

use crate::models::{Edge, EdgeType, Graph, Node, NodeStatus};

/// 發散引擎（Diverge Engine）
///
/// 負責在推理圖中發散生成新的分支節點。
#[derive(Debug, Clone)]
pub struct DivergeEngine {
    /// 隨機種子（可選）
    seed: Option<u64>,
}

impl Default for DivergeEngine {
    fn default() -> Self {
        DivergeEngine::new()
    }
}

impl DivergeEngine {
    /// 建立新的發散引擎
    ///
    /// # 範例
    /// ```
    /// let engine = DivergeEngine::new();
    /// ```
    pub fn new() -> Self {
        DivergeEngine { seed: None }
    }

    /// 建立有指定種子的發散引擎
    pub fn new_with_seed(seed: u64) -> Self {
        DivergeEngine { seed: Some(seed) }
    }

    /// 發散：針對一個節點生成多個可能的子節點
    ///
    /// 基本邏輯：
    /// 1. 根據隨機種子生成多個分支
    /// 2. 每個分支是一個新的草稿節點
    /// 3. 權重從父節點繼承（略微衰減）
    ///
    /// # 引數
    /// - `graph`: 推理圖
    /// - `node_id`: 父節點 ID
    /// - `count`: 要生成的子節點數量
    /// - `contents`: 每個子節點的內容（可選，若空則使用預設內容）
    ///
    /// # 範例
    /// ```
    /// let mut graph = Graph::new();
    /// let parent = Node::new("父親節點".to_string(), 1);
    /// let parent_id = parent.id.clone();
    /// graph.add_node(parent);
    ///
    /// let engine = DivergeEngine::new();
    /// let children = engine.diverge(&mut graph, &parent_id, 3, None);
    /// assert_eq!(children.len(), 3);
    /// ```
    pub fn diverge(
        &self,
        graph: &mut Graph,
        node_id: &str,
        count: i32,
        contents: Option<Vec<String>>,
    ) -> Vec<Node> {
        // 先取得需要的父節點資訊，clone 避免 borrow 問題
        let (parent_weight, parent_confidence, parent_step, parent_content) = {
            let parent = match graph.get_node(node_id) {
                Some(n) => n,
                None => return Vec::new(),
            };

            // 父節點狀態檢查：只有 Draft 或 Active 可以發散
            if parent.status != NodeStatus::Draft && parent.status != NodeStatus::Active {
                return Vec::new();
            }

            (parent.weight, parent.confidence, parent.step, parent.content.clone())
        };

        let mut rng = self.get_rng();
        let next_step = parent_step + 1;
        let mut results = Vec::new();

        for i in 0..count {
            // 計算子節點內容
            let content = if let Some(ref c) = contents {
                if (i as usize) < c.len() {
                    c[i as usize].clone()
                } else {
                    format!("發散分支 {}", i + 1)
                }
            } else {
                // 根據不同分支給予不同的預設內容
                match i {
                    0 => format!("探索方向 A: 基於「{}」的延伸思考", parent_content),
                    1 => format!("探索方向 B: 反面假設「{}」", parent_content),
                    _ => format!("探索方向 {}: 細節深化", i + 1),
                }
            };

            // 建立子節點
            // 權重從父節點略微衰減（加上隨機性）
            let weight_factor = 0.7 + rng.gen_range(0.0..0.3);
            let child_weight = parent_weight * weight_factor;

            // 信心度略微下降
            let child_confidence = (parent_confidence * 0.9).max(0.1);

let child = Node::new_with(
                content.clone(),
                next_step,
                content.clone(),  // content = question for divergence nodes
                child_weight,
                child_confidence,
                0.0, // complexity initial
            );

            let child_id = child.id.clone();

            // 建立從父節點到子節點的邊
            let edge = Edge::new_with_weight(
                node_id.to_string(),
                child_id.clone(),
                EdgeType::Divergence,
                child_weight,
            );

            // 加入圖中（會自動維護 parent_edges 和 child_edges）
            graph.add_node(child);
            graph.add_edge(edge);

            // 取出剛才加入的節點（做為回傳值）
            if let Some(added_node) = graph.get_node(&child_id) {
                results.push(added_node.clone());
            }
        }

        results
    }

    /// 針對一個節點發散，但不加入圖中（純計算用）
    ///
    /// # 引數
    /// - `node`: 父節點
    /// - `count`: 要生成的子節點數量
    pub fn diverge_only(&self, node: &Node, count: i32) -> Vec<Node> {
        let mut rng = self.get_rng();
        let parent_weight = node.weight;
        let next_step = node.step + 1;
        let mut results = Vec::new();

        for i in 0..count {
            let content = format!("發散分支 {}（待加入圖中）", i + 1);

            let weight_factor = 0.7 + rng.gen_range(0.0..0.3);
            let child_weight = parent_weight * weight_factor;
            let child_confidence = (node.confidence * 0.9).max(0.1);

            let child = Node::new_with(content.clone(), next_step, content.clone(), child_weight, child_confidence, 0.0);
            results.push(child);
        }

        results
    }

    /// 取得隨機數生成器
    fn get_rng(&self) -> StdRng {
        if let Some(seed) = self.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diverge_engine_new() {
        let engine = DivergeEngine::new();
        assert!(engine.seed.is_none());
    }

    #[test]
    fn test_diverge_engine_new_with_seed() {
        let engine = DivergeEngine::new_with_seed(42);
        assert!(engine.seed.is_some());
        assert_eq!(engine.seed.unwrap(), 42);
    }

    #[test]
    fn test_diverge_basic() {
        let mut graph = Graph::new();
        let parent = Node::new("父親節點".to_string(), 1);
        let parent_id = parent.id.clone();
        graph.add_node(parent);

        let engine = DivergeEngine::new();
        let children = engine.diverge(&mut graph, &parent_id, 3, None);

        assert_eq!(children.len(), 3);
        assert_eq!(graph.node_count(), 4); // 1 父 + 3 子
        assert_eq!(graph.edge_count(), 3); // 3 條邊
    }

    #[test]
    fn test_diverge_with_custom_contents() {
        let mut graph = Graph::new();
        let parent = Node::new("父親節點".to_string(), 1);
        let parent_id = parent.id.clone();
        graph.add_node(parent);

        let engine = DivergeEngine::new();
        let contents = vec!["自訂內容 A".to_string(), "自訂內容 B".to_string()];
        let children = engine.diverge(&mut graph, &parent_id, 2, Some(contents));

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].content, "自訂內容 A");
        assert_eq!(children[1].content, "自訂內容 B");
    }

    #[test]
    fn test_diverge_inherits_weight() {
        let mut graph = Graph::new();
        let parent = Node::new_with("父親".to_string(), 1, "父親".to_string(), 0.8, 0.9, 0.0);
        let parent_id = parent.id.clone();
        graph.add_node(parent);

        let engine = DivergeEngine::new();
        let children = engine.diverge(&mut graph, &parent_id, 2, None);

        // 子節點權重應該比父節點低
        for child in &children {
            assert!(child.weight < 0.8);
        }
    }

    #[test]
    fn test_diverge_pruned_node() {
        let mut graph = Graph::new();
        let mut parent = Node::new("父親節點".to_string(), 1);
        parent.status = NodeStatus::Pruned;
        let parent_id = parent.id.clone();
        graph.add_node(parent);

        let engine = DivergeEngine::new();
        let children = engine.diverge(&mut graph, &parent_id, 3, None);

        // 已刪除的節點不能發散
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_diverge_nonexistent_node() {
        let mut graph = Graph::new();
        let engine = DivergeEngine::new();
        let children = engine.diverge(&mut graph, "不存在的ID", 3, None);

        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_diverge_only() {
        let parent = Node::new_with("父親".to_string(), 1, "父親".to_string(), 0.8, 0.9, 0.0);
        let engine = DivergeEngine::new_with_seed(123);
        let children = engine.diverge_only(&parent, 3);

        // 不加入圖中，所以圖仍然是空的
        assert_eq!(children.len(), 3);
        // children[0] 是新建立的 Node，UUID 應該不同於 parent
        assert_ne!(children[0].id, parent.id);
    }
}
