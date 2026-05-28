//! Gemma REPL — gemma4 互動對話介面
//!
//! v0.3 新功能：提供一個讓使用者能和 gemma4 對話的介面。
//! gemma4 作為控制器，透過工具系統操作 Rust Engine。
//!
//! # 流程
//!
//! ```ignore
//! 使用者輸入
//!     ↓
//! gemma4 (controller)
//!     ↓ 工具調用
//! Rust Engine (diverge/converge/save/load/output)
//!     ↓
//! Workspace XML 更新
//!     ↓
//! gemma4 讀取 status.xml 繼續對話
//!     ↓
//! 輸出回應給使用者
//! ```
//!
//! # 使用方式
//!
//! ```bash
//! cargo run -- --对话
//! ```

use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

use crate::controller::gemma_controller::{ControllerMode, GemmaController, QuestionPhase};
use crate::engine::{ComplexityBudget, ConstraintManager, ConvergeEngine, DivergeEngine};
use crate::cli::VisualPanel;
use crate::models::{Graph, Node, NodeStatus, Topic, TopicPhase};
use crate::tools::ToolExecutor;
use crate::workspace::Workspace;

// 可觀測性系統（v0.6 新增）
use crate::observability::{
    conversation::{ConversationLogger, ConversationRound, ToolCallRecord},
    phase_transition::{PhaseTransition, PhaseTransitionLogger},
    constraint_log::ConstraintChangeLogger,
    session_summary::{SessionSummaryLogger, SessionStats},
    snapshot::SnapshotLogger,
    ObservableLogger,
};

/// Gemma REPL 狀態
#[derive(Debug)]
pub struct GemmaREPLState {
    /// 控制器
    pub controller: GemmaController,
    /// 對話主題
    pub topic: String,
    /// 約束條件管理器
    pub constraint_manager: ConstraintManager,
    /// 是否正在運行
    pub running: bool,
    /// 視覺化面板
    pub visual: VisualPanel,
    /// 對話回合計數器（v0.6 新增）
    pub round_count: usize,
}

/// 建立新的 Gemma REPL 狀態
impl GemmaREPLState {
    /// 使用主題和約束建立新的 Gemma REPL
    pub fn new(topic: String, constraints: Vec<String>) -> Self {
        let graph = Graph::new();
        let mut controller = GemmaController::with_state("gemma4:e4b", graph);
        controller.set_task(topic.clone());

        // 初始化約束條件管理器
        let mut constraint_manager = ConstraintManager::new();
        for c in constraints {
            constraint_manager.add_user_constraint(c);
        }

        GemmaREPLState {
            controller,
            topic,
            constraint_manager,
            running: true,
            visual: VisualPanel::new(),
            round_count: 0,
        }
    }

    /// 建立有自訂模型的 Gemma REPL
    #[allow(dead_code)]
    pub fn with_model(topic: String, constraints: Vec<String>, model: &str) -> Self {
        let graph = Graph::new();
        let mut controller = GemmaController::with_state(model, graph);
        controller.set_task(topic.clone());

        // 初始化約束條件管理器
        let mut constraint_manager = ConstraintManager::new();
        for c in constraints {
            constraint_manager.add_user_constraint(c);
        }

        GemmaREPLState {
            controller,
            topic,
            constraint_manager,
            running: true,
            visual: VisualPanel::new(),
            round_count: 0,
        }
    }
}

