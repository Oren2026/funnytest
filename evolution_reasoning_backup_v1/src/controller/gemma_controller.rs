//! Gemma 控制器
//!
//! 負責協調 gemma4 模型與 Engine 的互動。

use std::sync::{Arc, Mutex};

use crate::engine::ConstraintManager;
use crate::memory::MemoryManager;
use crate::models::{Graph, NodeStatus};
use crate::summarizer::NodeUpdateParser;
use crate::ollama::client::{Message, OllamaClient, Tool, ToolCall, ToolMessage};
use crate::tools::{ToolExecutor, ToolRegistry};
use crate::workspace::Workspace;

/// Gemma 控制器
///
/// 整合 gemma4 模型、工具執行器、Workspace、約束條件管理器和長期記憶系統。
pub struct GemmaController {
    /// Ollama 客戶端
    ollama: OllamaClient,
    /// 工具執行器
    executor: Arc<Mutex<ToolExecutor>>,
    /// 工具註冊表
    registry: ToolRegistry,
    /// Workspace
    workspace: Workspace,
    /// 對話歷史
    messages: Vec<Message>,
    /// 當前任務
    task: String,
    /// 當前模式（發散/收斂）
    mode: ControllerMode,
    /// 當前提問階段（v0.4 新增）
    phase: QuestionPhase,
    /// 前一提問階段（用於檢測階段轉換）
    previous_phase: Option<QuestionPhase>,
    /// 約束條件管理器（v0.5 新增）
    constraint_manager: ConstraintManager,
    /// 長期記憶管理器（v0.5 新增）
    memory_manager: MemoryManager,
}

impl std::fmt::Debug for GemmaController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GemmaController")
            .field("task", &self.task)
            .field("mode", &self.mode)
            .field("messages_count", &self.messages.len())
            .field("registry", &self.registry)
            .finish()
    }
}

/// 控制器模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerMode {
    /// 發散模式（預設）
    Diverge,
    /// 收斂模式
    Converge,
}

/// 提問階段（v0.4 新增）
///
/// 根據節點數量決定當前應該處於哪個階段，
/// 不同階段有不同的互動策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionPhase {
    /// 探索期：節點 < 3，大量提問
    Exploration,
    /// 發展期：節點 3-10，開始建議但仍問問題確認
    Development,
    /// 成熟期：節點 > 10，進入發散/收斂模式
    Mature,
}

impl QuestionPhase {
    /// 根據節點數量取得當前階段
    pub fn from_node_count(count: usize) -> Self {
        if count < 3 {
            QuestionPhase::Exploration
        } else if count <= 10 {
            QuestionPhase::Development
        } else {
            QuestionPhase::Mature
        }
    }

    /// 取得階段名稱（中文）
    pub fn name(&self) -> &'static str {
        match self {
            QuestionPhase::Exploration => "探索期",
            QuestionPhase::Development => "發展期",
            QuestionPhase::Mature => "成熟期",
        }
    }

    /// 取得階段轉換提示訊息
    ///
    /// 當階段改變時，回傳要告訴 gemma4 的訊息。
    pub fn transition_message(&self) -> &'static str {
        match self {
            QuestionPhase::Exploration => {
                "你正處於「探索期」——節點數量少（< 3），你的主要任務是透過提問了解用戶的興趣、價值觀、經驗。不要急於給出框架或建議，每個回覆至少包含一個問題。"
            }
            QuestionPhase::Development => {
                "探索期已結束。現在進入「發展期」，你可以開始提出建議，但每次建議前仍需確認用戶想法。繼續透過提問深化理解。"
            }
            QuestionPhase::Mature => {
                "發展期已結束。現在進入「成熟期」，你可以在確認用戶想法後，主導發散和收斂操作，開始形成具體的建議框架。"
            }
        }
    }
}

impl Default for ControllerMode {
    fn default() -> Self {
        ControllerMode::Diverge
    }
}

