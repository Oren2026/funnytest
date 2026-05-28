//! Graph 整合測試

use evolution_reasoning::models::{Edge, EdgeType, Graph, Node};

#[test]
fn test_graph_full_lifecycle() {
    let mut graph = Graph::new();

    // 1. 建立根節點
    let root = Node::new("根節點".to_string(), 1);
    let root_id = root.id.clone();
    graph.add_node(root);
    assert_eq!(graph.node_count(), 1);

    // 2. 加入子節點
    let child1 = Node::new("子節點 1".to_string(), 2);
    let child1_id = child1.id.clone();
    graph.add_node(child1);
    graph.add_edge(Edge::new(root_id.clone(), child1_id.clone(), EdgeType::Reasoning));

    // 3. 驗證父子關係
    let parents = graph.get_parents(&child1_id);
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0].id, root_id);

    let children = graph.get_children(&root_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, child1_id);
}

#[test]
fn test_graph_multiple_children() {
    let mut graph = Graph::new();
    let parent = Node::new("父節點".to_string(), 1);
    let parent_id = parent.id.clone();
    graph.add_node(parent);

    // 建立三個子節點
    for i in 1..=3 {
        let child = Node::new(format!("子節點 {}", i), 2);
        let child_id = child.id.clone();
        graph.add_node(child);
        graph.add_edge(Edge::new(parent_id.clone(), child_id, EdgeType::Divergence));
    }

    let children = graph.get_children(&parent_id);
    assert_eq!(children.len(), 3);
}

#[test]
fn test_graph_remove_with_edges() {
    let mut graph = Graph::new();
    let n1 = Node::new("節點1".to_string(), 1);
    let n2 = Node::new("節點2".to_string(), 2);
    let n1_id = n1.id.clone();
    let n2_id = n2.id.clone();
    graph.add_node(n1);
    graph.add_node(n2);
    graph.add_edge(Edge::new(n1_id.clone(), n2_id.clone(), EdgeType::Reasoning));

    assert_eq!(graph.edge_count(), 1);

    // 刪除節點，相關邊也會被刪除
    graph.remove_node(&n1_id);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_graph_complexity() {
    let mut graph = Graph::new();
    let n1 = Node::new_with("節點1".to_string(), 1, 1.0, 1.0, 10.0);
    let n2 = Node::new_with("節點2".to_string(), 2, 1.0, 1.0, 20.0);
    graph.add_node(n1);
    graph.add_node(n2);

    assert!((graph.total_complexity() - 30.0).abs() < 0.001);
}

#[test]
fn test_graph_pruned_not_counted() {
    let mut graph = Graph::new();
    let n1 = Node::new_with("節點1".to_string(), 1, 1.0, 1.0, 10.0);
    let n2 = Node::new_with("節點2".to_string(), 2, 1.0, 1.0, 20.0);
    let n2_id = n2.id.clone();
    graph.add_node(n1);
    graph.add_node(n2);

    graph.prune_node(&n2_id);
    assert!((graph.total_complexity() - 10.0).abs() < 0.001);
    assert_eq!(graph.node_count(), 1); // 只有一個 active 節點
}
