//! 圖（Graph）資料結構
//!
//! Graph 是推理圖的核心結構，包含節點和邊的集合。

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::edge::{Edge, EdgeType};
use super::node::{Node, NodeStatus};

/// 主題階段（Topic Phase）
///
/// v0.7 新增：用於多主題並行時，各主題的獨立階段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicPhase {
    /// 探索期
    Exploration,
    /// 發展期
    Development,
    /// 成熟期
    Mature,
}

impl TopicPhase {
    /// 根據節點數量取得當前階段
    pub fn from_node_count(count: usize) -> Self {
        if count < 3 {
            TopicPhase::Exploration
        } else if count <= 10 {
            TopicPhase::Development
        } else {
            TopicPhase::Mature
        }
    }

    /// 取得階段名稱
    pub fn name(&self) -> &'static str {
        match self {
            TopicPhase::Exploration => "探索期",
            TopicPhase::Development => "發展期",
            TopicPhase::Mature => "成熟期",
        }
    }
}

/// 主題（Topic）
///
/// v0.7 新增：代表一個獨立的主題探索方向。
/// 每個主題有自己的根節點和階段。
#[derive(Debug, Clone)]
pub struct Topic {
    /// 主題 ID
    pub id: String,
    /// 根節點 ID
    pub root_node_id: String,
    /// 主題標題
    pub title: String,
    /// 創建時間
    pub created_at: DateTime<Utc>,
}

impl Topic {
    /// 建立新主題
    pub fn new(id: String, root_node_id: String, title: String) -> Self {
        Topic {
            id,
            root_node_id,
            title,
            created_at: Utc::now(),
        }
    }
}

/// 圖（Graph）
///
/// 包含所有節點和邊的集合，提供圖的基礎操作。
#[derive(Debug, Clone)]
pub struct Graph {
    /// 節點 map：ID -> Node
    pub nodes: HashMap<String, Node>,
    /// 邊 map：ID -> Edge
    pub edges: HashMap<String, Edge>,
    /// 主題 map：ID -> Topic（v0.7 新增）
    pub topics: HashMap<String, Topic>,
    /// 目前選中的主題 ID（v0.7 新增）
    pub current_topic_id: Option<String>,
}

impl Default for Graph {
    fn default() -> Self {
        Graph::new()
    }
}