impl GemmaController {
    /// 建立新的 Gemma 控制器
    pub fn new(model: impl Into<String>) -> Self {
        // 載入已存在的約束條件
        let workspace = Workspace::new();
        let (_, constraints_xml) = workspace.read_constraints();
        let constraint_manager = if constraints_xml.is_empty() {
            ConstraintManager::new()
        } else {
            ConstraintManager::from_xml(&constraints_xml)
        };

        // 初始化長期記憶管理器
        let memory_manager = MemoryManager::new();

        GemmaController {
            ollama: OllamaClient::new(model),
            executor: Arc::new(Mutex::new(ToolExecutor::new())),
            registry: ToolRegistry::new(),
            workspace,
            messages: Vec::new(),
            task: String::new(),
            mode: ControllerMode::Diverge,
            phase: QuestionPhase::Exploration,
            previous_phase: None,
            constraint_manager,
            memory_manager,
        }
    }

    /// 建立有初始狀態的控制器
    pub fn with_state(model: impl Into<String>, graph: Graph) -> Self {
        let node_count = graph.node_count();
        let phase = QuestionPhase::from_node_count(node_count);

        // 載入已存在的約束條件
        let workspace = Workspace::new();
        let (_, constraints_xml) = workspace.read_constraints();
        let constraint_manager = if constraints_xml.is_empty() {
            ConstraintManager::new()
        } else {
            ConstraintManager::from_xml(&constraints_xml)
        };

        // 初始化長期記憶管理器
        let memory_manager = MemoryManager::new();

        GemmaController {
            ollama: OllamaClient::new(model),
            executor: Arc::new(Mutex::new(ToolExecutor::with_graph(graph))),
            registry: ToolRegistry::new(),
            workspace,
            messages: Vec::new(),
            task: String::new(),
            mode: ControllerMode::Diverge,
            phase,
            previous_phase: None,
            constraint_manager,
            memory_manager,
        }
    }

    /// 設定當前任務
    pub fn set_task(&mut self, task: impl Into<String>) {
        self.task = task.into();
    }

    /// 取得當前任務
    pub fn task(&self) -> &str {
        &self.task
    }

    /// 設定模式
    pub fn set_mode(&mut self, mode: ControllerMode) {
        self.mode = mode;
    }

    /// 取得當前模式
    pub fn mode(&self) -> ControllerMode {
        self.mode
    }

    /// 取得當前提問階段（v0.4）
    pub fn phase(&self) -> QuestionPhase {
        self.phase
    }

    /// 檢查 Ollama 是否可用
    pub async fn health_check(&self) -> bool {
        self.ollama.health_check().await
    }

    /// 執行一輪對話
    ///
    /// 發送訊息給 gemma4，處理工具呼叫，直到不再有工具呼叫。
    pub async fn run_round(&mut self, user_input: &str) -> Result<String, ControllerError> {
        // 讀取狀態摘要（讓 gemma4 知道目前狀態）
        // 檢查並處理階段轉換（v0.4）
        let transition_message = self.check_phase_transition();

        // 建構系統提示詞（v0.7: graph 狀態直接 inject，不需要讀 status.xml）
        let system_prompt = self.build_system_prompt();

        // 加入階段轉換提示訊息（如果有的話）
        if let Some(msg) = transition_message {
            self.messages.push(Message::system(&msg));
        }

        // 加入使用者訊息
        self.messages.push(Message::user(user_input));

        // 執行對話迴圈
        let max_iterations = 10;
        let mut iteration = 0;

        while iteration < max_iterations {
            iteration += 1;

            // 建構請求（加入系統提示詞如果是第一輪）
            let mut request_messages = Vec::new();
            request_messages.push(Message::system(&system_prompt));
            request_messages.extend(self.messages.clone());

            // 發送請求（不使用 API tools——完全依靠文字解析執行工具）
            let response = self
                .ollama
                .chat(request_messages, None) // v0.7: 不傳 tools，靠文字解析
                .await
                .map_err(ControllerError::Ollama)?;

            let assistant_message = response.message;
            let content = &assistant_message.content;
            self.messages.push(assistant_message.clone());

            // v0.7 新增：檢查文字輸出中的 JSON tool call block
            let text_calls = self.parse_text_tool_calls(content);
            if !text_calls.is_empty() {
                // 最多只處理第一個文字 tool call，避免迴圈
                let call = &text_calls[0];
                let result = self.execute_tool(call);
                // 把工具結果加入，但不繼續迴圈——直接返回乾淨的回覆
                self.messages.push(Message {
                    role: "tool".to_string(),
                    content: format!("工具執行結果：{}", result.message),
                    tool_calls: None,
                });
                // 返回（跳過 JSON block 並附加工具結果說明）
                let clean = self.strip_json_tool_calls(content);
                return Ok(format!(
                    "{}\n\n[工具執行: {} → {}]",
                    clean,
                    call.function.name,
                    if result.success { "成功" } else { "失敗" }
                ));
            }

            //沒有工具呼叫，返回回覆內容（移除 JSON block）
            // v0.7: 移除输出去的 JSON tool call block，避免下輪重複輸出
            let clean = self.strip_json_tool_calls(content);

            // v0.8: 解析並應用 NodeUpdate（節點萃取）
            let update_result = self.apply_node_update(content);

            // 返回（附加萃取結果如果有）
            if let Ok(msg) = update_result {
                return Ok(format!("{}\n\n{}", clean, msg));
            }
            return Ok(clean);
        }

        Err(ControllerError::MaxIterationsReached(max_iterations))
    }

