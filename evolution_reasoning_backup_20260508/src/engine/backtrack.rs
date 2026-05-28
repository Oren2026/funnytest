//! 回溯系統（Backtracking System）
//!
//! v0.8-B 新增：假設驗證與回溯機制。
//!
//! 負責：
//! - 管理決策檢查點（Checkpoint）
//! - 記錄失敗模式（FailurePattern）
//! - 根據失敗生成修正假設（CorrectionHypothesis）
//! - 圖狀態回滾

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Edge, EdgeType, Graph, Node, NodeStatus};

/// 檢查點來源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointReason {
    /// 用戶決策點
    UserDecision,
    /// 階段轉換
    PhaseTransition,
    /// 發散前
    PreDiverge,
    /// 手動建立
    Manual,
    /// 執行前
    PreExecute,
}

impl std::fmt::Display for CheckpointReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointReason::UserDecision => write!(f, "user_decision"),
            CheckpointReason::PhaseTransition => write!(f, "phase_transition"),
            CheckpointReason::PreDiverge => write!(f, "pre_diverge"),
            CheckpointReason::Manual => write!(f, "manual"),
            CheckpointReason::PreExecute => write!(f, "pre_execute"),
        }
    }
}

/// 檢查點
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// 唯一識別碼
    pub id: String,
    /// 掛載的節點 ID
    pub node_id: String,
    /// 圖的 XML snapshot
    pub snapshot: String,
    /// 建立時間
    pub created_at: DateTime<Local>,
    /// 建立原因
    pub reason: CheckpointReason,
    /// 描述
    pub description: String,
}

impl Checkpoint {
    /// 建立新的檢查點
    pub fn new(node_id: String, snapshot: String, reason: CheckpointReason, description: &str) -> Self {
        Checkpoint {
            id: Uuid::new_v4().to_string(),
            node_id,
            snapshot,
            created_at: Local::now(),
            reason,
            description: description.to_string(),
        }
    }
}

/// 失敗模式類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailurePatternType {
    /// 退出碼非零
    ExitNonZero,
    /// 命令不存在
    CommandNotFound,
    /// 執行逾時
    Timeout,
    /// JSON 解析失敗
    ParseError,
    /// 執行失敗（其他）
    ExecutionFailed,
    /// Graph 節點未找到
    NodeNotFound,
}

impl std::fmt::Display for FailurePatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailurePatternType::ExitNonZero => write!(f, "exit_nonzero"),
            FailurePatternType::CommandNotFound => write!(f, "command_not_found"),
            FailurePatternType::Timeout => write!(f, "timeout"),
            FailurePatternType::ParseError => write!(f, "parse_error"),
            FailurePatternType::ExecutionFailed => write!(f, "execution_failed"),
            FailurePatternType::NodeNotFound => write!(f, "node_not_found"),
        }
    }
}

impl FailurePatternType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "exit_nonzero" | "exit_nonzero" => FailurePatternType::ExitNonZero,
            "command_not_found" => FailurePatternType::CommandNotFound,
            "timeout" => FailurePatternType::Timeout,
            "parse_error" | "parse_error" => FailurePatternType::ParseError,
            "execution_failed" => FailurePatternType::ExecutionFailed,
            "node_not_found" => FailurePatternType::NodeNotFound,
            _ => FailurePatternType::ExecutionFailed,
        }
    }
}

/// 失敗模式記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// 唯一識別碼
    pub id: String,
    /// 失敗類型
    pub pattern_type: FailurePatternType,
    /// 相關命令
    pub command: String,
    /// 退出碼
    pub exit_code: Option<i32>,
    /// 錯誤輸出
    pub stderr: String,
    /// 發生時的節點上下文
    pub context_node_id: String,
    /// 發生時間
    pub occurred_at: DateTime<Local>,
}