impl Graph {
    /// 建立新的空圖
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            topics: HashMap::new(),
            current_topic_id: None,
        }
    }

    /// 新增節點
    ///
    /// # 引數
    /// - `node`: 要新增的節點
    ///
    /// # 範例
    /// ```
    /// let mut graph = Graph::new();
    /// let node = Node::new("想法".to_string(), 1);
    /// graph.add_node(node);
    /// ```
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// 新增邊（自動維護節點的 parent/child edges）
    ///
    /// # 引數
    /// - `edge`: 要新增的邊
    pub fn add_edge(&mut self, mut edge: Edge) {
        let edge_id = edge.id.clone();

        // 維護 from 節點的 child_edges
        if let Some(from_node) = self.nodes.get_mut(&edge.from) {
            from_node.add_child_edge(edge_id.clone());
        }

        // 維護 to 節點的 parent_edges
        if let Some(to_node) = self.nodes.get_mut(&edge.to) {
            to_node.add_parent_edge(edge_id.clone());
        }

        self.edges.insert(edge_id, edge);
    }

    /// 刪除節點（同時刪除相關的邊）
    ///
    /// # 引數
    /// - `id`: 節點 ID
    pub fn remove_node(&mut self, id: &str) -> Option<Node> {
        // 找出所有與此節點相連的邊
        let edge_ids: Vec<String> = self
            .edges
            .values()
            .filter(|e| e.from == id || e.to == id)
            .map(|e| e.id.clone())
            .collect();

        // 刪除所有相關邊
        for edge_id in &edge_ids {
            self.remove_edge(edge_id);
        }

        // 刪除節點
        self.nodes.remove(id)
    }

    /// 刪除邊（同時維護節點的 parent/child edges）
    ///
    /// # 引數
    /// - `id`: 邊 ID
    pub fn remove_edge(&mut self, id: &str) -> Option<Edge> {
        if let Some(edge) = self.edges.remove(id) {
            // 從 from 節點移除 child_edges
            if let Some(from_node) = self.nodes.get_mut(&edge.from) {
                from_node.remove_child_edge(id);
            }

            // 從 to 節點移除 parent_edges
            if let Some(to_node) = self.nodes.get_mut(&edge.to) {
                to_node.remove_parent_edge(id);
            }

            return Some(edge);
        }
        None
    }

    /// 取得節點
    ///
    /// # 引數
    /// - `id`: 節點 ID
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// 取得節點（可變）
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// 取得子節點
    ///
    /// # 引數
    /// - `id`: 節點 ID
    ///
    /// # 範例
    /// ```
    /// let children = graph.get_children("node1");
    /// ```
    pub fn get_children(&self, id: &str) -> Vec<&Node> {
        let node = match self.nodes.get(id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        node.child_edges
            .iter()
            .filter_map(|edge_id| self.edges.get(edge_id))
            .filter_map(|edge| self.nodes.get(&edge.to))
            .collect()
    }

    /// 取得父節點
    ///
    /// # 引數
    /// - `id`: 節點 ID
    pub fn get_parents(&self, id: &str) -> Vec<&Node> {
        let node = match self.nodes.get(id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        node.parent_edges
            .iter()
            .filter_map(|edge_id| self.edges.get(edge_id))
            .filter_map(|edge| self.nodes.get(&edge.from))
            .collect()
    }

    /// 取得所有根節點（沒有父節點的節點）
    pub fn get_root_nodes(&self) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| n.parent_edges.is_empty())
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════
    // 主題管理方法（v0.7 新增）
    // ═══════════════════════════════════════════════════════════════════

    /// 新增主題（v0.7 新增）
    ///
    /// 建立新主題並新增對應的根節點。
    ///
    /// # 引數
    /// - `title`: 主題標題
    ///
    /// # 回傳
    /// 新建立的主題
    pub fn add_topic(&mut self, title: String) -> Topic {
        // 建立根節點
        let root_node = Node::new(title.clone(), 1);
        let root_node_id = root_node.id.clone();

        // 建立主題
        let topic = Topic::new(
            uuid::Uuid::new_v4().to_string(),
            root_node_id.clone(),
            title,
        );

        // 加入圖
        self.add_node(root_node);
        self.topics.insert(topic.id.clone(), topic.clone());
        self.current_topic_id = Some(topic.id.clone());

        topic
    }

    /// 取得所有主題（v0.7 新增）
    pub fn get_topics(&self) -> Vec<&Topic> {
        self.topics.values().collect()
    }

    /// 取得目前選中的主題（v0.7 新增）
    pub fn get_current_topic(&self) -> Option<&Topic> {
        self.current_topic_id
            .as_ref()
            .and_then(|id| self.topics.get(id))
    }

    /// 設定目前選中的主題（v0.7 新增）
    pub fn set_current_topic(&mut self, id: &str) -> bool {
        if self.topics.contains_key(id) {
            self.current_topic_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// 取得目前主題的根節點（v0.7 新增）
    ///
    /// 如果沒有選中任何主題，回傳所有根節點。
    pub fn get_current_topic_root_nodes(&self) -> Vec<&Node> {
        if let Some(topic) = self.get_current_topic() {
            // 回傳特定主題的根節點
            if let Some(node) = self.nodes.get(&topic.root_node_id) {
                vec![node]
            } else {
                Vec::new()
            }
        } else {
            // 沒有選中主題，回傳所有根節點
            self.get_root_nodes()
        }
    }

    /// 計算主題的節點數量（v0.7 新增）
    ///
    /// 計算指定主題根節點下的所有節點數量。
    pub fn count_topic_nodes(&self, topic_id: &str) -> usize {
        if let Some(topic) = self.topics.get(topic_id) {
            self.count_descendants(&topic.root_node_id)
        } else {
            0
        }
    }

    /// 計算節點的所有子孫節點數量
    fn count_descendants(&self, node_id: &str) -> usize {
        let mut count = 1; // 包含自己
        for child in self.get_children(node_id) {
            count += self.count_descendants(&child.id);
        }
        count
    }

    /// 取得主題的階段（v0.7 新增）
    pub fn get_topic_phase(&self, topic_id: &str) -> TopicPhase {
        let node_count = self.count_topic_nodes(topic_id);
        TopicPhase::from_node_count(node_count)
    }

    /// 取得所有葉節點（沒有子節點的節點）
    pub fn get_leaf_nodes(&self) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| n.child_edges.is_empty())
            .collect()
    }

    /// 取得圖中所有節點
    pub fn get_all_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    /// 取得圖中所有邊
    pub fn get_all_edges(&self) -> Vec<&Edge> {
        self.edges.values().collect()
    }

    /// 取得節點的深度（從根節點計算）
    ///
    /// # 引數
    /// - `id`: 節點 ID
    /// - `visited`: 已經拜訪過的節點 ID（用於檢測迴圈）
    pub fn get_depth(&self, id: &str, mut visited: Vec<String>) -> i32 {
        if visited.contains(&id.to_string()) {
            // 檢測到迴圈
            return 0;
        }

        visited.push(id.to_string());

        let parents = self.get_parents(id);
        if parents.is_empty() {
            return 1;
        }

        let max_parent_depth = parents
            .iter()
            .map(|p| self.get_depth(&p.id, visited.clone()))
            .max()
            .unwrap_or(1);

        max_parent_depth + 1
    }

    /// 計算圖的總複雜度
    pub fn total_complexity(&self) -> f64 {
        self.nodes
            .values()
            .filter(|n| n.status != NodeStatus::Pruned)
            .map(|n| n.complexity)
            .sum()
    }

    /// 找出所有需要收斂的節點（分數低於閾值）
    ///
    /// # 引數
    /// - `threshold`: 分數閾值
    pub fn find_low_score_nodes(&self, threshold: f64) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| n.status != NodeStatus::Pruned && n.score() < threshold)
            .collect()
    }

    /// 標記節點為已刪除（Pruned）
    pub fn prune_node(&mut self, id: &str) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Pruned;
            true
        } else {
            false
        }
    }

    /// 將節點狀態改為 Locked
    pub fn lock_node(&mut self, id: &str) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Locked;
            true
        } else {
            false
        }
    }

    /// 將節點狀態改為 Active
    pub fn activate_node(&mut self, id: &str) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Active;
            true
        } else {
            false
        }
    }

    /// 取得目前非刪除狀態的節點數量
    pub fn node_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.status != NodeStatus::Pruned)
            .count()
    }

    /// 取得目前非刪除狀態的邊數量
    pub fn edge_count(&self) -> usize {
        // 計算兩端節點都不是 Pruned 的邊
        self.edges
            .values()
            .filter(|e| {
                let from_active = self
                    .nodes
                    .get(&e.from)
                    .map(|n| n.status != NodeStatus::Pruned)
                    .unwrap_or(false);
                let to_active = self
                    .nodes
                    .get(&e.to)
                    .map(|n| n.status != NodeStatus::Pruned)
                    .unwrap_or(false);
                from_active && to_active
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_new() {
        let graph = Graph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_graph_add_node() {
        let mut graph = Graph::new();
        let node = Node::new("測試節點".to_string(), 1);
        let node_id = node.id.clone();
        graph.add_node(node);

        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.get_node(&node_id).is_some());
    }

    #[test]
    fn test_graph_add_edge() {
        let mut graph = Graph::new();
        let node1 = Node::new("節點1".to_string(), 1);
        let node2 = Node::new("節點2".to_string(), 2);
        let node1_id = node1.id.clone();
        let node2_id = node2.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);

        let edge = Edge::new(node1_id.clone(), node2_id.clone(), EdgeType::Reasoning);
        let edge_id = edge.id.clone();
        graph.add_edge(edge);

        assert_eq!(graph.edges.len(), 1);

        // 檢查節點的 child_edges 和 parent_edges 是否正確維護
        let n1 = graph.get_node(&node1_id).unwrap();
        let n2 = graph.get_node(&node2_id).unwrap();
        assert!(n1.child_edges.contains(&edge_id));
        assert!(n2.parent_edges.contains(&edge_id));
    }

    #[test]
    fn test_graph_remove_node() {
        let mut graph = Graph::new();
        let node1 = Node::new("節點1".to_string(), 1);
        let node2 = Node::new("節點2".to_string(), 2);
        let node1_id = node1.id.clone();
        let node2_id = node2.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);

        let edge = Edge::new(node1_id.clone(), node2_id.clone(), EdgeType::Reasoning);
        graph.add_edge(edge);

        graph.remove_node(&node1_id);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0); // 邊也會被刪除
        assert!(graph.get_node(&node1_id).is_none());
    }

    #[test]
    fn test_graph_get_children() {
        let mut graph = Graph::new();
        let node1 = Node::new("節點1".to_string(), 1);
        let node2 = Node::new("節點2".to_string(), 2);
        let node1_id = node1.id.clone();
        let node2_id = node2.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);

        let edge = Edge::new(node1_id.clone(), node2_id.clone(), EdgeType::Reasoning);
        graph.add_edge(edge);

        let children = graph.get_children(&node1_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, node2_id);
    }

    #[test]
    fn test_graph_get_parents() {
        let mut graph = Graph::new();
        let node1 = Node::new("節點1".to_string(), 1);
        let node2 = Node::new("節點2".to_string(), 2);
        let node1_id = node1.id.clone();
        let node2_id = node2.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);

        let edge = Edge::new(node1_id.clone(), node2_id.clone(), EdgeType::Reasoning);
        graph.add_edge(edge);

        let parents = graph.get_parents(&node2_id);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].id, node1_id);
    }

    #[test]
    fn test_graph_get_root_nodes() {
        let mut graph = Graph::new();
        let node1 = Node::new("節點1".to_string(), 1);
        let node2 = Node::new("節點2".to_string(), 2);
        let node1_id = node1.id.clone();
        let node2_id = node2.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);

        let edge = Edge::new(node1_id.clone(), node2_id, EdgeType::Reasoning);
        graph.add_edge(edge);

        let roots = graph.get_root_nodes();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, node1_id);
    }

    #[test]
    fn test_graph_prune_node() {
        let mut graph = Graph::new();
        let node = Node::new("測試".to_string(), 1);
        let node_id = node.id.clone();
        graph.add_node(node);

        graph.prune_node(&node_id);

        let n = graph.get_node(&node_id).unwrap();
        assert_eq!(n.status, NodeStatus::Pruned);
    }

    #[test]
    fn test_graph_total_complexity() {
        let mut graph = Graph::new();
        let node1 = Node::new_with("節點1".to_string(), 1, 1.0, 1.0, 10.0);
        let node2 = Node::new_with("節點2".to_string(), 2, 1.0, 1.0, 20.0);

        graph.add_node(node1);
        graph.add_node(node2);

        assert!((graph.total_complexity() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_graph_find_low_score_nodes() {
        let mut graph = Graph::new();
        let node1 = Node::new_with("節點1".to_string(), 1, 0.9, 0.9, 0.0); // score = 0.81
        let node2 = Node::new_with("節點2".to_string(), 2, 0.1, 0.1, 0.0); // score = 0.01

        graph.add_node(node1);
        graph.add_node(node2);

        let low = graph.find_low_score_nodes(0.5);
        assert_eq!(low.len(), 1);
    }

    #[test]
    fn test_graph_node_count() {
        let mut graph = Graph::new();
        graph.add_node(Node::new("節點1".to_string(), 1));
        graph.add_node(Node::new("節點2".to_string(), 2));

        assert_eq!(graph.node_count(), 2);

        // prune 一個節點
        let node_id = graph.get_all_nodes()[0].id.clone();
        graph.prune_node(&node_id);

        assert_eq!(graph.node_count(), 1);
    }
}