    /// 移除文字中的 JSON tool call block（v0.7 新增）
    fn strip_json_tool_calls(&self, content: &str) -> String {
        let mut result = Vec::new();
        let mut in_code_block = false;

        for line in content.lines() {
            let trimmed = line.trim();
            // 檢查是否在 ```json ... ``` 區塊內
            if trimmed.starts_with("```json") {
                in_code_block = true;
                continue;
            }
            if in_code_block && trimmed == "```" {
                in_code_block = false;
                continue;
            }
            if in_code_block {
                continue; // 跳過 JSON 區塊內容
            }
            // 跳過純 JSON 的行（不在 code block 內）
            if let Ok(_) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    continue;
                }
            }
            result.push(line);
        }

        let cleaned = result.join("\n").trim().to_string();
        if cleaned.is_empty() {
            content.trim().to_string()
        } else {
            cleaned
        }
    }

    /// 解析並應用 NodeUpdate（v0.8 新增）
    /// 從 gemma4 回應中萃取 <node_update> 並寫入節點欄位
    fn apply_node_update(&self, content: &str) -> Result<String, String> {
        // 嘗試解析 NodeUpdate
        let update = match NodeUpdateParser::extract(content) {
            Some(u) if u.has_content() => u,
            _ => return Err("No valid NodeUpdate found".to_string()),
        };

        // 取得 graph（從 executor）
        let mut graph = self.executor.lock().unwrap().get_graph();

        // 應用更新
        NodeUpdateParser::apply(&mut graph, &update)
    }

    /// 從文字內容中解析 JSON tool call block（v0.7 新增）
    ///
    /// gemma4 小模型可能把工具當文字輸出，我們從文字中解析 JSON 區塊。
    fn parse_text_tool_calls(&self, content: &str) -> Vec<ToolCall> {
        // 找 ```json ... ``` 區塊
        let mut calls = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            // 嘗試解析行作為 JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                // 兩種格式：{"name": "...", "parameters": {...}}
                // 或 {"function": {"name": "...", "arguments": "..."}}
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    let params = json.get("parameters")
                        .and_then(|v| serde_json::to_string(v).ok())
                        .unwrap_or_else(|| "{}".to_string());
                    calls.push(ToolCall {
                        function: crate::ollama::client::ToolCallFunction {
                            name: name.to_string(),
                            arguments: params,
                        },
                    });
                } else if let Some(func) = json.get("function").and_then(|v| v.as_object()) {
                    let name = func.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let args = func.get("arguments")
                        .map(|v| {
                            if let Some(s) = v.as_str() {
                                s.to_string()
                            } else {
                                serde_json::to_string(v).unwrap_or_default()
                            }
                        })
                        .unwrap_or_else(|| "{}".to_string());
                    calls.push(ToolCall {
                        function: crate::ollama::client::ToolCallFunction {
                            name: name.to_string(),
                            arguments: args,
                        },
                    });
                }
            }
        }

        calls
    }

    /// 執行單一工具呼叫
    fn execute_tool(&self, tool_call: &ToolCall) -> ToolResult {
        self.executor.lock().unwrap().execute(tool_call)
    }

    /// 檢查階段轉換並更新（v0.4）
    ///
    /// 根據當前圖的節點數計算新階段，
    /// 如果階段改變，回傳轉換提示訊息。
    fn check_phase_transition(&mut self) -> Option<String> {
        let graph = self.executor.lock().unwrap().get_graph();
        let node_count = graph.node_count();
        let new_phase = QuestionPhase::from_node_count(node_count);

        // 如果階段改變
        if new_phase != self.phase {
            self.previous_phase = Some(self.phase);
            self.phase = new_phase;
            // 回傳階段轉換訊息
            Some(new_phase.transition_message().to_string())
        } else {
            None
        }
    }

    /// 讀取狀態摘要
    fn read_status_summary(&self) -> String {
        // 讀取 workspace status.xml
        let path = self.workspace.status_path();
        let status_content = if path.exists() {
            std::fs::read_to_string(path).unwrap_or_else(|_| {
                "狀態檔案不存在".to_string()
            })
        } else {
            "尚未初始化狀態".to_string()
        };

        // v0.7: 讀取 Feedback 摘要（直接從記憶體中的圖）
        let graph = self.executor.lock().unwrap().get_graph();
        let feedback_summary = graph.feedback_summary();

        format!(
            "{}\n\n--- Execution Feedback ---\n{}\n--- End Feedback ---",
            status_content, feedback_summary
        )
    }

    /// 建構節點上下文（v0.8 新增）
    /// 每次只送「當前任務 + 節點重點資訊」，不是整張圖
    /// 固定格式，不因深度而被二次壓縮
    fn build_node_context(&self) -> String {
        let graph = self.executor.lock().unwrap().get_graph();
        let nodes = graph.get_all_nodes();

        if nodes.is_empty() {
            return "## 推理圖狀態\n\n（目前沒有任何節點，從零開始）".to_string();
        }

        // 找到「活跃且最具體」的節點作為當前焦點
        // 原則：优先选有 key_findings 或 conclusion 的節點，其次选子節點最多的
        let focus_node = nodes.iter()
            .filter(|n| n.status == NodeStatus::Active || n.status == NodeStatus::Draft)
            .max_by_key(|n| {
                let findings_score = n.key_findings.len();
                let has_conclusion = if n.conclusion.is_some() { 10 } else { 0 };
                let child_score = n.child_edges.len();
                findings_score + has_conclusion + child_score
            })
            .map(|n| n.to_prompt_summary())
            .unwrap_or_else(|| "（找不到活躍節點）".to_string());

        // 只顯示「最相關的」其他節點（最多2個）
        let related_nodes: Vec<String> = nodes.iter()
            .filter(|n| n.status != NodeStatus::Pruned && n.status != NodeStatus::Failed)
            .take(2)
            .map(|n| n.to_prompt_summary())
            .collect();

        let mut lines = vec!["## 推理圖狀態（v0.8 格式）\n".to_string()];

        lines.push("### 當前焦點節點\n".to_string());
        lines.push(focus_node);
        lines.push("\n### 其他相關節點（最多2個）\n".to_string());
        for rn in &related_nodes {
            lines.push((*rn).clone());
            lines.push("\n".to_string());
        }

        lines.join("")
    }

    /// 建構系統提示詞（v0.7: graph 狀態直接 inject，不需要 gemma4 去讀檔案）
    fn build_system_prompt(&self) -> String {
        let mode_str = match self.mode {
            ControllerMode::Diverge => "發散（探索）",
            ControllerMode::Converge => "收斂（聚焦）",
        };

        // 取得約束條件格式化
        let constraints_str = self.constraint_manager.format_for_prompt();

        // 取得長期記憶格式化
        let memory_str = self.memory_manager.format_for_prompt();

        // 每次都從記憶體中的圖建構節點上下文（v0.8: 只送當前焦點 + 相關節點，不送全圖）
        let node_context = self.build_node_context();

        match self.phase {
            QuestionPhase::Exploration => {
                format!(
                    r#"你是一個推理控制器，正在「探索期」。

## 當前推理圖狀態（已 inject，你不需要去讀取）
{}

## 你的任務
每次回复前，先用 `diverge` 工具記錄用户偏好。

## 輸出格式（嚴格遵守）

先用這個 JSON 格式呼叫工具，然後再用文字回覆：

```
{{"name": "diverge", "parameters": {{"node_id": "根節點ID或null", "content": "用户說的話或偏好"}}}}
```

範例（圖為空時）：
用户：「我想悠閒的兩天一夜金門旅行，看戰地歷史和吃美食」
你輸出：
```
{{"name": "diverge", "parameters": {{"node_id": null, "content": "悠閒、兩天一夜、戰地歷史、在地美食"}}}}
```
然後文字回复：「了解了。那戰地歷史方面，你對哪個時期最有興趣？」

範例（已有根節點時，node_id 傳那個 ID）：
用户：「我想吃牛肉麵」
你輸出：
```
{{"name": "diverge", "parameters": {{"node_id": "根節點ID", "content": "想吃牛肉麵"}}}}
```

## 規則
- 圖為空時 node_id 傳 null，系統會自動建立根節點
- 圖已有節點時，node_id 傳**第一個節點的 ID**（在上面的推理圖狀態中可以查到）
- 記錄 3 個偏好後就可以結束探索期

## 節點更新格式（v0.8 新增，每輪回覆後必須包含）
在你回覆的最後，請用這個格式更新當前節點：

```xml
<node_update>
  <findings>
    <item>你從用户輸入中發現的重點（最多5條）</item>
    <item>例如：用戶偏好悠閒、想要戰地歷史、喜歡美食</item>
  </findings>
  <conclusion null="true"/>
  <topics>
    <topic>金門</topic>
    <topic>戰地歷史</topic>
  </topics>
</node_update>
```

注意：
- findings 是濃縮的要點，不是對話原文
- conclusion 還沒有形成共識時用 `null="true"`
- topics 是相關主題（用於横向關聯）

{}

{}"#,
                    node_context,
                    constraints_str,
                    memory_str
                )
            }
            QuestionPhase::Development => {
                format!(
                    r#"你是一個推理控制器，正在進入「發展期」。

當前任務：{}

## 當前推理圖狀態（已 inject，你不需要去讀取）
{}

當前模式：{}

{}

{}

重要：你現在可以開始提出建議，但每次建議前需先問：
「你對這個方向有什麼想法？」或「這符合你的預期嗎？」

可用工具：
- diverge(node_id, content, count) - 發散生成子節點
- converge(threshold) - 觸發收斂
- save(name) - 儲存狀態到 workspace
- load(name) - 從 workspace 載入狀態
- output(format, name) - 產出為 XML/MD 檔案
- status() - 回傳狀態摘要
- execute(command, args, node_id) - 執行外部命令並記錄 Feedback
- verify(node_id, expected) - 驗證執行結果
- retry(node_id) - 重試失敗的任務

## Execution Feedback Loop（v0.7 新增）

當你呼叫 execute 工具時：
1. 系統會實際執行命令（echo、python3、curl 等）
2. 執行結果會寫入節點的 feedback_result 欄位
3. 下次讀取 status() 時，會看到 Feedback 摘要

**如何根據 Feedback 調整推理：**
- 如果 execute 失敗（節點變為 [失敗]）→ 考慮其他方法，不要重複失敗的路徑
- 如果 execute 成功 → 這個假設得到驗證，可以繼續發散
- 使用 verify 工具驗證預期結果是否符合
- 使用 retry 工具重新執行失敗的任務（先修正問題再重試）

規則：
- 預設模式為發散
- 當複雜度 > 80% 閾值，自動提醒
- 沒有固定目標，持續探索
- 產出存到 workspace/

"#,
                    self.task, node_context, mode_str,
                    constraints_str, memory_str
                )
            }
            QuestionPhase::Mature => {
                format!(
                    r#"你是一個推理控制器，正在進入「成熟期」。

當前任務：{}

## 當前推理圖狀態（已 inject，你不需要去讀取）
{}

當前模式：{}

{}

{}

重要：在確認用戶想法後，你可以主導發散和收斂操作，開始形成具體的建議框架。

可用工具：
- diverge(node_id, content, count) - 發散生成子節點
- converge(threshold) - 觸發收斂
- save(name) - 儲存狀態到 workspace
- load(name) - 從 workspace 載入狀態
- output(format, name) - 產出為 XML/MD 檔案
- status() - 回傳狀態摘要
- execute(command, args, node_id) - 執行外部命令並記錄 Feedback
- verify(node_id, expected) - 驗證執行結果
- retry(node_id) - 重試失敗的任務

## Execution Feedback Loop（v0.7 新增）

當你呼叫 execute 工具時：
1. 系統會實際執行命令（echo、python3、curl 等）
2. 執行結果會寫入節點的 feedback_result 欄位
3. 下次讀取 status() 時，會看到 Feedback 摘要

**如何根據 Feedback 調整推理：**
- 如果 execute 失敗（節點變為 [失敗]）→ 考慮其他方法，不要重複失敗的路徑
- 如果 execute 成功 → 這個假設得到驗證，可以繼續發散
- 使用 verify 工具驗證預期結果是否符合
- 使用 retry 工具重新執行失敗的任務（先修正問題再重試）

規則：
- 預設模式為發散
- 當複雜度 > 80% 閾值，自動提醒
- 沒有固定目標，持續探索
- 產出存到 workspace/

"#,
                    self.task, node_context, mode_str,
                    constraints_str, memory_str
                )
            }
        }
    }

    /// 取得工具執行器的圖副本
    pub fn get_graph(&self) -> Graph {
        self.executor.lock().unwrap().get_graph()
    }

    /// 同步圖到工具執行器（v0.7 新增）
    ///
    /// 當在 REPL 中直接修改圖時，需要呼叫此方法同步回工具執行器。
    pub fn sync_graph(&mut self, graph: Graph) {
        let mut executor = self.executor.lock().unwrap();
        // 透過 replace_graph 方式同步
        *executor = ToolExecutor::with_graph(graph);
    }

    /// 取得對話歷史
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// 清除對話歷史
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
}

