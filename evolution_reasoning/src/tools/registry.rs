//! 工具註冊表
//!
//! 定義所有可用工具的名稱、描述和參數結構。

use std::collections::HashMap;

use crate::ollama::client::{FunctionDefinition, Tool, ToolParameters};

/// 工具註冊表
///
/// 儲存所有可用工具的定義，並可根據名稱查詢。
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// 工具定義
    tools: HashMap<String, Tool>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// 建立新的工具註冊表（包含所有內建工具）
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // diverge(node_id, content, count) - 發散生成子節點
        let mut diverge_params = ToolParameters::new();
        diverge_params.add_string_prop("node_id", "父節點 ID");
        diverge_params.add_string_prop("content", "新節點內容（可選）");
        diverge_params.add_integer_prop("count", "要生成的子節點數量（預設 3）");

        tools.insert(
            "diverge".to_string(),
            Tool::new(
                "diverge",
                "發散：在指定節點下生成多個可能的子節點。這是探索不同思考方向的核心操作。",
                diverge_params,
            ),
        );

        // converge() - 觸發收斂
        let mut converge_params = ToolParameters::new();
        converge_params.add_string_prop("threshold", "分數閾值（可選，預設使用系統設定）");

        tools.insert(
            "converge".to_string(),
            Tool::new(
                "converge",
                "收斂：評估並刪除圖中低分節點。當複雜度過高或需要聚焦時觸發。",
                converge_params,
            ),
        );

        // save(name) - 儲存狀態
        let mut save_params = ToolParameters::new();
        save_params.add_string_prop("name", "儲存名稱");

        tools.insert(
            "save".to_string(),
            Tool::new(
                "save",
                "儲存：將當前推理圖狀態儲存到 workspace 的命名檔案。",
                save_params,
            ),
        );

        // load(name) - 載入狀態
        let mut load_params = ToolParameters::new();
        load_params.add_string_prop("name", "儲存名稱");

        tools.insert(
            "load".to_string(),
            Tool::new(
                "load",
                "載入：從 workspace 的命名檔案載入推理圖狀態。",
                load_params,
            ),
        );

        // output(format) - 產出檔案
        let mut output_params = ToolParameters::new();
        output_params.add_string_prop("format", "輸出格式：xml 或 md");
        output_params.add_string_prop("name", "輸出檔名（不含副檔名）");

        tools.insert(
            "output".to_string(),
            Tool::new(
                "output",
                "輸出：將當前狀態產出為指定格式的檔案到 workspace 目錄。",
                output_params,
            ),
        );

        // status() - 狀態摘要
        tools.insert(
            "status".to_string(),
            Tool::new(
                "status",
                "狀態：取得當前推理圖的狀態摘要（節點數、邊數、複雜度等）。gemma4 可讀取 workspace/status.xml 獲取詳細資訊。",
                ToolParameters::new(),
            ),
        );

        // execute(command, args, timeout_ms, parse_mode, node_id) - 沙盒執行外部命令
        let mut execute_params = ToolParameters::new();
        execute_params.add_string_prop("command", "要執行的命令（如 echo, python3, curl）");
        execute_params.properties.get_mut("command").unwrap().description = "要執行的命令（如 echo, python3, curl）".to_string();
        // args 是選填的，不加到 required
        execute_params.add_integer_prop("timeout_ms", "逾時毫秒（預設 30000）");
        execute_params.properties.get_mut("timeout_ms").unwrap().description = "逾時毫秒（預設 30000）".to_string();
        execute_params.add_string_prop("parse_mode", "解析模式：json | text | exit_code（預設 text）");
        execute_params.properties.get_mut("parse_mode").unwrap().description = "解析模式：json | text | exit_code（預設 text）".to_string();
        execute_params.add_string_prop("node_id", "關聯的節點 ID（可選，填寫後 Feedback 會寫入該節點）");
        execute_params.properties.get_mut("node_id").unwrap().description = "關聯的節點 ID（可選，填寫後 Feedback 會寫入該節點）".to_string();

        tools.insert(
            "execute".to_string(),
            Tool::new(
                "execute",
                "沙盒執行：在隔離環境中執行外部命令，並回傳結構化結果。parse_mode=json 時會嘗試將 stdout 解析為 JSON。超時或執行失敗也會回傳 error 欄位。",
                execute_params,
            ),
        );

        // exec_status() - 沙盒環境資訊
        tools.insert(
            "exec_status".to_string(),
            Tool::new(
                "exec_status",
                "沙盒狀態：取得執行環境的資訊（作業系統、可用目錄、環境變數）。",
                ToolParameters::new(),
            ),
        );

        // checkpoint(node_id, reason, description) - 建立檢查點
        let mut checkpoint_params = ToolParameters::new();
        checkpoint_params.add_string_prop("node_id", "掛載的節點 ID（可選，預設 current_topic root）");
        checkpoint_params.properties.get_mut("node_id").unwrap().description = "掛載的節點 ID（可選，預設 current_topic root）".to_string();
        checkpoint_params.add_string_prop("reason", "建立原因：user_decision | phase_transition | pre_diverge | pre_execute | manual（預設 manual）");
        checkpoint_params.properties.get_mut("reason").unwrap().description = "建立原因：user_decision | phase_transition | pre_diverge | pre_execute | manual（預設 manual）".to_string();
        checkpoint_params.add_string_prop("description", "檢查點描述（可選）");

        tools.insert(
            "checkpoint".to_string(),
            Tool::new(
                "checkpoint",
                "建立檢查點：將當前 graph 狀態 snapshot 保存到回溯系統。在重要決策點或執行外部命令前建立。可用 backtrack 回溯到此狀態。",
                checkpoint_params,
            ),
        );

        // backtrack(checkpoint_id, restore_last) - 回溯到檢查點
        let mut backtrack_params = ToolParameters::new();
        backtrack_params.add_string_prop("checkpoint_id", "目標檢查點 ID（與 restore_last 二選一）");
        backtrack_params.properties.get_mut("checkpoint_id").unwrap().description = "目標檢查點 ID（與 restore_last 二選一）".to_string();
        backtrack_params.add_string_prop("restore_last", "是否回溯到最後一個檢查點：true 或 false");
        backtrack_params.properties.get_mut("restore_last").unwrap().description = "是否回溯到最後一個檢查點：true 或 false".to_string();

        tools.insert(
            "backtrack".to_string(),
            Tool::new(
                "backtrack",
                "回溯：將 graph 狀態恢復到指定檢查點。執行外部命令失敗後常用於恢復到 pre_execute 檢查點。",
                backtrack_params,
            ),
        );

        // record_failure(node_id, pattern_type, command, exit_code, stderr, execute_result_json) - 記錄失敗模式
        let mut record_failure_params = ToolParameters::new();
        record_failure_params.add_string_prop("node_id", "失敗發生時的節點 ID");
        record_failure_params.properties.get_mut("node_id").unwrap().description = "失敗發生時的節點 ID".to_string();
        record_failure_params.add_string_prop("pattern_type", "失敗類型：exit_nonzero | command_not_found | timeout | parse_error | execution_failed（可選，有 execute_result_json 時自動判斷）");
        record_failure_params.properties.get_mut("pattern_type").unwrap().description = "失敗類型（可選，有 execute_result_json 時自動判斷）".to_string();
        record_failure_params.add_string_prop("command", "相關命令（可選）");
        record_failure_params.add_integer_prop("exit_code", "退出碼（可選）");
        record_failure_params.add_string_prop("stderr", "錯誤輸出（可選）");
        record_failure_params.add_string_prop("execute_result_json", "execute 工具的完整 JSON 輸出（傳入後自動解析失敗模式）");

        tools.insert(
            "record_failure".to_string(),
            Tool::new(
                "record_failure",
                "記錄失敗：將 execute 失敗的模式記錄到回溯系統，並生成修正假設供後續決策參考。建議在 execute 失敗後立即呼叫。",
                record_failure_params,
            ),
        );

        // get_hypotheses(failure_id, get_last) - 取得修正假設
        let mut get_hypotheses_params = ToolParameters::new();
        get_hypotheses_params.add_string_prop("failure_id", "失敗記錄 ID（與 get_last 二選一）");
        get_hypotheses_params.properties.get_mut("failure_id").unwrap().description = "失敗記錄 ID（與 get_last 二選一）".to_string();
        get_hypotheses_params.add_string_prop("get_last", "是否取得最後一筆記錄的假設：true 或 false");
        get_hypotheses_params.properties.get_mut("get_last").unwrap().description = "是否取得最後一筆記錄的假設：true 或 false".to_string();

        tools.insert(
            "get_hypotheses".to_string(),
            Tool::new(
                "get_hypotheses",
                "取得假設：根據失敗記錄生成多個可能的修正假設，每個假設包含 hypothesis（原因推測）、suggested_action（建議動作）、confidence（信心度）。",
                get_hypotheses_params,
            ),
        );

        // export_graph(format) - 匯出推理圖為 YAML/JSON/DSL
        let mut export_graph_params = ToolParameters::new();
        export_graph_params.add_string_prop("format", "匯出格式：yaml（預設）、json、dsl");

        tools.insert(
            "export_graph".to_string(),
            Tool::new(
                "export_graph",
                "匯出推理圖：將當前推理圖狀態匯出為 YAML、JSON 或 DSL 格式。可用於外部工作流整合或備份。",
                export_graph_params,
            ),
        );

        // export_node(node_id, format) - 匯出單一節點
        let mut export_node_params = ToolParameters::new();
        export_node_params.add_string_prop("node_id", "要匯出的節點 ID");
        export_node_params.add_string_prop("format", "匯出格式：yaml（預設）、json、dsl");

        tools.insert(
            "export_node".to_string(),
            Tool::new(
                "export_node",
                "匯出節點：將指定節點的詳細狀態匯出為 YAML、JSON 或 DSL 格式。",
                export_node_params,
            ),
        );

        // export_hypotheses(failure_id, format) - 匯出假設列表
        let mut export_hypotheses_params = ToolParameters::new();
        export_hypotheses_params.add_string_prop("failure_id", "失敗記錄 ID（可選，不提供則匯出全部）");
        export_hypotheses_params.add_string_prop("format", "匯出格式：yaml（預設）、json、dsl");

        tools.insert(
            "export_hypotheses".to_string(),
            Tool::new(
                "export_hypotheses",
                "匯出假設：將回溯系統中的修正假設匯出為 YAML、JSON 或 DSL 格式，方便外部 AI 使用或持久化。",
                export_hypotheses_params,
            ),
        );

        // export_backtrack(format) - 匯出回溯狀態
        let mut export_backtrack_params = ToolParameters::new();
        export_backtrack_params.add_string_prop("format", "匯出格式：yaml（預設）、json、dsl");

        tools.insert(
            "export_backtrack".to_string(),
            Tool::new(
                "export_backtrack",
                "匯出回溯狀態：將檢查點和失敗歷史匯出為 YAML、JSON 或 DSL 格式。",
                export_backtrack_params,
            ),
        );

        // export_memory(format) - 匯出長期記憶狀態
        let mut export_memory_params = ToolParameters::new();
        export_memory_params.add_string_prop("format", "匯出格式：yaml（預設）、json、dsl");

        tools.insert(
            "export_memory".to_string(),
            Tool::new(
                "export_memory",
                "匯出長期記憶狀態：將用戶 profile、歷史和已探索主題匯出為 YAML、JSON 或 DSL 格式。",
                export_memory_params,
            ),
        );

        // query_backtrack(resource) - 查詢回溯狀態端點
        let mut query_backtrack_params = ToolParameters::new();
        query_backtrack_params.add_string_prop("resource", "查詢資源：checkpoints、failures、hypotheses、summary");

        tools.insert(
            "query_backtrack".to_string(),
            Tool::new(
                "query_backtrack",
                "查詢回溯狀態：查詢檢查點、失敗歷史或假設摘要，返回 JSON 格式。",
                query_backtrack_params,
            ),
        );

        // verify(node_id, expected) - 驗證執行結果（v0.7 新增）
        let mut verify_params = ToolParameters::new();
        verify_params.add_string_prop("node_id", "要驗證的節點 ID");
        verify_params.add_string_prop("expected", "預期結果的關鍵字");

        tools.insert(
            "verify".to_string(),
            Tool::new(
                "verify",
                "驗證執行結果：檢查指定節點的 Feedback 結果是否包含預期的關鍵字。",
                verify_params,
            ),
        );

        // retry(node_id) - 重試失敗任務（v0.7 新增）
        let mut retry_params = ToolParameters::new();
        retry_params.add_string_prop("node_id", "要重試的節點 ID");

        tools.insert(
            "retry".to_string(),
            Tool::new(
                "retry",
                "重試失敗任務：對指定節點的失敗任務進行重試，並返回新的執行結果。",
                retry_params,
            ),
        );

        // last_node_id() - 查詢最後建立的節點 ID（v0.7 新增）
        // 讓 gemma4 可以追蹤最近建立的節點，以便在後續 diverge 呼叫中使用
        tools.insert(
            "last_node_id".to_string(),
            Tool::new(
                "last_node_id",
                "查詢最後建立的節點 ID。當你需要對最新建立的節點進行操作時，先呼叫此工具取得節點 ID，再將其用於下次 diverge 的 node_id 參數。",
                ToolParameters::new(),
            ),
        );

        ToolRegistry { tools }
    }

    /// 取得所有工具定義（給 Ollama API 用）
    pub fn get_all_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    /// 根據名稱取得工具定義
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// 檢查工具是否存在
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// 取得所有工具名稱
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.has_tool("diverge"));
        assert!(registry.has_tool("converge"));
        assert!(registry.has_tool("save"));
        assert!(registry.has_tool("load"));
        assert!(registry.has_tool("output"));
        assert!(registry.has_tool("status"));
    }

    #[test]
    fn test_tool_registry_get_all() {
        let registry = ToolRegistry::new();
        let tools = registry.get_all_tools();
        assert_eq!(tools.len(), 21);
    }

    #[test]
    fn test_tool_registry_tool_names() {
        let registry = ToolRegistry::new();
        let names = registry.tool_names();
        assert!(names.contains(&"diverge"));
        assert!(names.contains(&"status"));
    }

    #[test]
    fn test_tool_registry_get_tool() {
        let registry = ToolRegistry::new();
        let diverge = registry.get_tool("diverge").unwrap();
        assert_eq!(diverge.function.name, "diverge");
    }

    #[test]
    fn test_tool_registry_has_not() {
        let registry = ToolRegistry::new();
        assert!(!registry.has_tool("nonexistent"));
    }
}
