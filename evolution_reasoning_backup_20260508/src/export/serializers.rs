//! 序列化器（Serializers）
//!
//! 將 Rust 結構轉換為 YAML、JSON、DSL 格式。

use crate::export::{
    BacktrackExportView, GraphExportView, HypothesisExportView,
    HypothesesExportView, MemoryExportView, NodeExportView,
};

use serde::Serialize;

/// 序列化成 YAML
pub fn to_yaml<T: Serialize>(value: &T) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(value)
}

/// 序列化成 JSON（格式化）
pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

// ─────────────────────────────────────────────────────────────────────────────
// DSL 生成器
// ─────────────────────────────────────────────────────────────────────────────

/// 將 GraphExportView 轉換為 DSL
pub fn to_dsl_graph(view: &GraphExportView) -> String {
    let mut dsl = String::new();
    dsl.push_str("# Evolution Reasoning Graph — DSL Export\n");
    dsl.push_str(&format!("# Generated at: {}\n", chrono::Utc::now().to_rfc3339()));
    dsl.push_str(&format!(
        "# Nodes: {}, Edges: {}, Topics: {}\n\n",
        view.node_count, view.edge_count, view.topic_count
    ));
    dsl.push_str("graph {\n");
    dsl.push_str("  meta {\n");
    dsl.push_str(&format!("    total_complexity: {}\n", view.total_complexity));
    dsl.push_str(&format!(
        "    status_counts {{ locked: {}, draft: {}, pruned: {} }}\n",
        view.locked_nodes, view.draft_nodes, view.pruned_nodes
    ));
    dsl.push_str("  }\n\n");

    // 主題
    if !view.topics.is_empty() {
        dsl.push_str("  topics [\n");
        for topic in &view.topics {
            dsl.push_str(&format!(
                "    {{ id: {}, title: \"{}\", root: {} }}\n",
                topic.id,
                escape_dsl_string(&topic.title),
                topic.root_node_id
            ));
        }
        dsl.push_str("  ]\n\n");
    }

    // 節點
    dsl.push_str("  nodes [\n");
    for node in &view.nodes {
        dsl.push_str(&format!(
            "    {{ id: {}, step: {}, status: {}, weight: {}, confidence: {}, complexity: {} }}\n      \"{}\"\n",
            node.id,
            node.step,
            node.status.to_lowercase(),
            node.weight,
            node.confidence,
            node.complexity,
            escape_dsl_string(&node.content)
        ));
    }
    dsl.push_str("  ]\n\n");

    // 邊
    dsl.push_str("  edges [\n");
    for edge in &view.edges {
        dsl.push_str(&format!(
            "    {{ id: {}, {} => {}, type: {}, weight: {} }}\n",
            edge.id,
            edge.from_id,
            edge.to_id,
            edge.edge_type.to_lowercase(),
            edge.weight
        ));
    }
    dsl.push_str("  ]\n");
    dsl.push_str("}\n");
    dsl
}

/// 將 NodeExportView 轉換為 DSL
pub fn to_dsl_node(view: &NodeExportView) -> String {
    let mut dsl = String::new();
    dsl.push_str(&format!("# Node {} — DSL Export\n\n", view.id));
    dsl.push_str("node {\n");
    dsl.push_str(&format!("  id: {}\n", view.id));
    dsl.push_str(&format!("  step: {}\n", view.step));
    dsl.push_str(&format!("  status: {}\n", view.status.to_lowercase()));
    dsl.push_str(&format!("  weight: {}\n", view.weight));
    dsl.push_str(&format!("  confidence: {}\n", view.confidence));
    dsl.push_str(&format!("  complexity: {}\n", view.complexity));
    dsl.push_str(&format!("  parents: {}\n", view.parent_count));
    dsl.push_str(&format!("  children: {}\n", view.child_count));
    dsl.push_str("  content: \"");
    dsl.push_str(&escape_dsl_string(&view.content));
    dsl.push_str("\"\n");
    dsl.push_str("}\n");
    dsl
}

