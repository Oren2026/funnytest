//! Diverge Engine 測試

use evolution_reasoning::engine::DivergeEngine;
use evolution_reasoning::models::{Graph, Node, NodeStatus};

#[test]
fn test_diverge_creates_children() {
    let mut graph = Graph::new();
    let parent = Node::new("父親節點".to_string(), 1);
    let parent_id = parent.id.clone();
    graph.add_node(parent);

    let engine = DivergeEngine::new();
    let children = engine.diverge(&mut graph, &parent_id, 3, None);

    assert_eq!(children.len(), 3);
    assert_eq!(graph.node_count(), 4); // 1 parent + 3 children
    assert_eq!(graph.edge_count(), 3);
}

#[test]
fn test_diverge_with_custom_contents() {
    let mut graph = Graph::new();
    let parent = Node::new("父親".to_string(), 1);
    let parent_id = parent.id.clone();
    graph.add_node(parent);

    let engine = DivergeEngine::new();
    let contents = vec![
        "方向 A".to_string(),
        "方向 B".to_string(),
        "方向 C".to_string(),
    ];
    let children = engine.diverge(&mut graph, &parent_id, 3, Some(contents));

    assert_eq!(children.len(), 3);
    assert_eq!(children[0].content, "方向 A");
    assert_eq!(children[1].content, "方向 B");
    assert_eq!(children[2].content, "方向 C");
}

#[test]
fn test_diverge_child_inherits_with_decay() {
    let mut graph = Graph::new();
    let parent = Node::new_with("父親".to_string(), 1, 1.0, 0.9, 0.0);
    let parent_id = parent.id.clone();
    let parent_weight = parent.weight;
    let parent_confidence = parent.confidence;
    let parent_step = parent.step;
    graph.add_node(parent);

    let engine = DivergeEngine::new();
    let children = engine.diverge(&mut graph, &parent_id, 1, None);

    assert_eq!(children.len(), 1);
    let child = &children[0];
    // 權重應該衰減（小於父節點）
    assert!(child.weight < parent_weight);
    // 信心度應該略微下降
    assert!(child.confidence < parent_confidence);
    // 步驟應該是父節點 + 1
    assert_eq!(child.step, parent_step + 1);
}

#[test]
fn test_diverge_cannot_on_pruned() {
    let mut graph = Graph::new();
    let mut parent = Node::new("父親".to_string(), 1);
    parent.status = NodeStatus::Pruned;
    let parent_id = parent.id.clone();
    graph.add_node(parent);

    let engine = DivergeEngine::new();
    let children = engine.diverge(&mut graph, &parent_id, 3, None);

    assert_eq!(children.len(), 0);
}

#[test]
fn test_diverge_cannot_on_locked() {
    let mut graph = Graph::new();
    let mut parent = Node::new("父親".to_string(), 1);
    parent.status = NodeStatus::Locked;
    let parent_id = parent.id.clone();
    graph.add_node(parent);

    let engine = DivergeEngine::new();
    let children = engine.diverge(&mut graph, &parent_id, 3, None);

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
fn test_diverge_only_generates_nodes() {
    let parent = Node::new_with("父親".to_string(), 1, 0.8, 0.8, 0.0);
    let engine = DivergeEngine::new_with_seed(42);
    let children = engine.diverge_only(&parent, 3);

    // 沒有加入圖中
    assert_eq!(children.len(), 3);
}

#[test]
fn test_diverge_seed_reproducibility() {
    let parent = Node::new_with("父親".to_string(), 1, 0.8, 0.8, 0.0);

    let engine1 = DivergeEngine::new_with_seed(12345);
    let children1 = engine1.diverge_only(&parent, 3);

    let engine2 = DivergeEngine::new_with_seed(12345);
    let children2 = engine2.diverge_only(&parent, 3);

    // 相同 seed 應該產生相同內容
    for (c1, c2) in children1.iter().zip(children2.iter()) {
        assert_eq!(c1.content, c2.content);
    }
}