/// 啟動 Gemma REPL互動式對話介面
///
/// 初始化 workspace、目錄和使用者輸入主題，
/// 然後進入對話迴圈。
pub fn run_gemma_repl() {
    println!("=== Evolution Reasoning Tool v0.4 ===");
    println!("gemma4 互動對話模式（提問習慣系統）\n");

    // 讀取討論主題
    print!("輸入討論主題：");
    io::stdout().flush().unwrap();
    let mut topic = String::new();
    if io::stdin().lock().read_line(&mut topic).is_err() {
        println!("無法讀取主題輸入");
        return;
    }
    let topic = topic.trim().to_string();
    if topic.is_empty() {
        println!("主題不能為空");
        return;
    }

    // 讀取約束條件（可選）
    print!("約束條件（可選，多個用分號分隔）：");
    io::stdout().flush().unwrap();
    let mut constraints_input = String::new();
    let _ = io::stdin().lock().read_line(&mut constraints_input);
    let constraints: Vec<String> = constraints_input
        .trim()
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    println!();
    println!("主題：{}", topic);
    if !constraints.is_empty() {
        println!("約束：{}", constraints.join("; "));
    }
    println!("---");
    println!();

    // 建立 REPL 狀態
    let mut state = GemmaREPLState::new(topic, constraints.clone());
    state.controller.set_task(state.topic.clone());

    // 初始化 workspace（確保目錄存在）
    let workspace = Workspace::new();
    let _ = workspace.ensure_dir();

    // 初始化可觀測性系統（v0.6 新增）
    let obs_logger = ObservableLogger::new();
    let _ = obs_logger.ensure_dirs();

    // 初始化各類日誌
    let conv_logger = ConversationLogger::with_topic(obs_logger.logs_dir(), &state.topic)
        .expect("無法建立對話日誌");
    let phase_logger = PhaseTransitionLogger::new(obs_logger.logs_dir())
        .expect("無法建立階段轉換日誌");
    let constraint_logger = ConstraintChangeLogger::new(obs_logger.logs_dir())
        .expect("無法建立約束變化日誌");
    let snapshot_logger = SnapshotLogger::new(obs_logger.snapshots_dir());
    let mut session_logger = SessionSummaryLogger::new(obs_logger.logs_dir(), &state.topic)
        .expect("無法建立 session 摘要");

    // 記錄初始約束（如果有）
    for c in state.constraint_manager.get_all() {
        let _ = constraint_logger.log_added(c);
    }

    // 檢查 Ollama 是否可用
    let rt = tokio::runtime::Runtime::new().expect("無法建立 tokio runtime");
    let ollama_ready = rt.block_on(state.controller.health_check());

    if !ollama_ready {
        println!("警告：無法連接到 Ollama 服務 (http://localhost:11434)");
        println!("請確認 Ollama 正在運行，或許要先執行：ollama serve");
        println!();
    } else {
        println!("✓ Ollama 連線成功\n");
    }

    // 進入對話迴圈
    println!("開始與 gemma4 對話（輸入 'quit' 結束，'help' 顯示說明）\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().lock().read_line(&mut input) {
            Ok(0) => break, // EOF
            Err(_) => break,
            Ok(_) => {}
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // 處理特殊指令
        match input {
            "quit" | "exit" | "q" => {
                println!("再見！");
                break;
            }
            "help" | "?" => {
                print_repl_help();
                continue;
            }
            "status" | "s" => {
                show_current_status(&state);
                continue;
            }
            "mode" => {
                toggle_mode(&mut state);
                continue;
            }
            "graph" | "show" => {
                show_graph(&state);
                continue;
            }
            "topics" | "t" => {
                show_topics(&state);
                continue;
            }
            _ if input.starts_with("new-topic ") => {
                let topic_title = input.trim_start_matches("new-topic ").trim();
                if !topic_title.is_empty() {
                    create_new_topic(&mut state, topic_title);
                } else {
                    println!("請提供主題標題，例如：new-topic 睡眠品質");
                }
                continue;
            }
            _ if input.starts_with("switch-topic ") => {
                let topic_id = input.trim_start_matches("switch-topic ").trim();
                switch_topic(&mut state, topic_id);
                continue;
            }
            _ => {}
        }

        // 回合計數器 +1（v0.6 新增）
        state.round_count += 1;
        let current_round = state.round_count;

        // 記錄階段轉換（檢查是否有變化）
        let phase_before = state.controller.phase();
        let node_count_before = state.controller.get_graph().node_count();
        let constraint_count_before = state.constraint_manager.len();

        // 傳送給 gemma4 處理
        println!("\n[gemma4：處理中...]");

        let result = rt.block_on(state.controller.run_round(input));

        // 處理回覆並記錄日誌
        match result {
            Ok(response) => {
                println!("\ngemma4：{}", response);

                // 對話日誌記錄（v0.6 新增）
                let round = ConversationRound {
                    round: current_round,
                    user_input: input.to_string(),
                    gemma_response: response,
                    tool_calls: Vec::new(), // 工具呼叫由 controller 內部處理
                };
                let _ = conv_logger.log_round(&round);

                // 更新 session 統計
                session_logger.increment_rounds();
                // 估算提問數（探索期較多提問）
                if phase_before == QuestionPhase::Exploration {
                    session_logger.add_question();
                    session_logger.add_question(); // 探索期預估每回合 2 個問題
                } else if phase_before == QuestionPhase::Development {
                    session_logger.add_question();
                }
            }
            Err(e) => {
                println!("\n錯誤：{}", e);
                if matches!(e, crate::controller::gemma_controller::ControllerError::MaxIterationsReached(_)) {
                    println!("（已達到最大工具呼叫迭代次數，gemma4 可能需要更長的回應）");
                }
            }
        }

        // 更新 workspace status.xml
        let graph = state.controller.get_graph();
        let _ = workspace.update_status(&graph);

        // 階段轉換日誌記錄（v0.6 新增）
        let phase_after = state.controller.phase();
        if phase_before != phase_after {
            let transition = PhaseTransition {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                from: phase_before,
                to: phase_after,
                reason: format!("節點數達到 {}", graph.node_count()),
                node_count: graph.node_count(),
            };
            let _ = phase_logger.log_transition(&transition);
            session_logger.add_phase_transition();
        }

        // 節點圖快照（v0.6 新增）
        let _ = snapshot_logger.save_snapshot(&graph, current_round);

        // 約束變化檢查並記錄（v0.6 新增）
        let current_constraints = state.constraint_manager.get_all();
        let constraint_count_diff = current_constraints.len() as isize - constraint_count_before as isize;
        if constraint_count_diff != 0 {
            session_logger.add_constraint_change();
        }

        println!();
    }

    // 儲存最終狀態
    let graph = state.controller.get_graph();
    let _ = workspace.save_state(&graph, "final");

    // Session 摘要寫入（v0.6 新增）
    session_logger.set_final_state(
        graph.node_count(),
        graph.edge_count(),
        graph.total_complexity(),
        state.controller.phase().name(),
    );
    let _ = session_logger.write_summary();

    println!("狀態已儲存到 workspace");
    println!("日誌已儲存到 {}", obs_logger.logs_dir().display());
}