impl FailurePattern {
    /// 建立失敗記錄
    pub fn new(
        pattern_type: FailurePatternType,
        command: String,
        exit_code: Option<i32>,
        stderr: String,
        context_node_id: String,
    ) -> Self {
        FailurePattern {
            id: Uuid::new_v4().to_string(),
            pattern_type,
            command,
            exit_code,
            stderr,
            context_node_id,
            occurred_at: Local::now(),
        }
    }
}

/// 修正假設
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionHypothesis {
    /// 唯一識別碼
    pub id: String,
    /// 原始失敗描述
    pub original_failure: String,
    /// 假設內容
    pub hypothesis: String,
    /// 建議的修正動作
    pub suggested_action: String,
    /// 信心度（0-1）
    pub confidence: f64,
}

impl CorrectionHypothesis {
    /// 根據失敗模式和命令產生假設
    pub fn generate(pattern: &FailurePattern) -> Vec<Self> {
        let mut hypotheses = Vec::new();

        match pattern.pattern_type {
            FailurePatternType::CommandNotFound => {
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: format!("命令不存在: {}", pattern.command),
                    hypothesis: "命令未安裝或不在 PATH 中".to_string(),
                    suggested_action: format!("使用 which {} 確認命令是否存在，或安裝必要的工具", pattern.command),
                    confidence: 0.9,
                });
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: format!("命令不存在: {}", pattern.command),
                    hypothesis: "命令名稱拼寫錯誤".to_string(),
                    suggested_action: "檢查命令拼寫是否正確，嘗試使用 which 或 type 查找正確名稱".to_string(),
                    confidence: 0.6,
                });
            }
            FailurePatternType::ExitNonZero => {
                let exit = pattern.exit_code.unwrap_or(-1);
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: format!("命令執行失敗，exit code: {}", exit),
                    hypothesis: format!("命令執行失敗，可能是參數錯誤或輸入資料有問題（exit={}）", exit),
                    suggested_action: "檢查命令參數是否正確，確認 stderr 輸出內容".to_string(),
                    confidence: 0.8,
                });
                if exit == 127 {
                    hypotheses.push(CorrectionHypothesis {
                        id: Uuid::new_v4().to_string(),
                        original_failure: format!("命令執行失敗，exit code: {}", exit),
                        hypothesis: "exit code 127 = 命令不存在".to_string(),
                        suggested_action: "確認命令已安裝且在 PATH 中".to_string(),
                        confidence: 0.95,
                    });
                } else if exit == 1 {
                    hypotheses.push(CorrectionHypothesis {
                        id: Uuid::new_v4().to_string(),
                        original_failure: format!("命令執行失敗，exit code: {}", exit),
                        hypothesis: "exit code 1 = 一般錯誤，可能需要檢查輸出".to_string(),
                        suggested_action: "查看 stdout/stderr 輸出來確定錯誤原因".to_string(),
                        confidence: 0.7,
                    });
                }
            }
            FailurePatternType::Timeout => {
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: format!("命令逾時: {}", pattern.command),
                    hypothesis: "命令執行時間超過設定的逾時時間".to_string(),
                    suggested_action: "增加 timeout_ms 參數，或檢查命令是否陷入無限迴圈".to_string(),
                    confidence: 0.95,
                });
            }
            FailurePatternType::ParseError => {
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: "JSON 解析失敗".to_string(),
                    hypothesis: "命令輸出不是有效的 JSON 格式".to_string(),
                    suggested_action: "將 parse_mode 改為 'text'，或使用 jq 等工具處理輸出".to_string(),
                    confidence: 0.9,
                });
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: "JSON 解析失敗".to_string(),
                    hypothesis: "輸出可能包含非 UTF-8 字元或空白字元問題".to_string(),
                    suggested_action: "嘗試先 echo 或 cat 查看原始輸出".to_string(),
                    confidence: 0.5,
                });
            }
            FailurePatternType::ExecutionFailed => {
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: format!("執行失敗: {}", pattern.command),
                    hypothesis: "可能是權限不足或路徑問題".to_string(),
                    suggested_action: "檢查檔案權限、工作目錄和路徑是否正確".to_string(),
                    confidence: 0.7,
                });
            }
            FailurePatternType::NodeNotFound => {
                hypotheses.push(CorrectionHypothesis {
                    id: Uuid::new_v4().to_string(),
                    original_failure: format!("節點不存在: {}", pattern.context_node_id),
                    hypothesis: "嘗試操作的節點 ID 不存在於當前 graph 中".to_string(),
                    suggested_action: "使用 status 工具確認目前的節點結構".to_string(),
                    confidence: 0.95,
                });
            }
        }

        // 如果有 stderr，加入相關假設
        if !pattern.stderr.is_empty() && pattern.pattern_type != FailurePatternType::CommandNotFound {
            let stderr_snippet = pattern.stderr.chars().take(100).collect::<String>();
            hypotheses.push(CorrectionHypothesis {
                id: Uuid::new_v4().to_string(),
                original_failure: format!("stderr: {}", stderr_snippet),
                hypothesis: format!("錯誤訊息提示: {}", stderr_snippet),
                suggested_action: "根據 stderr 內容調整命令或參數".to_string(),
                confidence: 0.85,
            });
        }

        hypotheses
    }
}

