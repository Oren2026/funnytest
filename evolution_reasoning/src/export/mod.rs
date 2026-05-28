//! 部署介面（Export Module）
//!
//! v0.8-C 新增：匯出推理圖狀態為外部可用格式。
//! - YAML/DSL 匯出：結構化狀態輸出
//! - HTTP 查詢 API：狀態端點
//! - stdin 外部觸發：接收外部任務
//! - AI Export：推理過程餵給其他 AI

pub mod serializers;

use serde::{Deserialize, Serialize};

use crate::engine::BacktrackManager;
use crate::models::{Edge, EdgeType, Graph, Node, NodeStatus};
use crate::memory::MemoryManager;

// ─────────────────────────────────────────────────────────────────────────────
// 匯出格式與視圖
// ─────────────────────────────────────────────────────────────────────────────

/// 匯出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Yaml,
    Json,
    Dsl,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Yaml
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Yaml => write!(f, "yaml"),
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Dsl => write!(f, "dsl"),
        }
    }
}

/// 圖匯出視圖（去除內部複雜性）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportView {
    pub node_count: usize,
    pub edge_count: usize,
    pub topic_count: usize,
    pub nodes: Vec<NodeExportView>,
    pub edges: Vec<EdgeExportView>,
    pub topics: Vec<TopicExportView>,
    pub total_complexity: f64,
    pub locked_nodes: usize,
    pub draft_nodes: usize,
    pub pruned_nodes: usize,
}

/// 節點匯出視圖（精簡版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExportView {
    pub id: String,
    pub step: i32,
    pub content: String,
    pub weight: f64,
    pub confidence: f64,
    pub complexity: f64,
    pub status: String,
    pub parent_count: usize,
    pub child_count: usize,
}

impl From<&Node> for NodeExportView {
    fn from(node: &Node) -> Self {
        NodeExportView {
            id: node.id.clone(),
            step: node.step,
            content: node.content.clone(),
            weight: node.weight,
            confidence: node.confidence,
            complexity: node.complexity,
            status: format!("{:?}", node.status),
            parent_count: node.parent_edges.len(),
            child_count: node.child_edges.len(),
        }
    }
}

/// 邊匯出視圖（精簡版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeExportView {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
    pub weight: f64,
}

impl From<&Edge> for EdgeExportView {
    fn from(edge: &Edge) -> Self {
        EdgeExportView {
            id: edge.id.clone(),
            from_id: edge.from.clone(),
            to_id: edge.to.clone(),
            edge_type: format!("{:?}", edge.edge_type),
            weight: edge.weight,
        }
    }
}

/// 主題匯出視圖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicExportView {
    pub id: String,
    pub title: String,
    pub root_node_id: String,
    pub created_at: String,
}