/// 印出 REPL 說明
fn print_repl_help() {
    println!("=== Gemma REPL 指令說明 ===");
    println!();
    println!("一般指令：");
    println!("  <任何文字>      - 傳送給 gemma4 分析");
    println!("  quit / exit / q - 結束對話");
    println!("  help / ?        - 顯示此說明");
    println!();
    println!("狀態指令：");
    println!("  status / s      - 顯示當前推理圖狀態");
    println!("  graph / show    - 顯示推理圖結構");
    println!("  mode            - 切換發散/收斂模式");
    println!();
    println!("主題指令（v0.7 新增多主題並行）：");
    println!("  topics / t      - 顯示所有主題");
    println!("  new-topic <標題> - 建立新主題");
    println!("  switch-topic <ID/關鍵字> - 切換到指定主題");
    println!();
    println!("gemma4 會自動使用工具操作推理圖：");
    println!("  - diverge()     - 發散生成子節點");
    println!("  - converge()    - 收斂刪除低分節點");
    println!("  - save()        - 儲存狀態");
    println!("  - load()        - 載入狀態");
    println!("  - output()      - 產出檔案");
    println!("  - status()      - 查詢狀態");
    println!();
}

/// 顯示當前狀態
fn show_current_status(state: &GemmaREPLState) {
    let graph = state.controller.get_graph();
    let mode = state.controller.mode();
    let phase = state.controller.phase();

    println!("\n=== 目前狀態 ===");
    println!("主題：{}", state.topic);
    println!("階段：{}（節點數：{}）", phase.name(), graph.node_count());
    println!("模式：{}",
        match mode {
            ControllerMode::Diverge => "發散（探索）",
            ControllerMode::Converge => "收斂（聚焦）",
        }
    );
    println!("節點數：{}", graph.node_count());
    println!("邊數：{}", graph.edge_count());
    println!("複雜度：{:.2}", graph.total_complexity());
    println!();
}

