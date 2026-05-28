//! 收斂引擎（Converge Engine）
//!
//! 負責收斂推理：評估並刪除無效節點。

use crate::engine::budget::ThresholdGate;
use crate::models::{Graph, Node, NodeStatus};

/// 收斂引擎（Converge Engine）
///
/// 負責在推理圖中收斂移除低分節點。
#[derive(Debug, Clone)]
pub struct ConvergeEngine {
    /// 閾值閘門
    threshold_gate: ThresholdGate,
}

impl Default for ConvergeEngine {
    fn default() -> Self {
        ConvergeEngine::new()
    }
}

impl ConvergeEngine {
    /// 建立新的收斂引擎
    ///
    /// # 範例
    /// ```
    /// let engine = ConvergeEngine::new();
    /// ```
    pub fn new() -> Self {
        ConvergeEngine {
            threshold_gate: ThresholdGate::default(),
        }
    }

    /// 建立有自訂閾值的收斂引擎
    pub fn new_with_threshold(threshold: f64) -> Self {
        ConvergeEngine {
            threshold_gate: ThresholdGate::new_with(threshold, 0.6),
        }
    }

    /// 建立有完整自訂參數的收斂引擎
    pub fn new_with_gate(gate: ThresholdGate) -> Self {
        ConvergeEngine {
            threshold_gate: gate,
        }
    }

    /// 收斂：評估並刪除無效節點
    ///
    /// 基本邏輯：
    /// 1. 計算每個節點的分數
    /// 2. 低於閾值的標記為 Pruned
    /// 3. 移除對應的邊
    ///
    /// # 引數
    /// - `graph`: 推理圖
    /// - `threshold`: 分數閾值（可選，若為 None 使用內部的 threshold_gate）
    ///
    /// # 範例
    /// ```
    /// let mut graph = Graph::new();
    /// // ... 加入節點 ...
    ///
    /// let engine = ConvergeEngine::new();
    /// engine.converge(&mut graph, None);
    /// ```
    pub fn converge(&self, graph: &mut Graph, threshold: Option<f64>) -> Vec<String> {
        let threshold = threshold.unwrap_or(self.threshold_gate.threshold);
        let mut pruned_ids = Vec::new();

        // 找出所有低分節點
        let low_score_nodes: Vec<String> = graph
            .nodes
            .values()
            .filter(|n| {
                n.status != NodeStatus::Pruned
                && n.status != NodeStatus::Locked // Locked 的節點不收斂
                && n.score() < threshold
            })
            .map(|n| n.id.clone())
            .collect();

        // 標記為 Pruned
        for node_id in &low_score_nodes {
            graph.prune_node(node_id);
            pruned_ids.push(node_id.clone());
        }

        // 清理孤立的邊（兩端節點都存在的邊不需要刪除，只有當一端被 Pruned 時才刪除）
        // 這在 Graph.remove_node 時已經處理

        pruned_ids
    }

    /// 根據複雜度進行收斂
    ///
    /// 當圖的總複雜度超過閾值時，移除最低分的節點直到在預算內。
    ///
    /// # 引數
    /// - `graph`: 推理圖
    /// - `max_complexity`: 最大複雜度上限
    pub fn converge_by_complexity(&self, graph: &mut Graph, max_complexity: f64) -> Vec<String> {
        let mut pruned_ids = Vec::new();

        while graph.total_complexity() > max_complexity {
            // 找出最低分的節點
            let lowest = graph
                .nodes
                .values()
                .filter(|n| {
                    n.status != NodeStatus::Pruned && n.status != NodeStatus::Locked
                })
                .min_by(|a, b| a.score().partial_cmp(&b.score()).unwrap());

            match lowest {
                Some(node) => {
                    let node_id = node.id.clone();
                    graph.prune_node(&node_id);
                    pruned_ids.push(node_id);
                }
                None => break, // 沒有更多節點可以刪除
            }
        }

        pruned_ids
    }

    /// 根據信心度進行收斂
    ///
    /// 當節點信心度低於閾值時標記為刪除。
    ///
    /// # 引數
    /// - `graph`: 推理圖
    /// - `confidence_threshold`: 信心度閾值（0.0 ~ 1.0）
    pub fn converge_by_confidence(
        &self,
        graph: &mut Graph,
        confidence_threshold: f64,
    ) -> Vec<String> {
        let mut pruned_ids = Vec::new();

        let low_confidence_nodes: Vec<String> = graph
            .nodes
            .values()
            .filter(|n| {
                n.status != NodeStatus::Pruned
                    && n.status != NodeStatus::Locked
                    && n.confidence < confidence_threshold
            })
            .map(|n| n.id.clone())
            .collect();

        for node_id in low_confidence_nodes {
            graph.prune_node(&node_id);
            pruned_ids.push(node_id);
        }

        pruned_ids
    }

