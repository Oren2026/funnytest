//! 決策樹生成器（Decision Tree Generator）
//!
//! v0.7 新增：根據圖結構生成結構化的決策樹輸出。
//!
//! # 輸出格式
//!
//! ```ignore
//! ## 決策樹：{主題}
//!
//! ### 方向 A：{方向名稱}
//! - {子節點1}
//! - {子節點2}
//!
//! ### 方向 B：{方向名稱}
//! - {子節點1}
//! - {子節點2}
//!
//! ### 建議優先順序
//! 1. 優先：{最高分節點}
//! 2. 其次：{次高分節點}
//! ...
//! ```

use crate::models::{Graph, Node, NodeStatus};

/// 決策樹節點
#[derive(Debug, Clone)]
pub struct DecisionNode {
    /// 節點 ID
    pub id: String,
    /// 內容
    pub content: String,
    /// 分數
    pub score: f64,
    /// 信心度
    pub confidence: f64,
    /// 是否鎖定
    pub is_locked: bool,
    /// 子節點
    pub children: Vec<DecisionNode>,
}

impl DecisionNode {
    /// 從 Graph 節點建立 DecisionNode
    pub fn from_node(graph: &Graph, node: &Node) -> Self {
        let children: Vec<DecisionNode> = graph
            .get_children(&node.id)
            .iter()
            .filter(|c| c.status != NodeStatus::Pruned)
            .map(|c| DecisionNode::from_node(graph, c))
            .collect();

        DecisionNode {
            id: node.id.clone(),
            content: node.content.clone(),
            score: node.score(),
            confidence: node.confidence,
            is_locked: node.status == NodeStatus::Locked,
            children,
        }
    }

    /// 計算節點的深度
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// 計算節點總數
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }
}

/// 決策樹
#[derive(Debug, Clone)]
pub struct DecisionTree {
    /// 主題名稱
    pub topic: String,
    /// 根節點
    pub root: DecisionNode,
    /// 總節點數
    pub total_nodes: usize,
}

impl DecisionTree {
    /// 從圖建立決策樹
    pub fn from_graph(graph: &Graph, topic: &str) -> Option<Self> {
        let roots = graph.get_root_nodes();
        if roots.is_empty() {
            return None;
        }

        // 如果有多個根，選擇 current_topic 的根
        let root_node = if let Some(current_topic) = graph.get_current_topic() {
            roots.iter()
                .find(|n| n.id == current_topic.root_node_id)
                .or_else(|| roots.first())
                .copied()
        } else {
            roots.first().copied()
        }?;

        let decision_root = DecisionNode::from_node(graph, root_node);
        let total_nodes = decision_root.count();

        Some(DecisionTree {
            topic: topic.to_string(),
            root: decision_root,
            total_nodes,
        })
    }

    /// 生成 Markdown 格式的決策樹
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("## 決策樹：{}\n\n", self.topic));

        // 按方向分組顯示（第一層子節點）
        let directions = &self.root.children;
        if directions.is_empty() {
            output.push_str(&format!("- {} (分數:{:.2}, 信心:{:.2})\n",
                self.root.content, self.root.score, self.root.confidence));
        } else {
            for (i, dir) in directions.iter().enumerate() {
                let label = (b'A' + i as u8) as char;
                output.push_str(&format!("### 方向 {}：{}\n", label, dir.content));

                if dir.children.is_empty() {
                    output.push_str(&format!("- 分數:{:.2}, 信心:{:.2}\n", dir.score, dir.confidence));
                } else {
                    for child in &dir.children {
                        output.push_str(&format!("- {} (分數:{:.2}, 信心:{:.2})\n",
                            child.content, child.score, child.confidence));
                    }
                }
                output.push('\n');
            }
        }

        // 建議優先順序
        output.push_str("### 建議優先順序\n");
        let mut priorities = self.collect_priorities();
        priorities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        for (i, p) in priorities.iter().take(5).enumerate() {
            let rank = match i {
                0 => "優先",
                1 => "其次",
                2 => "第三",
                _ => "建議",
            };
            output.push_str(&format!("{}. {}：{} (分數:{:.2})\n",
                i + 1, rank, p.content, p.score));
        }

        output.push_str(&format!("\n---\n*總節點數：{}*\n", self.total_nodes));

        output
    }

    /// 收集所有可比較的節點
    fn collect_priorities(&self) -> Vec<&DecisionNode> {
        let mut nodes = Vec::new();
        self.collect_nodes_recursive(&self.root, &mut nodes);
        nodes
    }

    fn collect_nodes_recursive<'a>(&self, node: &'a DecisionNode, result: &mut Vec<&'a DecisionNode>) {
        // 只加入葉節點或鎖定的節點
        if node.children.is_empty() || node.is_locked {
            result.push(node);
        }
        for child in &node.children {
            self.collect_nodes_recursive(child, result);
        }
    }
}