/// 回溯管理器
#[derive(Debug, Clone)]
pub struct BacktrackManager {
    /// 檢查點列表
    checkpoints: Vec<Checkpoint>,
    /// 失敗模式歷史
    failure_history: Vec<FailurePattern>,
}

impl Default for BacktrackManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BacktrackManager {
    /// 建立新的回溯管理器
    pub fn new() -> Self {
        BacktrackManager {
            checkpoints: Vec::new(),
            failure_history: Vec::new(),
        }
    }

    /// 建立檢查點
    pub fn create_checkpoint(
        &mut self,
        node_id: String,
        graph: &Graph,
        reason: CheckpointReason,
        description: &str,
    ) -> Checkpoint {
        let snapshot = graph_snapshot_xml(graph);
        let checkpoint = Checkpoint::new(node_id, snapshot, reason, description);
        self.checkpoints.push(checkpoint.clone());
        checkpoint
    }

    /// 取得所有檢查點
    pub fn get_checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    /// 根據 ID 取得檢查點
    pub fn get_checkpoint(&self, id: &str) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    /// 取得最後一個檢查點
    pub fn get_last_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    /// 回溯到指定檢查點
    pub fn restore_from_checkpoint(&self, checkpoint_id: &str) -> Option<Graph> {
        self.checkpoints
            .iter()
            .find(|c| c.id == checkpoint_id)
            .and_then(|c| graph_from_xml(&c.snapshot).ok())
    }

    /// 回溯到最後一個檢查點
    pub fn restore_last_checkpoint(&self) -> Option<Graph> {
        self.checkpoints.last().and_then(|c| graph_from_xml(&c.snapshot).ok())
    }

    /// 記錄失敗模式
    pub fn record_failure(&mut self, failure: FailurePattern) {
        self.failure_history.push(failure);
    }