/// 切換模式
fn toggle_mode(state: &mut GemmaREPLState) {
    let current = state.controller.mode();
    let new_mode = match current {
        ControllerMode::Diverge => ControllerMode::Converge,
        ControllerMode::Converge => ControllerMode::Diverge,
    };
    state.controller.set_mode(new_mode);

    let mode_str = match new_mode {
        ControllerMode::Diverge => "發散（探索）",
        ControllerMode::Converge => "收斂（聚焦）",
    };
    println!("模式切換為：{}", mode_str);
}

/// 顯示圖結構（使用視覺化面板）
fn show_graph(state: &GemmaREPLState) {
    let graph = state.controller.get_graph();
    let phase = state.controller.phase();

    if graph.node_count() == 0 {
        println!("{}", state.visual.format(crate::cli::visual::Color::Yellow, crate::cli::visual::Style::Normal, "（圖是空的）\n"));
        return;
    }

    // 如果有多個主題，分別顯示
    if graph.topics.len() > 1 {
        state.visual.display_multi_topic_graph(&graph);
    } else {
        state.visual.display_graph(&graph, phase);
    }
}

/// 顯示所有主題（v0.7 新增）
fn show_topics(state: &GemmaREPLState) {
    let graph = state.controller.get_graph();
    let topics = graph.get_topics();

    if topics.is_empty() {
        println!("{}", state.visual.format(crate::cli::visual::Color::Yellow, crate::cli::visual::Style::Normal, "（尚無主題）\n"));
        return;
    }

    println!();
    println!("{}", state.visual.format(crate::cli::visual::Color::White, crate::cli::visual::Style::Bold, &"═".repeat(50)));
    println!("  {}", state.visual.format(crate::cli::visual::Color::Cyan, crate::cli::visual::Style::Bold, "主題列表"));
    println!("{}", state.visual.format(crate::cli::visual::Color::White, crate::cli::visual::Style::Bold, &"═".repeat(50)));

    for topic in topics {
        let is_current = graph.current_topic_id.as_ref() == Some(&topic.id);
        let current_marker = if is_current {
            state.visual.format(crate::cli::visual::Color::Green, crate::cli::visual::Style::Bold, " ▶ ")
        } else {
            "   ".to_string()
        };

        let node_count = graph.count_topic_nodes(&topic.id);
        let phase = graph.get_topic_phase(&topic.id);
        let phase_color = match phase {
            TopicPhase::Exploration => crate::cli::visual::Color::White,
            TopicPhase::Development => crate::cli::visual::Color::Cyan,
            TopicPhase::Mature => crate::cli::visual::Color::Magenta,
        };

        println!(
            "{}{} {} [{}] 節點:{}{}",
            current_marker,
            state.visual.format(crate::cli::visual::Color::Yellow, crate::cli::visual::Style::Normal, &topic.title),
            state.visual.format(phase_color, crate::cli::visual::Style::Dim, &format!("({})", phase.name())),
            topic.id.chars().take(8).collect::<String>(),
            node_count,
            if is_current {
                state.visual.format(crate::cli::visual::Color::Green, crate::cli::visual::Style::Dim, " ← 目前")
            } else {
                state.visual.format(crate::cli::visual::Color::DarkGray, crate::cli::visual::Style::Dim, "")
            }
        );
    }
    println!("{}", state.visual.format(crate::cli::visual::Color::White, crate::cli::visual::Style::Bold, &"═".repeat(50)));
    println!();
}

/// 建立新主題（v0.7 新增）
fn create_new_topic(state: &mut GemmaREPLState, title: &str) {
    let mut graph = state.controller.get_graph();
    let topic = graph.add_topic(title.to_string());
    state.controller.sync_graph(graph);

    println!();
    println!("{}", state.visual.format(crate::cli::visual::Color::Green, crate::cli::visual::Style::Bold, &"✓".repeat(25)));
    println!("  新主題已建立：{}", state.visual.format(crate::cli::visual::Color::Yellow, crate::cli::visual::Style::Bold, title));
    println!("  主題 ID：{}", topic.id);
    println!("  根節點：{}", topic.root_node_id);
    println!("{}", state.visual.format(crate::cli::visual::Color::Green, crate::cli::visual::Style::Bold, &"✓".repeat(25)));
    println!();
}