/// 為整個圖生成決策樹報告（包含所有主題）
pub fn generate_full_report(graph: &Graph, topics: &[(&str, &str)]) -> String {
    // topics 是 (topic_id, topic_title) 的列表
    let mut report = String::new();

    report.push_str("# 推理決策樹報告\n\n");
    report.push_str(&format!("生成時間：{}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    for (topic_id, topic_title) in topics {
        // 找到該主題的根節點
        if let Some(topic) = graph.topics.get(*topic_id) {
            if let Some(root_node) = graph.get_node(&topic.root_node_id) {
                let decision_root = DecisionNode::from_node(graph, root_node);
                let tree = DecisionTree {
                    topic: (*topic_title).to_string(),
                    root: decision_root,
                    total_nodes: graph.count_topic_nodes(topic_id),
                };
                report.push_str(&tree.to_markdown());
                report.push('\n');
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Edge;
    use crate::models::EdgeType;

    #[test]
    fn test_decision_node_from_node() {
        let mut graph = Graph::new();
        let root = Node::new_with("根節點".to_string(), 1, "根節點".to_string(), 0.8, 0.9, 1.0);
        let root_id = root.id.clone();
        graph.add_node(root);

        let child = Node::new_with("子節點".to_string(), 2, "子節點".to_string(), 0.7, 0.8, 1.0);
        let child_id = child.id.clone();
        graph.add_node(child);

        let edge = Edge::new(root_id.clone(), child_id, EdgeType::Reasoning);
        graph.add_edge(edge);

        let node = graph.get_node(&root_id).unwrap();
        let decision_node = DecisionNode::from_node(&graph, node);

        assert_eq!(decision_node.content, "根節點");
        assert!((decision_node.score - 0.72).abs() < 0.001); // 0.8 * 0.9
        assert_eq!(decision_node.children.len(), 1);
        assert_eq!(decision_node.children[0].content, "子節點");
    }

    #[test]
    fn test_decision_tree_from_graph() {
        let mut graph = Graph::new();
        let root = Node::new_with("測試主題".to_string(), 1, "測試主題".to_string(), 0.8, 0.9, 1.0);
        graph.add_node(root);

        let tree = DecisionTree::from_graph(&graph, "測試");
        assert!(tree.is_some());

        let tree = tree.unwrap();
        assert_eq!(tree.topic, "測試");
        assert_eq!(tree.root.content, "測試主題");
    }

    #[test]
    fn test_decision_tree_to_markdown() {
        let mut graph = Graph::new();
        let root = Node::new_with("睡眠改善".to_string(), 1, "睡眠改善".to_string(), 0.8, 0.9, 1.0);
        let root_id = root.id.clone();
        graph.add_node(root);

        // 加入方向 A
        let dir_a = Node::new_with("方向 A：睡眠衛生".to_string(), 2, "方向 A：睡眠衛生".to_string(), 0.7, 0.85, 1.0);
        let dir_a_id = dir_a.id.clone();
        graph.add_node(dir_a);
        graph.add_edge(Edge::new(root_id.clone(), dir_a_id.clone(), EdgeType::Reasoning));

        let leaf_a1 = Node::new_with("光線控制".to_string(), 3, "光線控制".to_string(), 0.6, 0.9, 0.5);
        let leaf_a1_id = leaf_a1.id.clone();
        graph.add_node(leaf_a1);
        graph.add_edge(Edge::new(dir_a_id.clone(), leaf_a1_id.clone(), EdgeType::Reasoning));

        // 加入方向 B
        let dir_b = Node::new_with("方向 B：生理調節".to_string(), 2, "方向 B：生理調節".to_string(), 0.8, 0.8, 1.0);
        let dir_b_id = dir_b.id.clone();
        graph.add_node(dir_b);
        graph.add_edge(Edge::new(root_id.clone(), dir_b_id.clone(), EdgeType::Reasoning));

        let tree = DecisionTree::from_graph(&graph, "睡眠品質").unwrap();
        let markdown = tree.to_markdown();

        assert!(markdown.contains("決策樹：睡眠品質"));
        assert!(markdown.contains("方向 A"));
        assert!(markdown.contains("方向 B"));
        assert!(markdown.contains("光線控制"));
    }
}