impl From<&crate::models::Topic> for TopicExportView {
    fn from(topic: &crate::models::Topic) -> Self {
        TopicExportView {
            id: topic.id.clone(),
            title: topic.title.clone(),
            root_node_id: topic.root_node_id.clone(),
            created_at: topic.created_at.to_rfc3339(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Backtrack 匯出結構（給序列化器用，欄位名稱對齊 DSL）
// ─────────────────────────────────────────────────────────────────────────────

/// Backtrack 匯出結構（DSL 序列化器專用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktrackExportView {
    pub checkpoint_count: usize,
    pub failure_count: usize,
    pub checkpoints: Vec<CheckpointExportView>,
    pub failures: Vec<FailureExportView>,
    pub latest_hypotheses: Vec<HypothesisExportView>,
    pub total_groups: usize,
}

/// Checkpoint 匯出視圖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointExportView {
    pub id: String,
    pub node_id: String,
    pub reason: String,
    pub created_at: String,
    pub snapshot_bytes: usize,
}

/// Failure 匯出視圖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureExportView {
    pub id: String,
    pub node_id: String,
    pub pattern_type: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub recorded_at: String,
}

/// Hypothesis 匯出視圖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisExportView {
    pub id: String,
    pub failure_id: String,
    pub hypothesis: String,
    pub suggested_action: String,
    pub confidence: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Hypotheses 匯出結構
// ─────────────────────────────────────────────────────────────────────────────

/// Hypotheses 匯出結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesesExportView {
    pub total_failures: usize,
    pub total_hypotheses: usize,
    pub hypotheses: Vec<HypothesisExportView>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory 匯出結構
// ─────────────────────────────────────────────────────────────────────────────

/// Memory 匯出結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryExportView {
    pub memory_path: String,
    pub profile_exists: bool,
    pub history_exists: bool,
    pub topics_exists: bool,
    pub profile_summary: String,
    pub total_history: usize,
    pub explored_topics: usize,
    pub recent_topics: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// 轉換函式
// ─────────────────────────────────────────────────────────────────────────────

/// 將 Graph 轉換為 GraphExportView
pub fn graph_to_export_view(graph: &Graph) -> GraphExportView {
    let nodes: Vec<&Node> = graph.nodes.values().collect();
    let locked_nodes = nodes.iter().filter(|n| n.status == NodeStatus::Locked).count();
    let draft_nodes = nodes.iter().filter(|n| n.status == NodeStatus::Draft).count();
    let pruned_nodes = nodes.iter().filter(|n| n.status == NodeStatus::Pruned).count();
    let total_complexity: f64 = nodes.iter().map(|n| n.complexity).sum();

    GraphExportView {
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        topic_count: graph.topics.len(),
        nodes: nodes.iter().map(|n| NodeExportView::from(*n)).collect(),
        edges: graph.edges.values().map(EdgeExportView::from).collect(),
        topics: graph.topics.values().map(TopicExportView::from).collect(),
        total_complexity,
        locked_nodes,
        draft_nodes,
        pruned_nodes,
    }
}

/// 匯出 Graph 為指定格式
pub fn export_graph(graph: &Graph, format: ExportFormat) -> String {
    let view = graph_to_export_view(graph);
    match format {
        ExportFormat::Yaml => serializers::to_yaml(&view).unwrap_or_default(),
        ExportFormat::Json => serializers::to_json(&view).unwrap_or_default(),
        ExportFormat::Dsl => serializers::to_dsl_graph(&view),
    }
}

/// 匯出單一節點為指定格式
pub fn export_node(node: &Node, format: ExportFormat) -> String {
    let view = NodeExportView::from(node);
    match format {
        ExportFormat::Yaml => serializers::to_yaml(&view).unwrap_or_default(),
        ExportFormat::Json => serializers::to_json(&view).unwrap_or_default(),
        ExportFormat::Dsl => serializers::to_dsl_node(&view),
    }
}

/// 匯出 BacktrackManager 狀態
pub fn export_backtrack(bt: &BacktrackManager, format: ExportFormat) -> String {
    let checkpoints: Vec<CheckpointExportView> = bt.get_checkpoints()
        .iter()
        .map(|cp| CheckpointExportView {
            id: cp.id.clone(),
            node_id: cp.node_id.clone(),
            reason: cp.reason.to_string(),
            created_at: cp.created_at.to_rfc3339(),
            snapshot_bytes: cp.snapshot.len(),
        })
        .collect();

    let failures: Vec<FailureExportView> = bt.get_failure_history()
        .iter()
        .map(|f| FailureExportView {
            id: f.id.clone(),
            node_id: f.context_node_id.clone(),
            pattern_type: format!("{:?}", f.pattern_type),
            command: f.command.clone(),
            exit_code: f.exit_code,
            recorded_at: f.occurred_at.to_rfc3339(),
        })
        .collect();

    let latest_hypotheses: Vec<HypothesisExportView> = bt.get_failure_history()
        .last()
        .map(|f| {
            bt.get_hypotheses(&f.id)
                .into_iter()
                .map(|h| HypothesisExportView {
                    id: h.id.clone(),
                    failure_id: h.original_failure.clone(),
                    hypothesis: h.hypothesis.clone(),
                    suggested_action: h.suggested_action.clone(),
                    confidence: h.confidence,
                })
                .collect()
        })
        .unwrap_or_default();

    let view = BacktrackExportView {
        checkpoint_count: bt.checkpoint_count(),
        failure_count: bt.failure_count(),
        checkpoints,
        failures,
        latest_hypotheses,
        total_groups: bt.failure_count(),
    };

    match format {
        ExportFormat::Yaml => serializers::to_yaml(&view).unwrap_or_default(),
        ExportFormat::Json => serializers::to_json(&view).unwrap_or_default(),
        ExportFormat::Dsl => serializers::to_dsl_backtrack(&view),
    }
}

/// 匯出假設列表
pub fn export_hypotheses(bt: &BacktrackManager, _failure_id: Option<&str>, format: ExportFormat) -> String {
    let failure_history = bt.get_failure_history();
    let hypotheses: Vec<HypothesisExportView> = failure_history
        .iter()
        .flat_map(|f| {
            bt.get_hypotheses(&f.id)
                .into_iter()
                .map(|h| HypothesisExportView {
                    id: h.id.clone(),
                    failure_id: h.original_failure.clone(),
                    hypothesis: h.hypothesis.clone(),
                    suggested_action: h.suggested_action.clone(),
                    confidence: h.confidence,
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let view = HypothesesExportView {
        total_failures: failure_history.len(),
        total_hypotheses: hypotheses.len(),
        hypotheses,
    };

    match format {
        ExportFormat::Yaml => serializers::to_yaml(&view).unwrap_or_default(),
        ExportFormat::Json => serializers::to_json(&view).unwrap_or_default(),
        ExportFormat::Dsl => serializers::to_dsl_hypotheses(&view),
    }
}

/// 匯出 MemoryManager 狀態
pub fn export_memory(mem: &MemoryManager, format: ExportFormat) -> String {
    let (profile_exists, profile_raw) = mem.read_profile();
    let (history_exists, history_raw) = mem.read_history();
    let (topics_exists, topics_raw) = mem.read_topics();

    // profile_summary：取前 200 字元（避免過長）
    let profile_summary = if profile_raw.is_empty() {
        String::new()
    } else {
        profile_raw.chars().take(200).collect::<String>()
    };

    // 從 history 估算歷史筆數（每個 entry 以 "---" 分隔）
    let total_history = history_raw.split("---").filter(|s| !s.trim().is_empty()).count();

    // 從 topics raw 估算主題數（每行大約一個主題）
    let explored_topics = topics_raw.lines().filter(|l| !l.trim().is_empty()).count();

    // 最近主題取前 5 行
    let recent_topics: Vec<String> = topics_raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(5)
        .map(|l| l.to_string())
        .collect();

    let view = MemoryExportView {
        memory_path: mem.memory_path().display().to_string(),
        profile_exists,
        history_exists,
        topics_exists,
        profile_summary,
        total_history,
        explored_topics,
        recent_topics,
    };

    match format {
        ExportFormat::Yaml => serializers::to_yaml(&view).unwrap_or_default(),
        ExportFormat::Json => serializers::to_json(&view).unwrap_or_default(),
        ExportFormat::Dsl => serializers::to_dsl_memory(&view),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP API 類型（狀態查詢端點）
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;

/// HTTP API 查詢請求
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub resource: String,
    pub format: Option<String>,
}

/// HTTP API 回應
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub resource: String,
    pub format: String,
    pub content: String,
    pub timestamp: String,
}

impl QueryResponse {
    pub fn new(resource: &str, format: &str, content: String) -> Self {
        QueryResponse {
            resource: resource.to_string(),
            format: format.to_string(),
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 查詢 BacktrackManager 狀態
pub fn query_backtrack(bt: &BacktrackManager, resource: &str) -> String {
    match resource {
        "checkpoints" => {
            let cps: Vec<CheckpointExportView> = bt.get_checkpoints()
                .iter()
                .map(|cp| CheckpointExportView {
                    id: cp.id.clone(),
                    node_id: cp.node_id.clone(),
                    reason: cp.reason.to_string(),
                    created_at: cp.created_at.to_rfc3339(),
                    snapshot_bytes: cp.snapshot.len(),
                })
                .collect();
            serde_json::to_string_pretty(&cps).unwrap_or_else(|_| "{}".to_string())
        }
        "failures" => {
            let failures: Vec<FailureExportView> = bt.get_failure_history()
                .iter()
                .map(|f| FailureExportView {
                    id: f.id.clone(),
                    node_id: f.context_node_id.clone(),
                    pattern_type: format!("{:?}", f.pattern_type),
                    command: f.command.clone(),
                    exit_code: f.exit_code,
                    recorded_at: f.occurred_at.to_rfc3339(),
                })
                .collect();
            serde_json::to_string_pretty(&failures).unwrap_or_else(|_| "{}".to_string())
        }
        "hypotheses" => {
            let hypotheses: Vec<HypothesisExportView> = bt.get_failure_history()
                .last()
                .map(|f| {
                    bt.get_hypotheses(&f.id)
                        .into_iter()
                        .map(|h| HypothesisExportView {
                            id: h.id.clone(),
                            failure_id: h.original_failure.clone(),
                            hypothesis: h.hypothesis.clone(),
                            suggested_action: h.suggested_action.clone(),
                            confidence: h.confidence,
                        })
                        .collect()
                })
                .unwrap_or_default();
            serde_json::to_string_pretty(&hypotheses).unwrap_or_else(|_| "{}".to_string())
        }
        "summary" => {
            let summary = HashMap::from([
                ("checkpoint_count", bt.checkpoint_count().to_string()),
                ("failure_count", bt.failure_count().to_string()),
            ]);
            serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string())
        }
        _ => format!(r#"{{"error": "unknown resource: {}", "available": ["checkpoints", "failures", "hypotheses", "summary"]}}"#, resource),
    }
}

/// 解析 stdin 任務
#[derive(Debug, Deserialize)]
pub struct StdinTask {
    pub task: String,
    pub context: Option<String>,
    pub format: Option<String>,
}

impl Default for StdinTask {
    fn default() -> Self {
        StdinTask {
            task: String::new(),
            context: None,
            format: None,
        }
    }
}