    /// 從 execute 工具結果自動解析失敗並記錄
    pub fn record_execute_failure(
        &mut self,
        context_node_id: String,
        execute_result_json: &str,
    ) -> Option<FailurePattern> {
        // 嘗試解析 execute 結果
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(execute_result_json) {
            let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);

            if success {
                return None; // 沒有失敗
            }

            let command = result
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let exit_code = result.get("exit_code").and_then(|v| v.as_i64()).map(|i| i as i32);
            let stderr = result.get("stderr").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let error_msg = result.get("error").and_then(|v| v.as_str()).unwrap_or("");

            let pattern_type = if error_msg.contains("無法執行") || error_msg.contains("No such file") {
                FailurePatternType::CommandNotFound
            } else if error_msg.contains("timeout") || error_msg.contains("逾時") {
                FailurePatternType::Timeout
            } else if exit_code == Some(0) && !stderr.is_empty() {
                FailurePatternType::ExitNonZero
            } else if exit_code.map(|c| c != 0).unwrap_or(true) {
                FailurePatternType::ExitNonZero
            } else {
                FailurePatternType::ExecutionFailed
            };

            let failure = FailurePattern::new(
                pattern_type,
                command,
                exit_code,
                stderr,
                context_node_id,
            );

            self.record_failure(failure.clone());
            Some(failure)
        } else {
            None
        }
    }

    /// 根據失敗取得修正假設
    pub fn get_hypotheses_for_last_failure(&self) -> Vec<CorrectionHypothesis> {
        self.failure_history
            .last()
            .map(|f| CorrectionHypothesis::generate(f))
            .unwrap_or_default()
    }

    /// 根據失敗 ID 取得假設
    pub fn get_hypotheses(&self, failure_id: &str) -> Vec<CorrectionHypothesis> {
        self.failure_history
            .iter()
            .find(|f| f.id == failure_id)
            .map(|f| CorrectionHypothesis::generate(f))
            .unwrap_or_default()
    }

    /// 取得失敗歷史
    pub fn get_failure_history(&self) -> &[FailurePattern] {
        &self.failure_history
    }

    /// 取得失敗次數
    pub fn failure_count(&self) -> usize {
        self.failure_history.len()
    }

    /// 取得檢查點數量
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    /// 清除檢查點（記憶體）
    pub fn clear_checkpoints(&mut self) {
        self.checkpoints.clear();
    }

    /// 格式化為字串（給 gemma4 閱讀）
    pub fn format_for_prompt(&self) -> String {
        let mut result = String::new();

        if !self.checkpoints.is_empty() {
            result.push_str("## 檢查點歷史\n");
            for (i, cp) in self.checkpoints.iter().enumerate().rev().take(5) {
                result.push_str(&format!(
                    "{}. [{}] {} - {} ({})\n",
                    i + 1,
                    cp.id.chars().take(8).collect::<String>(),
                    cp.node_id,
                    cp.reason,
                    cp.created_at.format("%H:%M:%S")
                ));
            }
            result.push('\n');
        }

        if !self.failure_history.is_empty() {
            result.push_str("## 失敗歷史\n");
            for (i, f) in self.failure_history.iter().enumerate().rev().take(5) {
                result.push_str(&format!(
                    "{}. [{}] {} - exit={:?} @ {}\n",
                    i + 1,
                    f.id.chars().take(8).collect::<String>(),
                    f.pattern_type,
                    f.exit_code,
                    f.context_node_id
                ));
            }
            result.push('\n');
        }

        if result.is_empty() {
            result.push_str("（尚無檢查點或失敗記錄）\n");
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let cp = Checkpoint::new(
            "node-123".to_string(),
            "<graph>...</graph>".to_string(),
            CheckpointReason::PreDiverge,
            "測試檢查點",
        );

        assert_eq!(cp.node_id, "node-123");
        assert_eq!(cp.description, "測試檢查點");
        assert!(!cp.id.is_empty());
    }

    #[test]
    fn test_backtrack_manager_create_checkpoint() {
        let mut mgr = BacktrackManager::new();
        let mut graph = Graph::new();
        let node = crate::models::Node::new_with("test".to_string(), 1, 0.8, 0.9, 1.0);
        let node_id = node.id.clone();
        graph.add_node(node);

        let cp = mgr.create_checkpoint(node_id, &graph, CheckpointReason::Manual, "測試");
        assert_eq!(mgr.checkpoints.len(), 1);
        assert!(!cp.node_id.is_empty());
    }

    #[test]
    fn test_backtrack_manager_restore() {
        let mut mgr = BacktrackManager::new();
        let mut graph = Graph::new();
        let node = crate::models::Node::new_with("test".to_string(), 1, 0.8, 0.9, 1.0);
        let node_id = node.id.clone();
        graph.add_node(node);

        let cp = mgr.create_checkpoint(node_id.clone(), &graph, CheckpointReason::Manual, "測試");

        // restore
        let restored = mgr.restore_from_checkpoint(&cp.id);
        assert!(restored.is_some());
        let restored_graph = restored.unwrap();
        assert_eq!(restored_graph.node_count(), 1);
    }

    #[test]
    fn test_correction_hypothesis_generate_exit_nonzero() {
        let failure = FailurePattern::new(
            FailurePatternType::ExitNonZero,
            "curl".to_string(),
            Some(127),
            "curl: command not found".to_string(),
            "node-1".to_string(),
        );

        let hypotheses = CorrectionHypothesis::generate(&failure);
        assert!(!hypotheses.is_empty());
        assert!(hypotheses.iter().any(|h| h.hypothesis.contains("exit code 127")));
    }

    #[test]
    fn test_correction_hypothesis_generate_parse_error() {
        let failure = FailurePattern::new(
            FailurePatternType::ParseError,
            "echo".to_string(),
            Some(0),
            "not json output".to_string(),
            "node-1".to_string(),
        );

        let hypotheses = CorrectionHypothesis::generate(&failure);
        assert!(hypotheses.iter().any(|h| h.hypothesis.contains("JSON")));
    }

    #[test]
    fn test_failure_pattern_type_from_str() {
        assert_eq!(FailurePatternType::from_str("exit_nonzero"), FailurePatternType::ExitNonZero);
        assert_eq!(FailurePatternType::from_str("command_not_found"), FailurePatternType::CommandNotFound);
        assert_eq!(FailurePatternType::from_str("timeout"), FailurePatternType::Timeout);
        assert_eq!(FailurePatternType::from_str("unknown"), FailurePatternType::ExecutionFailed);
    }

    #[test]
    fn test_record_execute_failure() {
        let mut mgr = BacktrackManager::new();

        let result_json = serde_json::json!({
            "success": false,
            "exit_code": 127,
            "stderr": "command not found",
            "command": "fake_cmd",
            "error": "無法執行"
        }).to_string();

        let failure = mgr.record_execute_failure("node-abc".to_string(), &result_json);
        assert!(failure.is_some());
        let f = failure.unwrap();
        assert_eq!(f.pattern_type, FailurePatternType::CommandNotFound);
        assert_eq!(mgr.failure_count(), 1);
    }

    #[test]
    fn test_record_execute_failure_no_error() {
        let mut mgr = BacktrackManager::new();

        let result_json = serde_json::json!({
            "success": true,
            "exit_code": 0,
            "stdout": "ok"
        }).to_string();

        let failure = mgr.record_execute_failure("node-abc".to_string(), &result_json);
        assert!(failure.is_none());
        assert_eq!(mgr.failure_count(), 0);
    }

    #[test]
    fn test_get_hypotheses_for_last_failure() {
        let mut mgr = BacktrackManager::new();

        let result_json = serde_json::json!({
            "success": false,
            "exit_code": 1,
            "stderr": "parse error",
            "command": "jq"
        }).to_string();

        mgr.record_execute_failure("node-1".to_string(), &result_json);
        let hypotheses = mgr.get_hypotheses_for_last_failure();
        assert!(!hypotheses.is_empty());
    }

    #[test]
    fn test_format_for_prompt() {
        let mgr = BacktrackManager::new();
        let formatted = mgr.format_for_prompt();
        assert!(formatted.contains("尚無檢查點或失敗記錄"));
    }
}

