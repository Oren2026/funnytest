//! Gemma 控制器
//!
//! 負責協調 gemma4 模型與 Engine 的互動。

use std::sync::{Arc, Mutex};

use crate::engine::ConstraintManager;
use crate::memory::MemoryManager;
use crate::models::Graph;
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
        let status_summary = self.read_status_summary();

        // 檢查並處理階段轉換（v0.4）
        let transition_message = self.check_phase_transition();

        // 建構系統提示詞
        let system_prompt = self.build_system_prompt(&status_summary);

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

            // 發送請求
            let tools = Some(self.registry.get_all_tools());
            let response = self
                .ollama
                .chat(request_messages, tools)
                .await
                .map_err(ControllerError::Ollama)?;

            let assistant_message = response.message;
            self.messages.push(assistant_message.clone());

            // 檢查是否有工具呼叫
            let tool_calls = response.tool_calls;
            if tool_calls.is_none() || tool_calls.as_ref().unwrap().is_empty() {
                // 沒有工具呼叫，返回回覆內容
                return Ok(assistant_message.content);
            }

            // 處理工具呼叫
            let calls = tool_calls.unwrap();
            for call in calls {
                let result = self.execute_tool(&call);

                // 加入工具回應
                let tool_msg = ToolMessage::new(
                    format!("call_{}", iteration),
                    result.message,
                );
                self.messages.push(Message {
                    role: "tool".to_string(),
                    content: serde_json::to_string(&tool_msg).unwrap_or_default(),
                    tool_calls: None,
                });
            }
        }

        Err(ControllerError::MaxIterationsReached(max_iterations))
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
        let path = self.workspace.status_path();
        if path.exists() {
            std::fs::read_to_string(path).unwrap_or_else(|_| {
                "狀態檔案不存在".to_string()
            })
        } else {
            "尚未初始化狀態".to_string()
        }
    }

    /// 建構系統提示詞
    fn build_system_prompt(&self, status_summary: &str) -> String {
        let mode_str = match self.mode {
            ControllerMode::Diverge => "發散（探索）",
            ControllerMode::Converge => "收斂（聚焦）",
        };

        // 取得約束條件格式化
        let constraints_str = self.constraint_manager.format_for_prompt();

        // 取得長期記憶格式化
        let memory_str = self.memory_manager.format_for_prompt();

        match self.phase {
            QuestionPhase::Exploration => {
                format!(
                    r#"你是一個推理控制器，但你的首要任務是**提問**，而不是給答案。

當前處於「探索期」——節點數量少（< 3），你的主要任務是：
1. 透過提問了解用戶的興趣、價值觀、經驗
2. 不要急於給出框架或建議
3. 每個回覆至少包含一個問題

{}
{}

提問原則：
- 先問開放性問題了解用戶
- 避免問封閉性問題（是/否）
- 問題要具體且與用戶相關

興趣導向問題範本：
- 「你對哪些領域特別有熱情？」
- 「什麼事情讓你覺得有意義？」

價值觀導向問題範本：
- 「什麼事情對你來說最重要？」
- 「你希望的人生畫面是什麼樣子？」

經驗導向問題範本：
- 「你過去有什麼類似的經驗？」
- 「你之前嘗試過什麼方式？」

約束導向問題範本：
- 「有什麼限制是你必須考慮的？」
- 「時間、資源、環境上有什麼約束？」"#,
                    constraints_str,
                    memory_str
                )
            }
            QuestionPhase::Development => {
                format!(
                    r#"你是一個推理控制器，正在進入「發展期」。

當前任務：{}

狀態摘要（讀取 workspace/status.xml）：
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

規則：
- 預設模式為發散
- 當複雜度 > 80% 閾值，自動提醒
- 沒有固定目標，持續探索
- 產出存到 workspace/

每次操作後，請閱讀 workspace/status.xml 了解最新狀態。"#,
                    self.task, status_summary, mode_str,
                    constraints_str, memory_str
                )
            }
            QuestionPhase::Mature => {
                format!(
                    r#"你是一個推理控制器，正在進入「成熟期」。

當前任務：{}

狀態摘要（讀取 workspace/status.xml）：
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

規則：
- 預設模式為發散
- 當複雜度 > 80% 閾值，自動提醒
- 沒有固定目標，持續探索
- 產出存到 workspace/

每次操作後，請閱讀 workspace/status.xml 了解最新狀態。"#,
                    self.task, status_summary, mode_str,
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
        let status = "<status>節點數=0</status>";
        let prompt = controller.build_system_prompt(status);
        assert!(prompt.contains("推理控制器"));
        assert!(prompt.contains("探索期"));

        // Development phase prompt (with 3 nodes)
        let mut graph = Graph::new();
        graph.add_node(crate::models::Node::new("節點1".to_string(), 1));
        graph.add_node(crate::models::Node::new("節點2".to_string(), 2));
        graph.add_node(crate::models::Node::new("節點3".to_string(), 3));
        let controller2 = GemmaController::with_state("gemma4:e2b", graph);
        let status2 = "<status>節點數=3</status>";
        let prompt2 = controller2.build_system_prompt(status2);
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
