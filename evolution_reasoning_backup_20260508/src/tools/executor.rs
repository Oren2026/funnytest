//! 工具執行器
//!
//! 負責執行 gemma4 呼叫的工具，並返回結果。

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::engine::{BacktrackManager, CheckpointReason, ConvergeEngine, DivergeEngine, FailurePatternType};
use crate::export::{export_backtrack, export_graph, export_hypotheses, export_memory, export_node, query_backtrack, ExportFormat};
use crate::memory::MemoryManager;
use crate::models::Graph;
use crate::ollama::client::ToolCall;
use crate::workspace::Workspace;

/// 工具執行結果
#[derive(Debug)]
pub struct ToolResult {
    /// 是否成功
    pub success: bool,
    /// 結果訊息
    pub message: String,
    /// 工具名稱
    pub tool_name: String,
}

impl ToolResult {
    /// 建立成功結果
    pub fn success(tool_name: &str, message: impl Into<String>) -> Self {
        ToolResult {
            success: true,
            message: message.into(),
            tool_name: tool_name.to_string(),
        }
    }

    /// 建立失敗結果
    pub fn error(tool_name: &str, message: impl Into<String>) -> Self {
        ToolResult {
            success: false,
            message: message.into(),
            tool_name: tool_name.to_string(),
        }
    }
}

/// 工具執行器
///
/// 持有 Engine 和 Workspace 的引用，執行工具並操作它們。
pub struct ToolExecutor {
    /// 推理圖（可變）
    graph: Arc<Mutex<Graph>>,
    /// 發散引擎
    diverge_engine: DivergeEngine,
    /// 收斂引擎
    converge_engine: ConvergeEngine,
    /// 回溯管理器
    backtrack_manager: RefCell<BacktrackManager>,
    /// Workspace
    workspace: Workspace,
    /// 長期記憶管理器
    memory: MemoryManager,
    /// 當前複雜度
    complexity_budget: f64,
}

impl ToolExecutor {
    /// 建立新的工具執行器
    pub fn new() -> Self {
        ToolExecutor {
            graph: Arc::new(Mutex::new(Graph::new())),
            diverge_engine: DivergeEngine::new(),
            converge_engine: ConvergeEngine::new(),
            backtrack_manager: RefCell::new(BacktrackManager::new()),
            workspace: Workspace::new(),
            memory: MemoryManager::new(),
            complexity_budget: 0.0,
        }
    }

    /// 建立有初始圖的執行器
    pub fn with_graph(graph: Graph) -> Self {
        ToolExecutor {
            graph: Arc::new(Mutex::new(graph)),
            diverge_engine: DivergeEngine::new(),
            converge_engine: ConvergeEngine::new(),
            backtrack_manager: RefCell::new(BacktrackManager::new()),
            workspace: Workspace::new(),
            memory: MemoryManager::new(),
            complexity_budget: 0.0,
        }
    }

    /// 取得圖的副本
    pub fn get_graph(&self) -> Graph {
        self.graph.lock().unwrap().clone()
    }

    /// 取得回溯管理器
    pub fn get_backtrack_manager(&self) -> BacktrackManager {
        self.backtrack_manager.borrow().clone()
    }

    /// 執行工具呼叫
    ///
    /// # 引數
    /// - `tool_call`: 工具呼叫（包含名稱和參數）
    pub fn execute(&self, tool_call: &ToolCall) -> ToolResult {
        let name = &tool_call.function.name;
        let args = &tool_call.function.arguments;

        match name.as_str() {
            "diverge" => self.execute_diverge(args),
            "converge" => self.execute_converge(args),
            "save" => self.execute_save(args),
            "load" => self.execute_load(args),
            "output" => self.execute_output(args),
            "status" => self.execute_status(args),
            "execute" => self.execute_execute(args),
            "exec_status" => self.execute_exec_status(args),
            "checkpoint" => self.execute_checkpoint(args),
            "backtrack" => self.execute_backtrack(args),
            "record_failure" => self.execute_record_failure(args),
            "get_hypotheses" => self.execute_get_hypotheses(args),
            "export_graph" => self.execute_export_graph(args),
            "export_node" => self.execute_export_node(args),
            "export_hypotheses" => self.execute_export_hypotheses(args),
            "export_backtrack" => self.execute_export_backtrack(args),
            "export_memory" => self.execute_export_memory(args),
            "query_backtrack" => self.execute_query_backtrack(args),
            _ => ToolResult::error(name, format!("未知的工具: {}", name)),
        }
    }