// ============================================================================
// Graph XML serialization for checkpoints (local, no external dependency)
// ============================================================================

/// 將 Graph 序列化为简洁的 XML（用于 checkpoint snapshot）
fn graph_snapshot_xml(graph: &Graph) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graph>\n");
    xml.push_str("  <nodes>\n");
    for node in graph.get_all_nodes() {
        xml.push_str(&format!(
            "    <node id=\"{}\" step=\"{}\" weight=\"{:.4}\" confidence=\"{:.4}\" status=\"{:?}\">\n",
            node.id, node.step, node.weight, node.confidence, node.status
        ));
        xml.push_str(&format!("      <content><![CDATA[{}]]></content>\n", node.content));
        xml.push_str("    </node>\n");
    }
    xml.push_str("  </nodes>\n");
    xml.push_str("  <edges>\n");
    for edge in graph.get_all_edges() {
        xml.push_str(&format!(
            "    <edge id=\"{}\" from=\"{}\" to=\"{}\" type=\"{:?}\" weight=\"{:.4}\" />\n",
            edge.id, edge.from, edge.to, edge.edge_type, edge.weight
        ));
    }
    xml.push_str("  </edges>\n");
    xml.push_str(&format!("  <complexity>{:.4}</complexity>\n", graph.total_complexity()));
    xml.push_str("</graph>\n");
    xml
}