    /// 智能收斂：結合複雜度和信心度
    ///
    /// 使用 threshold_gate 的 should_converge 邏輯。
    ///
    /// # 引數
    /// - `graph`: 推理圖
    /// - `current_complexity`: 當前複雜度
    pub fn converge_smart(
        &self,
        graph: &mut Graph,
        current_complexity: f64,
    ) -> Vec<String> {
        let mut pruned_ids = Vec::new();

        for node in graph.nodes.values_mut() {
            if node.status == NodeStatus::Pruned || node.status == NodeStatus::Locked {
                continue;
            }

            if self.threshold_gate.should_converge(current_complexity, node.confidence) {
                node.status = NodeStatus::Pruned;
                pruned_ids.push(node.id.clone());
            }
        }

        pruned_ids
    }

    /// 取得閾值閘門的副本
    pub fn get_threshold_gate(&self) -> ThresholdGate {
        self.threshold_gate.clone()
    }

    /// 設定閾值閘門
    pub fn set_threshold_gate(&mut self, gate: ThresholdGate) {
        self.threshold_gate = gate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Edge, EdgeType};

    fn create_test_graph() -> Graph {
        let mut graph = Graph::new();

        // 建立多個節點，有些高分有些低分
        let node1 = Node::new_with("高分節點".to_string(), 1, "高分節點".to_string(), 0.9, 0.9, 10.0);
        let node2 = Node::new_with("低分節點".to_string(), 1, "低分節點".to_string(), 0.1, 0.1, 5.0);
        let node3 = Node::new_with("中分節點".to_string(), 1, "中分節點".to_string(), 0.5, 0.5, 8.0);

        let n1_id = node1.id.clone();
        let n2_id = node2.id.clone();
        let n3_id = node3.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);

        // 加入一些邊
        let edge = Edge::new(n1_id.clone(), n2_id.clone(), EdgeType::Reasoning);
        graph.add_edge(edge);

        graph
    }

    #[test]
    fn test_converge_engine_new() {
        let engine = ConvergeEngine::new();
        assert_eq!(engine.threshold_gate.threshold, 50.0);
    }

    #[test]
    fn test_converge_basic() {
        let mut graph = create_test_graph();
        let engine = ConvergeEngine::new();

        // threshold = 50.0，低分節點（score = 0.01）會被刪除
        let pruned = engine.converge(&mut graph, Some(0.3));

        assert!(pruned.contains(&graph.get_all_nodes().iter().find(|n| n.content == "低分節點").unwrap().id));
    }

    #[test]
    fn test_converge_by_complexity() {
        let mut graph = create_test_graph();
        let engine = ConvergeEngine::new();

        // 總複雜度 = 10 + 5 + 8 = 23
        assert!((graph.total_complexity() - 23.0).abs() < 0.001);

        // 限制複雜度為 15，應該刪除最低分的節點
        let pruned = engine.converge_by_complexity(&mut graph, 15.0);

        // 會刪除低分節點（score = 0.01）和中分節點（score = 0.25）
        // 直到總複雜度 <= 15
        assert!(!pruned.is_empty());
    }

    #[test]
    fn test_converge_by_confidence() {
        let mut graph = create_test_graph();
        let engine = ConvergeEngine::new();

        // confidence < 0.5 的會被刪除
        let pruned = engine.converge_by_confidence(&mut graph, 0.5);

        // 低分節點（confidence = 0.1）和中分節點（confidence = 0.5，剛好等於閾值不刪除）
        assert_eq!(pruned.len(), 1);
    }

    #[test]
    fn test_converge_smart() {
        let mut graph = create_test_graph();
        let engine = ConvergeEngine::new();

        // 使用低複雜度（不觸發 > threshold），只靠 confidence > 0.8 修剪
        // total complexity = 10 + 5 + 8 = 23
        let pruned = engine.converge_smart(&mut graph, 23.0);

        // 只有 node1 (confidence = 0.9 > 0.8) 會被刪除
        assert_eq!(pruned.len(), 1);
    }

    #[test]
    fn test_converge_does_not_remove_locked() {
        let mut graph = Graph::new();
        let mut node = Node::new_with("鎖定節點".to_string(), 1, "鎖定節點".to_string(), 0.1, 0.1, 1.0);
        node.status = NodeStatus::Locked;
        let node_id = node.id.clone();
        graph.add_node(node);

        let engine = ConvergeEngine::new();
        let pruned = engine.converge(&mut graph, Some(0.5));

        // 鎖定的節點不應該被刪除
        assert_eq!(pruned.len(), 0);
        assert_eq!(graph.get_node(&node_id).unwrap().status, NodeStatus::Locked);
    }

    #[test]
    fn test_converge_empty_graph() {
        let mut graph = Graph::new();
        let engine = ConvergeEngine::new();
        let pruned = engine.converge(&mut graph, Some(0.5));

        assert!(pruned.is_empty());
    }
}
