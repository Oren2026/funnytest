//! Converge Engine 測試

use evolution_reasoning::engine::{ConvergeEngine, ThresholdGate};
use evolution_reasoning::models::{Graph, Node, NodeStatus};

fn setup_test_graph() -> (Graph, String, String, String) {
    let mut graph = Graph::new();

    // 高分節點
    let high = Node::new_with("高分".to_string(), 1, 0.9, 0.9, 10.0);
    let high_id = high.id.clone();

    // 中分節點
    let mid = Node::new_with("中分".to_string(), 1, 0.5, 0.5, 5.0);
    let mid_id = mid.id.clone();

    // 低分節點
    let low = Node::new_with("低分".to_string(), 1, 0.1, 0.1, 2.0);
    let low_id = low.id.clone();

    graph.add_node(high);
    graph.add_node(mid);
    graph.add_node(low);

    (graph, high_id, mid_id, low_id)
}

#[test]
fn test_converge_removes_low_score() {
    let (mut graph, high_id, mid_id, low_id) = setup_test_graph();

    let engine = ConvergeEngine::new();
    // threshold = 0.02，低分節點（score = 0.01）會被刪除
    // mid: 0.25 > 0.02，不會被刪除
    let pruned = engine.converge(&mut graph, Some(0.02));

    assert!(pruned.contains(&low_id));
    assert!(!pruned.contains(&high_id));
    assert!(!pruned.contains(&mid_id));
}

#[test]
fn test_converge_preserves_locked() {
    let mut graph = Graph::new();
    let mut node = Node::new_with("鎖定".to_string(), 1, 0.1, 0.1, 1.0);
    node.status = NodeStatus::Locked;
    let _node_id = node.id.clone();
    graph.add_node(node);

    let engine = ConvergeEngine::new();
    let pruned = engine.converge(&mut graph, Some(0.3));

    assert_eq!(pruned.len(), 0);
}

#[test]
fn test_converge_by_complexity() {
    let (mut graph, _high_id, _mid_id, _low_id) = setup_test_graph();

    // 總複雜度 = 10 + 5 + 2 = 17
    let engine = ConvergeEngine::new();

    // 限制為 12，應該刪除低分節點直到符合
    let pruned = engine.converge_by_complexity(&mut graph, 12.0);

    assert!(!pruned.is_empty());
}

#[test]
fn test_converge_by_confidence() {
    let (mut graph, _high_id, mid_id, low_id) = setup_test_graph();

    let engine = ConvergeEngine::new();
    // confidence < 0.6
    // mid: 0.5 < 0.6 → true，會被刪除
    // low: 0.1 < 0.6 → true，會被刪除
    let pruned = engine.converge_by_confidence(&mut graph, 0.6);

    assert!(pruned.contains(&mid_id));
    assert!(pruned.contains(&low_id));
}

#[test]
fn test_converge_smart() {
    let mut graph = Graph::new();
    let node = Node::new_with("高信心".to_string(), 1, 0.9, 0.9, 10.0);
    let node_id = node.id.clone();
    graph.add_node(node);

    let engine = ConvergeEngine::new();
    // 信心度 > 0.8，複雜度 > threshold(50)
    let pruned = engine.converge_smart(&mut graph, 60.0);

    assert!(pruned.contains(&node_id));
}

#[test]
fn test_converge_empty_graph() {
    let mut graph = Graph::new();
    let engine = ConvergeEngine::new();
    let pruned = engine.converge(&mut graph, Some(0.5));

    assert_eq!(pruned.len(), 0);
}

#[test]
fn test_converge_custom_threshold_gate() {
    let gate = ThresholdGate::new_with(30.0, 0.7);
    let engine = ConvergeEngine::new_with_gate(gate);

    assert_eq!(engine.get_threshold_gate().threshold, 30.0);
    assert_eq!(engine.get_threshold_gate().confidence_weight, 0.7);
}

#[test]
fn test_converge_set_threshold_gate() {
    let mut engine = ConvergeEngine::new();
    let new_gate = ThresholdGate::new_with(40.0, 0.5);
    engine.set_threshold_gate(new_gate.clone());

    assert_eq!(engine.get_threshold_gate().threshold, 40.0);
}