/// 切換目前主題（v0.7 新增）
fn switch_topic(state: &mut GemmaREPLState, topic_id: &str) {
    let mut graph = state.controller.get_graph();

    // 先嘗試用 ID 切換
    if graph.set_current_topic(topic_id) {
        state.controller.sync_graph(graph);
        if let Some(topic) = state.controller.get_graph().get_current_topic() {
            println!("已切換到主題：{}", topic.title);
        }
        return;
    }

    // 如果不是 ID，嘗試當作標題的部分匹配
    // 先收集符合的 topic id 和 title
    let matching_ids: Vec<String> = graph.get_topics()
        .iter()
        .filter(|t| t.title.to_lowercase().contains(&topic_id.to_lowercase()))
        .map(|t| t.id.clone())
        .collect();

    let matching_count = matching_ids.len();

    if matching_count == 1 {
        // 只有一個符合，切換到該主題
        let target_id = &matching_ids[0];
        graph.set_current_topic(target_id);
        state.controller.sync_graph(graph);

        // 重新取得 graph 來顯示主題名稱
        let graph_after = state.controller.get_graph();
        if let Some(topic) = graph_after.get_current_topic() {
            println!("已切換到主題：{}", topic.title);
        }
    } else if matching_count == 0 {
        println!("找不到符合「{}」的主題", topic_id);
        println!("使用 `topics` 指令查看所有主題");
    } else {
        println!("有多個符合「{}」的主題，請使用更詳細的 ID：", topic_id);
        for t in graph.get_topics() {
            if t.title.to_lowercase().contains(&topic_id.to_lowercase()) {
                println!("  - {} [{}]", t.title, &t.id[..8]);
            }
        }
    }
}

/// 遞迴印出節點樹
fn print_node_tree(graph: &Graph, node: &crate::models::Node, depth: usize, is_last: bool) {
    let indent = if depth == 0 {
        String::new()
    } else {
        let prefix = if is_last { "  " } else { "│ " };
        prefix.repeat(depth - 1) + if is_last { "└─" } else { "├─" }
    };

    let status_marker = match node.status {
        crate::models::NodeStatus::Draft => "[D]",
        crate::models::NodeStatus::Active => "[A]",
        crate::models::NodeStatus::Pruned => "[X]",
        crate::models::NodeStatus::Locked => "[L]",
        crate::models::NodeStatus::Failed => "[F]",
    };

    let content_preview = node.content.chars().take(30).collect::<String>();
    println!(
        "{}{} {} (w:{:.2}, c:{:.2}, s:{:.2})",
        indent,
        status_marker,
        content_preview,
        node.weight,
        node.confidence,
        node.score()
    );

    let children = graph.get_children(&node.id);
    let child_count = children.len();
    for (i, child) in children.iter().enumerate() {
        let is_last_child = i == child_count - 1;
        print_node_tree(graph, child, depth + 1, is_last_child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemma_repl_state_new() {
        let state = GemmaREPLState::new(
            "測試主題".to_string(),
            vec!["約束1".to_string(), "約束2".to_string()],
        );
        assert_eq!(state.topic, "測試主題");
        assert_eq!(state.constraint_manager.len(), 2);
        assert!(state.running);
        assert_eq!(state.controller.task(), "測試主題");
    }

    #[test]
    fn test_gemma_repl_state_with_model() {
        let state = GemmaREPLState::with_model(
            "測試主題".to_string(),
            vec![],
            "gemma4:2b",
        );
        assert_eq!(state.topic, "測試主題");
        assert_eq!(state.constraint_manager.len(), 0);
    }

    #[test]
    fn test_toggle_mode() {
        let mut state = GemmaREPLState::new(
            "測試".to_string(),
            vec![],
        );
        assert_eq!(state.controller.mode(), ControllerMode::Diverge);

        toggle_mode(&mut state);
        assert_eq!(state.controller.mode(), ControllerMode::Converge);

        toggle_mode(&mut state);
        assert_eq!(state.controller.mode(), ControllerMode::Diverge);
    }

    #[test]
    fn test_print_repl_help() {
        // 確認不會 panic
        print_repl_help();
    }
}