/// 从 XML 反序列化为 Graph
fn graph_from_xml(xml: &str) -> Result<Graph, String> {
    let mut graph = Graph::new();

    // 解析节点
    let node_section = xml.split("<nodes>").nth(1)
        .and_then(|s| s.split("</nodes>").next())
        .ok_or("Invalid XML: missing nodes section")?;

    for node_chunk in node_section.split("<node ").skip(1) {
        let id = extract_xml_attr(node_chunk, "id")
            .ok_or("Invalid XML: missing node id")?;
        let step: i32 = extract_xml_attr(node_chunk, "step")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let weight: f64 = extract_xml_attr(node_chunk, "weight")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        let confidence: f64 = extract_xml_attr(node_chunk, "confidence")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);

        let content = node_chunk
            .split("<content><![CDATA[").nth(1)
            .and_then(|s| s.split("]]></content>").next())
            .unwrap_or("")
            .to_string();

        let status = extract_xml_attr(node_chunk, "status")
            .and_then(|s| parse_node_status(&s))
            .unwrap_or(NodeStatus::Active);

        let mut node = Node::new_with(content, step, weight, confidence, 0.0);
        node.id = id;
        node.status = status;
        graph.add_node(node);
    }

    // 解析边
    let edge_section = xml.split("<edges>").nth(1)
        .and_then(|s| s.split("</edges>").next())
        .unwrap_or("");

    for edge_chunk in edge_section.split("<edge ").skip(1) {
        let from = extract_xml_attr(edge_chunk, "from")
            .ok_or("Invalid XML: missing edge from")?;
        let to = extract_xml_attr(edge_chunk, "to")
            .ok_or("Invalid XML: missing edge to")?;
        let weight: f64 = extract_xml_attr(edge_chunk, "weight")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        let edge_type = extract_xml_attr(edge_chunk, "type")
            .and_then(|s| parse_edge_type(&s))
            .unwrap_or(EdgeType::Reasoning);

        let edge = Edge::new_with_weight(from, to, edge_type, weight);
        graph.add_edge(edge);
    }

    Ok(graph)
}

/// 从 XML 属性字符串中提取值
fn extract_xml_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    xml.find(&pattern)
        .and_then(|pos| {
            let start = pos + pattern.len();
            xml[start..].find('"').map(|end| xml[start..start + end].to_string())
        })
}

/// 解析 NodeStatus
fn parse_node_status(s: &str) -> Option<NodeStatus> {
    match s {
        "Draft" => Some(NodeStatus::Draft),
        "Active" => Some(NodeStatus::Active),
        "Pruned" => Some(NodeStatus::Pruned),
        "Locked" => Some(NodeStatus::Locked),
        _ => None,
    }
}

/// 解析 EdgeType
fn parse_edge_type(s: &str) -> Option<EdgeType> {
    match s {
        "Reasoning" => Some(EdgeType::Reasoning),
        "Constraint" => Some(EdgeType::Constraint),
        "Divergence" => Some(EdgeType::Divergence),
        _ => None,
    }
}