/// 將 BacktrackExportView 轉換為 DSL
pub fn to_dsl_backtrack(view: &BacktrackExportView) -> String {
    let mut dsl = String::new();
    dsl.push_str("# Evolution Backtrack State — DSL Export\n");
    dsl.push_str(&format!(
        "# Checkpoints: {}, Failures: {}, Hypothesis Groups: {}\n\n",
        view.checkpoint_count, view.failure_count, view.total_groups
    ));
    dsl.push_str("backtrack {\n");

    // Checkpoints
    dsl.push_str("  checkpoints [\n");
    for cp in &view.checkpoints {
        dsl.push_str(&format!(
            "    {{ id: {}, node: {}, reason: {}, at: {}, snapshot_bytes: {} }}\n",
            cp.id, cp.node_id, cp.reason, cp.created_at, cp.snapshot_bytes
        ));
    }
    dsl.push_str("  ]\n\n");

    // Failures
    dsl.push_str("  failures [\n");
    for f in &view.failures {
        dsl.push_str(&format!(
            "    {{ id: {}, node: {}, pattern: {}, exit: {:?}, at: {} }}\n",
            f.id, f.node_id, f.pattern_type, f.exit_code, f.recorded_at
        ));
    }
    dsl.push_str("  ]\n\n");

    // Latest Hypotheses
    dsl.push_str("  latest_hypotheses [\n");
    for h in &view.latest_hypotheses {
        dsl.push_str(&format!(
            "    {{ id: {}, failure: {}, confidence: {} }}\n      hypothesis: \"{}\"\n      action: \"{}\"\n",
            h.id,
            h.failure_id,
            h.confidence,
            escape_dsl_string(&h.hypothesis),
            escape_dsl_string(&h.suggested_action)
        ));
    }
    dsl.push_str("  ]\n");
    dsl.push_str("}\n");
    dsl
}

/// 將 HypothesesExportView 轉換為 DSL
pub fn to_dsl_hypotheses(view: &HypothesesExportView) -> String {
    let mut dsl = String::new();
    dsl.push_str("# Evolution Hypotheses — DSL Export\n");
    dsl.push_str(&format!(
        "# {} failures, {} total hypotheses\n\n",
        view.total_failures, view.total_hypotheses
    ));
    dsl.push_str("hypotheses {\n");
    dsl.push_str(&format!("  total_failures: {}\n", view.total_failures));
    dsl.push_str(&format!("  total_hypotheses: {}\n", view.total_hypotheses));
    dsl.push_str("  items [\n");
    for h in &view.hypotheses {
        dsl.push_str(&format!(
            "    {{ failure: {}, confidence: {} }}\n      hypothesis: \"{}\"\n      action: \"{}\"\n",
            h.failure_id,
            h.confidence,
            escape_dsl_string(&h.hypothesis),
            escape_dsl_string(&h.suggested_action)
        ));
    }
    dsl.push_str("  ]\n");
    dsl.push_str("}\n");
    dsl
}

/// 將 MemoryExportView 轉換為 DSL
pub fn to_dsl_memory(view: &MemoryExportView) -> String {
    let mut dsl = String::new();
    dsl.push_str("# Evolution Memory State — DSL Export\n\n");
    dsl.push_str("memory {\n");
    dsl.push_str("  profile: \"");
    dsl.push_str(&escape_dsl_string(&view.profile_summary));
    dsl.push_str("\"\n");
    dsl.push_str(&format!("  total_history: {}\n", view.total_history));
    dsl.push_str(&format!("  explored_topics: {}\n", view.explored_topics));
    dsl.push_str("  recent_topics: [");
    dsl.push_str(
        &view
            .recent_topics
            .iter()
            .map(|t| format!("\"{}\"", escape_dsl_string(t)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    dsl.push_str("]\n");
    dsl.push_str("}\n");
    dsl
}

/// 逸出 DSL 字串中的特殊字元
fn escape_dsl_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_dsl_string() {
        assert_eq!(escape_dsl_string("hello"), "hello");
        assert_eq!(
            escape_dsl_string("hello \"world\""),
            "hello \\\"world\\\""
        );
        assert_eq!(escape_dsl_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_to_yaml_roundtrip() {
        #[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            value: i32,
        }
        let val = TestStruct {
            name: "test".to_string(),
            value: 42,
        };
        let yaml = to_yaml(&val).unwrap();
        assert!(yaml.contains("name: test"));
        assert!(yaml.contains("value: 42"));
    }

    #[test]
    fn test_to_json_roundtrip() {
        #[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            items: Vec<i32>,
        }
        let val = TestStruct {
            name: "test".to_string(),
            items: vec![1, 2, 3],
        };
        let json = to_json(&val).unwrap();
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("1") && json.contains("2") && json.contains("3"));
    }

    #[test]
    fn test_dsl_graph_format() {
        let view = GraphExportView {
            node_count: 2,
            edge_count: 1,
            topic_count: 0,
            nodes: vec![NodeExportView {
                id: "n1".to_string(),
                step: 1,
                content: "test content".to_string(),
                weight: 1.0,
                confidence: 0.9,
                complexity: 0.5,
                status: "Active".to_string(),
                parent_count: 0,
                child_count: 1,
            }],
            edges: vec![],
            topics: vec![],
            total_complexity: 0.5,
            locked_nodes: 0,
            draft_nodes: 1,
            pruned_nodes: 0,
        };
        let dsl = to_dsl_graph(&view);
        assert!(dsl.contains("graph {"));
        assert!(dsl.contains("node_count") || dsl.contains("Nodes:"));
        assert!(dsl.contains("test content"));
        assert!(dsl.contains('}'));
    }
}