/// 控制器錯誤
#[derive(Debug)]
pub enum ControllerError {
    /// Ollama 錯誤
    Ollama(crate::ollama::client::OllamaError),
    /// 工具執行錯誤
    ToolError(String),
    /// 超出最大迭代次數
    MaxIterationsReached(usize),
}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerError::Ollama(e) => write!(f, "Ollama 錯誤: {}", e),
            ControllerError::ToolError(msg) => write!(f, "工具錯誤: {}", msg),
            ControllerError::MaxIterationsReached(n) => {
                write!(f, "超出最大迭代次數 ({})", n)
            }
        }
    }
}

impl std::error::Error for ControllerError {}

impl From<crate::ollama::client::OllamaError> for ControllerError {
    fn from(err: crate::ollama::client::OllamaError) -> Self {
        ControllerError::Ollama(err)
    }
}

/// 工具執行結果（重新匯出）
pub use crate::tools::executor::ToolResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_mode_default() {
        let controller = GemmaController::new("gemma4:e2b");
        assert_eq!(controller.mode(), ControllerMode::Diverge);
    }

    #[test]
    fn test_controller_set_mode() {
        let mut controller = GemmaController::new("gemma4:e2b");
        controller.set_mode(ControllerMode::Converge);
        assert_eq!(controller.mode(), ControllerMode::Converge);
    }

    #[test]
    fn test_controller_set_task() {
        let mut controller = GemmaController::new("gemma4:e2b");
        controller.set_task("測試任務");
        assert_eq!(controller.task(), "測試任務");
    }

    #[test]
    fn test_controller_with_state() {
        let graph = Graph::new();
        let controller = GemmaController::with_state("gemma4:e2b", graph);
        assert_eq!(controller.get_graph().node_count(), 0);
    }

    #[test]
    fn test_build_system_prompt() {
        // Exploration phase prompt
        let controller = GemmaController::new("gemma4:e2b");
        let prompt = controller.build_system_prompt();
        assert!(prompt.contains("推理控制器"));
        assert!(prompt.contains("探索期"));

        // Development phase prompt (with 3 nodes)
        let mut graph = Graph::new();
        graph.add_node(crate::models::Node::new("節點1".to_string(), 1));
        graph.add_node(crate::models::Node::new("節點2".to_string(), 2));
        graph.add_node(crate::models::Node::new("節點3".to_string(), 3));
        let controller2 = GemmaController::with_state("gemma4:e2b", graph);
        let prompt2 = controller2.build_system_prompt();
        assert!(prompt2.contains("推理控制器"));
        assert!(prompt2.contains("可用工具"));
        assert!(prompt2.contains("發展期"));
    }

    // === QuestionPhase tests (v0.4) ===

    #[test]
    fn test_question_phase_from_node_count() {
        assert_eq!(QuestionPhase::from_node_count(0), QuestionPhase::Exploration);
        assert_eq!(QuestionPhase::from_node_count(1), QuestionPhase::Exploration);
        assert_eq!(QuestionPhase::from_node_count(2), QuestionPhase::Exploration);
        assert_eq!(QuestionPhase::from_node_count(3), QuestionPhase::Development);
        assert_eq!(QuestionPhase::from_node_count(5), QuestionPhase::Development);
        assert_eq!(QuestionPhase::from_node_count(10), QuestionPhase::Development);
        assert_eq!(QuestionPhase::from_node_count(11), QuestionPhase::Mature);
        assert_eq!(QuestionPhase::from_node_count(100), QuestionPhase::Mature);
    }

    #[test]
    fn test_question_phase_name() {
        assert_eq!(QuestionPhase::Exploration.name(), "探索期");
        assert_eq!(QuestionPhase::Development.name(), "發展期");
        assert_eq!(QuestionPhase::Mature.name(), "成熟期");
    }

    #[test]
    fn test_question_phase_transition_message() {
        let exploration_msg = QuestionPhase::Exploration.transition_message();
        assert!(exploration_msg.contains("探索期"));
        assert!(exploration_msg.contains("提問"));

        let development_msg = QuestionPhase::Development.transition_message();
        assert!(development_msg.contains("發展期"));

        let mature_msg = QuestionPhase::Mature.transition_message();
        assert!(mature_msg.contains("成熟期"));
    }

    #[test]
    fn test_controller_initial_phase() {
        let controller = GemmaController::new("gemma4:e2b");
        assert_eq!(controller.phase(), QuestionPhase::Exploration);
    }

    #[test]
    fn test_controller_phase_with_state() {
        let mut graph = Graph::new();
        let node1 = crate::models::Node::new("節點1".to_string(), 1);
        graph.add_node(node1);
        let node2 = crate::models::Node::new("節點2".to_string(), 2);
        graph.add_node(node2);

        // 2 nodes = Exploration
        let controller = GemmaController::with_state("gemma4:e2b", graph);
        assert_eq!(controller.phase(), QuestionPhase::Exploration);

        // Add one more node to get 3 = Development
        let mut graph2 = Graph::new();
        graph2.add_node(crate::models::Node::new("節點1".to_string(), 1));
        graph2.add_node(crate::models::Node::new("節點2".to_string(), 2));
        graph2.add_node(crate::models::Node::new("節點3".to_string(), 3));
        let controller2 = GemmaController::with_state("gemma4:e2b", graph2);
        assert_eq!(controller2.phase(), QuestionPhase::Development);
    }

    #[test]
    fn test_controller_phase_from_graph() {
        // Test that controller starts with Exploration phase
        let controller = GemmaController::new("gemma4:e2b");
        assert_eq!(controller.phase(), QuestionPhase::Exploration);
    }
}