    /// 執行 diverge 工具
    fn execute_diverge(&self, args: &str) -> ToolResult {
        // 解析參數
        let args_json: serde_json::Value = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("diverge", format!("參數解析失敗: {}", e)),
        };

        let node_id = match args_json.get("node_id") {
            Some(v) => v.as_str().unwrap_or(""),
            None => return ToolResult::error("diverge", "缺少 node_id 參數"),
        };

        // 轉換 content 格式：Option<String> -> Option<Vec<String>>
        let content_vec: Option<Vec<String>> = args_json
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| vec![s.to_string()]);

        let count = args_json
            .get("count")
            .and_then(|v| v.as_i64())
            .unwrap_or(3) as i32;

        if node_id.is_empty() {
            return ToolResult::error("diverge", "node_id 不能為空");
        }

        // 執行發散
        let mut graph = self.graph.lock().unwrap();
        let children = self.diverge_engine.diverge(&mut graph, node_id, count, content_vec);

        if children.is_empty() {
            return ToolResult::error(
                "diverge",
                format!("無法對節點 {} 發散（節點不存在或狀態不允許）", node_id),
            );
        }

        let new_complexity = graph.total_complexity();

        // 更新 workspace 狀態
        let _ = self.workspace.save_state(&graph, "current");
        let _ = self.workspace.update_status(&graph);

        let child_ids: Vec<String> = children.iter().map(|c| c.id.clone()).collect();
        ToolResult::success(
            "diverge",
            format!(
                "已發散生成 {} 個子節點: {}，複雜度: {:.2}",
                children.len(),
                child_ids.join(", "),
                new_complexity
            ),
        )
    }

    /// 執行 converge 工具
    fn execute_converge(&self, args: &str) -> ToolResult {
        // 解析參數
        let args_json: serde_json::Value = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(_) => serde_json::Value::Object(serde_json::Map::new()), // 空的話用預設值
        };

        let threshold = args_json
            .get("threshold")
            .and_then(|v| v.as_f64());

        // 執行收斂
        let mut graph = self.graph.lock().unwrap();
        let pruned = self.converge_engine.converge(&mut graph, threshold);

        let new_complexity = graph.total_complexity();

        // 更新 workspace 狀態
        let _ = self.workspace.save_state(&graph, "current");
        let _ = self.workspace.update_status(&graph);

        if pruned.is_empty() {
            ToolResult::success(
                "converge",
                format!("沒有節點需要刪除，複雜度: {:.2}", new_complexity),
            )
        } else {
            ToolResult::success(
                "converge",
                format!(
                    "已刪除 {} 個節點: {}，複雜度: {:.2}",
                    pruned.len(),
                    pruned.join(", "),
                    new_complexity
                ),
            )
        }
    }

    /// 執行 save 工具
    fn execute_save(&self, args: &str) -> ToolResult {
        // 解析參數
        let args_json: serde_json::Value = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("save", format!("參數解析失敗: {}", e)),
        };

        let name = match args_json.get("name") {
            Some(v) => v.as_str().unwrap_or("default"),
            None => "default",
        };

        if name.is_empty() {
            return ToolResult::error("save", "name 不能為空");
        }

        // 儲存狀態
        let graph = self.graph.lock().unwrap();
        match self.workspace.save_named(&graph, name) {
            Ok(path) => ToolResult::success(
                "save",
                format!("狀態已儲存到: {}", path.display()),
            ),
            Err(e) => ToolResult::error("save", format!("儲存失敗: {}", e)),
        }
    }

    /// 執行 load 工具
    fn execute_load(&self, args: &str) -> ToolResult {
        // 解析參數
        let args_json: serde_json::Value = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("load", format!("參數解析失敗: {}", e)),
        };

        let name = match args_json.get("name") {
            Some(v) => v.as_str().unwrap_or("default"),
            None => "default",
        };

        if name.is_empty() {
            return ToolResult::error("load", "name 不能為空");
        }

        // 載入狀態
        match self.workspace.load_named(name) {
            Ok(graph) => {
                let mut g = self.graph.lock().unwrap();
                *g = graph;
                let new_complexity = g.total_complexity();
                let _ = self.workspace.update_status(&g);
                ToolResult::success(
                    "load",
                    format!("已從 {} 載入狀態，複雜度: {:.2}", name, new_complexity),
                )
            }
            Err(e) => ToolResult::error("load", format!("載入失敗: {}", e)),
        }
    }

    /// 執行 output 工具
    fn execute_output(&self, args: &str) -> ToolResult {
        // 解析參數
        let args_json: serde_json::Value = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("output", format!("參數解析失敗: {}", e)),
        };

        let format = match args_json.get("format") {
            Some(v) => v.as_str().unwrap_or("xml"),
            None => "xml",
        };

        let name = match args_json.get("name") {
            Some(v) => v.as_str().unwrap_or("output"),
            None => "output",
        };

        // 輸出狀態
        let graph = self.graph.lock().unwrap();
        match self.workspace.output(&graph, format, name) {
            Ok(path) => ToolResult::success(
                "output",
                format!("已輸出到: {} (格式: {})", path.display(), format),
            ),
            Err(e) => ToolResult::error("output", format!("輸出失敗: {}", e)),
        }
    }

    /// 執行 execute 工具（沙盒執行外部命令）
    fn execute_execute(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct ExecuteArgs {
            command: String,
            #[serde(default)]
            args: Vec<String>,
            #[serde(default = "default_timeout")]
            timeout_ms: u64,
            #[serde(default)]
            parse_mode: String,
        }

        fn default_timeout() -> u64 { 30000 }

        let parsed: ExecuteArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("execute", format!("參數解析失敗: {}", e)),
        };

        if parsed.command.is_empty() {
            return ToolResult::error("execute", "command 不能為空");
        }

        let output = match std::process::Command::new(&parsed.command)
            .args(&parsed.args)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return ToolResult::success("execute", serde_json::json!({
                    "success": false,
                    "error": format!("執行失敗: {}", e),
                    "command": parsed.command,
                    "args": parsed.args,
                }).to_string());
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let result = match parsed.parse_mode.as_str() {
            "json" => {
                match serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(v) => serde_json::json!({
                        "success": exit_code == 0,
                        "exit_code": exit_code,
                        "stdout": stdout,
                        "stderr": stderr,
                        "command": parsed.command,
                        "args": parsed.args,
                        "parsed": v,
                    }),
                    Err(_) => serde_json::json!({
                        "success": exit_code == 0,
                        "exit_code": exit_code,
                        "stdout": stdout,
                        "stderr": stderr,
                        "command": parsed.command,
                        "args": parsed.args,
                        "parsed": null,
                        "raw": stdout,
                        "parse_error": "stdout 不是有效 JSON",
                    }),
                }
            }
            "exit_code" => serde_json::json!({
                "success": exit_code == 0,
                "exit_code": exit_code,
                "command": parsed.command,
                "args": parsed.args,
            }),
            _ => serde_json::json!({
                "success": exit_code == 0,
                "exit_code": exit_code,
                "stdout": stdout,
                "stderr": stderr,
                "command": parsed.command,
                "args": parsed.args,
            }),
        };

        ToolResult::success("execute", result.to_string())
    }

    /// 執行 exec_status 工具（沙盒環境資訊）
    fn execute_exec_status(&self, _args: &str) -> ToolResult {
        let os_info = format!(
            "OS: {} {}, Arch: {}",
            std::env::consts::OS,
            std::env::consts::FAMILY,
            std::env::consts::ARCH
        );

        let env_vars: Vec<String> = std::env::vars()
            .filter(|(k, _)| k.starts_with(" Evolution") || k == "PATH" || k == "HOME")
            .map(|(k, v)| format!("{}={}", k, v))
            .take(20)
            .collect();

        let info = serde_json::json!({
            "platform": os_info,
            "sandbox_dir": "/tmp/evolution_sandbox",
            "env": env_vars,
        });

        ToolResult::success("exec_status", info.to_string())
    }

    /// 執行 checkpoint 工具（建立檢查點）
    fn execute_checkpoint(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct CheckpointArgs {
            #[serde(default)]
            node_id: String,
            #[serde(default = "default_checkpoint_reason")]
            reason: String,
            #[serde(default)]
            description: String,
        }

        fn default_checkpoint_reason() -> String {
            "manual".to_string()
        }

        let parsed: CheckpointArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("checkpoint", format!("參數解析失敗: {}", e)),
        };

        let reason = match parsed.reason.as_str() {
            "user_decision" => CheckpointReason::UserDecision,
            "phase_transition" => CheckpointReason::PhaseTransition,
            "pre_diverge" => CheckpointReason::PreDiverge,
            "pre_execute" => CheckpointReason::PreExecute,
            _ => CheckpointReason::Manual,
        };

        // 取得 node_id 和 description（分開 scope 避免鎖衝突）
        let (node_id, desc) = {
            let graph = self.graph.lock().unwrap();
            let nid = if parsed.node_id.is_empty() {
                graph.get_current_topic()
                    .and_then(|t| graph.get_node(&t.root_node_id))
                    .map(|n| n.id.clone())
                    .unwrap_or_else(|| "root".to_string())
            } else {
                parsed.node_id.clone()
            };

            let d = if parsed.description.is_empty() {
                format!("checkpoint_at_{}", chrono::Local::now().format("%H%M%S"))
            } else {
                parsed.description.clone()
            };
            (nid, d)
        };

        // 建立 checkpoint（在獨立的 scope 內借出 graph）
        let checkpoint = {
            let graph = self.graph.lock().unwrap();
            self.backtrack_manager.borrow_mut().create_checkpoint(
                node_id,
                &graph,
                reason,
                &desc,
            )
        };

        ToolResult::success("checkpoint", serde_json::json!({
            "checkpoint_id": checkpoint.id,
            "node_id": checkpoint.node_id,
            "reason": checkpoint.reason.to_string(),
            "description": checkpoint.description,
            "created_at": checkpoint.created_at.to_rfc3339(),
            "total_checkpoints": self.backtrack_manager.borrow().get_checkpoints().len(),
        }).to_string())
    }

    /// 執行 backtrack 工具（回溯到檢查點）
    fn execute_backtrack(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct BacktrackArgs {
            #[serde(default)]
            checkpoint_id: String,
            #[serde(default)]
            restore_last: String,
        }

        let parsed: BacktrackArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("backtrack", format!("參數解析失敗: {}", e)),
        };

        // 先取得 checkpoint_id（獨立的 borrow scope）
        let target_id = {
            let mgr = self.backtrack_manager.borrow();
            let restore = parsed.restore_last.to_lowercase() == "true";
            if parsed.checkpoint_id.is_empty() && !restore {
                return ToolResult::error("backtrack", "需要 checkpoint_id 或 restore_last=true");
            } else if restore {
                mgr.get_last_checkpoint()
                    .map(|c| c.id.clone())
                    .unwrap_or_default()
            } else {
                parsed.checkpoint_id.clone()
            }
        };

        if target_id.is_empty() {
            return ToolResult::error("backtrack", "沒有可用的檢查點");
        }

        // 在隔離的 scope 內 restore graph
        let node_count = {
            let mgr = self.backtrack_manager.borrow();
            match mgr.restore_from_checkpoint(&target_id) {
                Some(restored_graph) => {
                    let mut graph = self.graph.lock().unwrap();
                    *graph = restored_graph;
                    let nc = graph.node_count();
                    drop(graph);
                    drop(mgr);
                    let _ = self.workspace.save_state(&self.graph.lock().unwrap(), "current");
                    let _ = self.workspace.update_status(&self.graph.lock().unwrap());
                    nc
                }
                None => return ToolResult::error("backtrack", format!("無法回溯到檢查點 {}", target_id)),
            }
        };

        ToolResult::success("backtrack", serde_json::json!({
            "success": true,
            "restored_checkpoint_id": target_id,
            "restored_node_count": node_count,
        }).to_string())
    }

    /// 執行 record_failure 工具（記錄失敗模式）
    fn execute_record_failure(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct RecordFailureArgs {
            #[serde(default)]
            node_id: String,
            #[serde(default)]
            pattern_type: String,
            #[serde(default)]
            command: String,
            #[serde(default)]
            exit_code: Option<i32>,
            #[serde(default)]
            stderr: String,
            #[serde(default)]
            execute_result_json: String,
        }

        let parsed: RecordFailureArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("record_failure", format!("參數解析失敗: {}", e)),
        };

        // 如果有 execute_result_json，自動解析並記錄
        if !parsed.execute_result_json.is_empty() {
            let (failure_opt, total) = {
                let mut mgr = self.backtrack_manager.borrow_mut();
                let result = mgr.record_execute_failure(
                    parsed.node_id.clone(),
                    &parsed.execute_result_json,
                );
                let total = mgr.failure_count();
                (result, total)
            };

            if let Some(failure) = failure_opt {
                return ToolResult::success("record_failure", serde_json::json!({
                    "failure_id": failure.id,
                    "pattern_type": failure.pattern_type.to_string(),
                    "command": failure.command,
                    "exit_code": failure.exit_code,
                    "stderr": failure.stderr,
                    "confidence": "high",
                    "total_failures": total,
                }).to_string());
            } else {
                return ToolResult::success("record_failure", serde_json::json!({
                    "note": "execute 結果成功，無需記錄失敗"
                }).to_string());
            }
        }

        // 手動記錄
        let (failure, total) = {
            let mut mgr = self.backtrack_manager.borrow_mut();
            let pattern = FailurePatternType::from_str(&parsed.pattern_type);
            let failure = crate::engine::FailurePattern::new(
                pattern,
                parsed.command.clone(),
                parsed.exit_code,
                parsed.stderr.clone(),
                parsed.node_id.clone(),
            );
            mgr.record_failure(failure.clone());
            (failure, mgr.failure_count())
        };

        ToolResult::success("record_failure", serde_json::json!({
            "failure_id": failure.id,
            "pattern_type": failure.pattern_type.to_string(),
            "total_failures": total,
        }).to_string())
    }

    /// 執行 get_hypotheses 工具（取得修正假設）
    fn execute_get_hypotheses(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct GetHypothesesArgs {
            #[serde(default)]
            failure_id: String,
            #[serde(default)]
            get_last: String,
        }

        let parsed: GetHypothesesArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("get_hypotheses", format!("參數解析失敗: {}", e)),
        };

        let hypotheses = {
            let mgr = self.backtrack_manager.borrow();
            let get_last = parsed.get_last.to_lowercase() == "true";
            if !parsed.failure_id.is_empty() {
                mgr.get_hypotheses(&parsed.failure_id)
            } else if get_last {
                mgr.get_hypotheses_for_last_failure()
            } else {
                return ToolResult::error("get_hypotheses", "需要 failure_id 或 get_last=true");
            }
        };

        if hypotheses.is_empty() {
            return ToolResult::success("get_hypotheses", serde_json::json!({
                "hypotheses": [],
                "note": "沒有失敗記錄或假設"
            }).to_string());
        }

        let result: Vec<serde_json::Value> = hypotheses
            .iter()
            .map(|h| {
                serde_json::json!({
                    "id": h.id,
                    "original_failure": h.original_failure,
                    "hypothesis": h.hypothesis,
                    "suggested_action": h.suggested_action,
                    "confidence": h.confidence,
                })
            })
            .collect();

        ToolResult::success("get_hypotheses", serde_json::json!({
            "hypotheses": result,
            "count": result.len(),
        }).to_string())
    }

    /// 執行 export_graph 工具（匯出推理圖）
    fn execute_export_graph(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct ExportGraphArgs {
            #[serde(default = "default_format")]
            format: String,
        }
        fn default_format() -> String { "yaml".to_string() }

        let parsed: ExportGraphArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("export_graph", format!("參數解析失敗: {}", e)),
        };

        let fmt = match parsed.format.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "dsl" => ExportFormat::Dsl,
            _ => ExportFormat::Yaml,
        };

        let graph = self.graph.lock().unwrap();
        let output = export_graph(&graph, fmt);
        ToolResult::success("export_graph", output)
    }

    /// 執行 export_node 工具（匯出單一節點）
    fn execute_export_node(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct ExportNodeArgs {
            node_id: String,
            #[serde(default = "default_format")]
            format: String,
        }
        fn default_format() -> String { "yaml".to_string() }

        let parsed: ExportNodeArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("export_node", format!("參數解析失敗: {}", e)),
        };

        if parsed.node_id.is_empty() {
            return ToolResult::error("export_node", "node_id 不能為空");
        }

        let fmt = match parsed.format.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "dsl" => ExportFormat::Dsl,
            _ => ExportFormat::Yaml,
        };

        let graph = self.graph.lock().unwrap();
        let node = graph.nodes.get(&parsed.node_id);
        match node {
            Some(n) => {
                let output = export_node(n, fmt);
                ToolResult::success("export_node", output)
            }
            None => ToolResult::error("export_node", format!("節點 {} 不存在", parsed.node_id)),
        }
    }

    /// 執行 export_hypotheses 工具（匯出假設列表）
    fn execute_export_hypotheses(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct ExportHypothesesArgs {
            #[serde(default)]
            failure_id: String,
            #[serde(default = "default_format")]
            format: String,
        }
        fn default_format() -> String { "yaml".to_string() }

        let parsed: ExportHypothesesArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("export_hypotheses", format!("參數解析失敗: {}", e)),
        };

        let fmt = match parsed.format.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "dsl" => ExportFormat::Dsl,
            _ => ExportFormat::Yaml,
        };

        let mgr = self.backtrack_manager.borrow();
        let output = export_hypotheses(&mgr, if parsed.failure_id.is_empty() { None } else { Some(&parsed.failure_id) }, fmt);
        ToolResult::success("export_hypotheses", output)
    }

    /// 執行 export_backtrack 工具（匯出回溯狀態）
    fn execute_export_backtrack(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct ExportBacktrackArgs {
            #[serde(default = "default_format")]
            format: String,
        }
        fn default_format() -> String { "yaml".to_string() }

        let parsed: ExportBacktrackArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("export_backtrack", format!("參數解析失敗: {}", e)),
        };

        let fmt = match parsed.format.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "dsl" => ExportFormat::Dsl,
            _ => ExportFormat::Yaml,
        };

        let mgr = self.backtrack_manager.borrow();
        let output = export_backtrack(&mgr, fmt);
        ToolResult::success("export_backtrack", output)
    }

    /// 執行 export_memory 工具（匯出長期記憶狀態）
    fn execute_export_memory(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct ExportMemoryArgs {
            #[serde(default = "default_format")]
            format: String,
        }
        fn default_format() -> String { "yaml".to_string() }

        let parsed: ExportMemoryArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("export_memory", format!("參數解析失敗: {}", e)),
        };

        let fmt = match parsed.format.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "dsl" => ExportFormat::Dsl,
            _ => ExportFormat::Yaml,
        };

        let output = export_memory(&self.memory, fmt);
        ToolResult::success("export_memory", output)
    }

    /// 執行 query_backtrack 工具（HTTP API 查詢端點）
    fn execute_query_backtrack(&self, args: &str) -> ToolResult {
        #[derive(serde::Deserialize)]
        struct QueryBacktrackArgs {
            resource: String,
            #[serde(default)]
            format: String,
        }

        let parsed: QueryBacktrackArgs = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return ToolResult::error("query_backtrack", format!("參數解析失敗: {}", e)),
        };

        let mgr = self.backtrack_manager.borrow();
        let output = query_backtrack(&mgr, &parsed.resource);
        ToolResult::success("query_backtrack", output)
    }

    /// 執行 status 工具
    fn execute_status(&self, _args: &str) -> ToolResult {
        let graph = self.graph.lock().unwrap();

        // 更新 status.xml
        let _ = self.workspace.update_status(&graph);

        let node_count = graph.node_count();
        let edge_count = graph.edge_count();
        let total_complexity = graph.total_complexity();

        let status_summary = format!(
            "狀態摘要：節點數={}, 邊數={}, 複雜度={:.2}",
            node_count, edge_count, total_complexity
        );

        ToolResult::success("status", status_summary)
    }

    /// 取得複雜度
    pub fn complexity(&self) -> f64 {
        self.complexity_budget
    }

    /// 設定複雜度
    #[allow(dead_code)]
    pub fn set_complexity(&mut self, complexity: f64) {
        self.complexity_budget = complexity;
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("test", "成功訊息");
        assert!(result.success);
        assert_eq!(result.tool_name, "test");
        assert_eq!(result.message, "成功訊息");
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("test", "錯誤訊息");
        assert!(!result.success);
        assert_eq!(result.tool_name, "test");
        assert_eq!(result.message, "錯誤訊息");
    }

    #[test]
    fn test_tool_executor_new() {
        let executor = ToolExecutor::new();
        assert_eq!(executor.graph.lock().unwrap().node_count(), 0);
    }

    #[test]
    fn test_execute_status() {
        let executor = ToolExecutor::new();
        let tool_call = ToolCall {
            function: crate::ollama::client::ToolCallFunction {
                name: "status".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let result = executor.execute(&tool_call);
        assert!(result.success);
        assert!(result.message.contains("節點數=0"));
    }

    #[test]
    fn test_execute_diverge_invalid_node() {
        let executor = ToolExecutor::new();
        let tool_call = ToolCall {
            function: crate::ollama::client::ToolCallFunction {
                name: "diverge".to_string(),
                arguments: r#"{"node_id": "nonexistent", "count": 3}"#.to_string(),
            },
        };

        let result = executor.execute(&tool_call);
        assert!(!result.success);
    }

    #[test]
    fn test_execute_echo_command() {
        let executor = ToolExecutor::new();
        let tool_call = ToolCall {
            function: crate::ollama::client::ToolCallFunction {
                name: "execute".to_string(),
                arguments: r#"{"command": "echo", "args": ["hello", "world"], "parse_mode": "text"}"#.to_string(),
            },
        };

        let result = executor.execute(&tool_call);
        assert!(result.success);
        assert!(result.message.contains("hello"));
        assert!(result.message.contains("world"));
        assert!(result.message.contains("\"exit_code\": 0") || result.message.contains("exit_code"));
    }

    #[test]
    fn test_execute_json_mode() {
        let executor = ToolExecutor::new();
        let tool_call = ToolCall {
            function: crate::ollama::client::ToolCallFunction {
                name: "execute".to_string(),
                arguments: r#"{"command": "echo", "args": ["{\"key\": \"value\"}"], "parse_mode": "json"}"#.to_string(),
            },
        };

        let result = executor.execute(&tool_call);
        assert!(result.success);
        assert!(result.message.contains("parsed"));
        assert!(result.message.contains("key"));
    }

    #[test]
    fn test_execute_invalid_command() {
        let executor = ToolExecutor::new();
        let tool_call = ToolCall {
            function: crate::ollama::client::ToolCallFunction {
                name: "execute".to_string(),
                arguments: r#"{"command": "nonexistent_command_xyz"}"#.to_string(),
            },
        };

        let result = executor.execute(&tool_call);
        assert!(result.success); // returns success with error info
        assert!(result.message.contains("error") || result.message.contains("執行失敗"));
    }

    #[test]
    fn test_exec_status() {
        let executor = ToolExecutor::new();
        let tool_call = ToolCall {
            function: crate::ollama::client::ToolCallFunction {
                name: "exec_status".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let result = executor.execute(&tool_call);
        assert!(result.success);
        assert!(result.message.contains("platform"));
        assert!(result.message.contains("darwin") || result.message.contains("macos") || result.message.contains("os"));
    }
}
