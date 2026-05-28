//! 簡單 CLI（REPL）介面
//!
//! 提供互動式命令列介面來操作推理圖。

use std::io::{self, BufRead, Write};

use crate::engine::{ComplexityBudget, ConvergeEngine, DivergeEngine};
use crate::models::{Edge, EdgeType, Graph, Node, NodeStatus};

/// REPL 狀態
#[derive(Debug)]
pub struct ReplState {
    /// 推理圖
    pub graph: Graph,
    /// 發散引擎
    pub diverge_engine: DivergeEngine,
    /// 收斂引擎
    pub converge_engine: ConvergeEngine,
    /// 複雜度預算
    pub complexity_budget: ComplexityBudget,
    /// 下一個節點 ID（用於快速引用）
    pub last_node_id: Option<String>,
}

impl Default for ReplState {
    fn default() -> Self {
        ReplState::new()
    }
}

impl ReplState {
    /// 建立新的 REPL 狀態
    pub fn new() -> Self {
        ReplState {
            graph: Graph::new(),
            diverge_engine: DivergeEngine::new(),
            converge_engine: ConvergeEngine::new(),
            complexity_budget: ComplexityBudget::new(),
            last_node_id: None,
        }
    }
}

/// 啟動 REPL
pub fn run_repl() {
    let mut state = ReplState::new();

    println!("=== Evolution Reasoning Tool v0.1 ===");
    println!("輸入 'help' 取得指令說明，'quit' 結束程式");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(_) = io::stdin().lock().read_line(&mut input) {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let result = process_command(&mut state, input);
        if let Err(msg) = result {
            println!("錯誤: {}", msg);
        }

        if input == "quit" || input == "exit" {
            println!("再見！");
            break;
        }
    }
}

/// 處理命令
fn process_command(state: &mut ReplState, input: &str) -> Result<(), String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        "help" => {
            print_help();
            Ok(())
        }
        "create" => {
            // create node "內容"
            if parts.len() < 2 {
                return Err("用法: create node <內容>".to_string());
            }
            if parts[1] != "node" {
                return Err("未知的 create 子指令".to_string());
            }
            let content = parts[2..].join(" ");
            let content = content.trim_matches('"').to_string();
            cmd_create_node(state, &content)
        }
        "add" => {
            // add child <父節點ID> <內容>
            if parts.len() < 2 {
                return Err("用法: add child <父節點ID> <內容>".to_string());
            }
            if parts[1] != "child" {
                return Err("未知的 add 子指令".to_string());
            }
            if parts.len() < 3 {
                return Err("用法: add child <父節點ID> <內容>".to_string());
            }
            let parent_id = parts[2];
            let content = if parts.len() > 3 {
                parts[3..].join(" ").trim_matches('"').to_string()
            } else {
                "新節點".to_string()
            };
            cmd_add_child(state, parent_id, &content)
        }
        "diverge" => {
            // diverge <節點ID> <數量>
            if parts.len() < 2 {
                return Err("用法: diverge <節點ID> [數量]".to_string());
            }
            let node_id = parts[1];
            let count: i32 = if parts.len() > 2 {
                parts[2].parse().map_err(|_| "數量必須是數字")?
            } else {
                3
            };
            cmd_diverge(state, node_id, count)
        }
        "converge" => {
            // converge [閾值]
            let threshold: Option<f64> = if parts.len() > 1 {
                Some(parts[1].parse().map_err(|_| "閾值必須是數字")?)
            } else {
                None
            };
            cmd_converge(state, threshold)
        }
        "show" => {
            // show graph
            if parts.len() < 2 {
                return Err("用法: show graph | status".to_string());
            }
            match parts[1] {
                "graph" => {
                    cmd_show_graph(state);
                    Ok(())
                }
                "status" => {
                    cmd_show_status(state);
                    Ok(())
                }
                _ => Err("未知的 show 子指令".to_string()),
            }
        }
        "node" => {
            // node <節點ID>
            if parts.len() < 2 {
                return Err("用法: node <節點ID>".to_string());
            }
            cmd_show_node(state, parts[1])
        }
        "lock" => {
            // lock <節點ID>
            if parts.len() < 2 {
                return Err("用法: lock <節點ID>".to_string());
            }
            cmd_lock_node(state, parts[1])
        }
        "prune" => {
            // prune <節點ID>
            if parts.len() < 2 {
                return Err("用法: prune <節點ID>".to_string());
            }
            cmd_prune_node(state, parts[1])
        }
        "quit" | "exit" => {
            Ok(())
        }
        _ => Err(format!("未知的指令: {}，輸入 'help' 取得說明", parts[0])),
    }
}

/// 印出說明
fn print_help() {
    println!("=== Evolution Reasoning Tool 指令說明 ===");
    println!();
    println!("節點操作:");
    println!("  create node <內容>         - 建立新節點");
    println!("  add child <父節點ID> <內容> - 加入子節點");
    println!("  node <節點ID>              - 顯示節點詳細資訊");
    println!("  lock <節點ID>             - 鎖定節點");
    println!("  prune <節點ID>            - 刪除節點");
    println!();
    println!("發散/收斂:");
    println!("  diverge <節點ID> [數量]    - 發散生成子節點（預設數量: 3）");
    println!("  converge [閾值]            - 收斂刪除低分節點");
    println!();
    println!("顯示:");
    println!("  show graph                - 顯示圖結構");
    println!("  show status              - 顯示狀態統計");
    println!();
    println!("其他:");
    println!("  help                     - 顯示此說明");
    println!("  quit                     - 結束程式");
}

/// 建立新節點
fn cmd_create_node(state: &mut ReplState, content: &str) -> Result<(), String> {
    let step = (state.graph.node_count() as i32) + 1;
    let node = Node::new(content.to_string(), step);
    let node_id = node.id.clone();
    state.graph.add_node(node);
    state.last_node_id = Some(node_id.clone());
    println!("已建立節點: {}", node_id);
    println!("  內容: {}", content);
    println!("  步驟: {}", step);
    Ok(())
}

/// 加入子節點
fn cmd_add_child(state: &mut ReplState, parent_id: &str, content: &str) -> Result<(), String> {
    let parent = state
        .graph
        .get_node(parent_id)
        .ok_or_else(|| format!("找不到節點: {}", parent_id))?
        .clone();

    let step = parent.step + 1;
    let weight = parent.weight * 0.8;
    let confidence = parent.confidence * 0.9;

    let child = Node::new_with(
        content.to_string(),
        step,
        weight,
        confidence,
        0.0,
    );

    let child_id = child.id.clone();
    let edge = Edge::new_with_weight(
        parent_id.to_string(),
        child_id.clone(),
        EdgeType::Reasoning,
        weight,
    );

    state.graph.add_node(child);
    state.graph.add_edge(edge);
    state.last_node_id = Some(child_id.clone());

    println!("已加入子節點: {}", child_id);
    println!("  內容: {}", content);
    println!("  父節點: {}", parent_id);
    Ok(())
}

/// 發散
fn cmd_diverge(state: &mut ReplState, node_id: &str, count: i32) -> Result<(), String> {
    let children = state.diverge_engine.diverge(&mut state.graph, node_id, count, None);

    if children.is_empty() {
        return Err(format!(
            "無法對節點 {} 發散（節點不存在或狀態不允許）",
            node_id
        ));
    }

    println!("已發散生成 {} 個子節點:", children.len());
    for child in &children {
        println!("  - {} (weight: {:.2}, confidence: {:.2})", child.id, child.weight, child.confidence);
    }

    Ok(())
}

/// 收斂
fn cmd_converge(state: &mut ReplState, threshold: Option<f64>) -> Result<(), String> {
    let pruned = state.converge_engine.converge(&mut state.graph, threshold);

    if pruned.is_empty() {
        println!("沒有節點需要刪除");
    } else {
        println!("已刪除 {} 個節點:", pruned.len());
        for id in &pruned {
            println!("  - {}", id);
        }
    }

    Ok(())
}

/// 顯示圖結構
fn cmd_show_graph(state: &mut ReplState) {
    let roots = state.graph.get_root_nodes();

    if roots.is_empty() {
        println!("（圖是空的）");
        return;
    }

    println!("=== 推理圖結構 ===");
    for root in roots {
        print_node_tree(state, root, 0, true);
    }
    println!();
}

/// 遞迴印出節點樹
fn print_node_tree(state: &ReplState, node: &Node, depth: usize, is_last: bool) {
    let indent = if depth == 0 {
        "".to_string()
    } else {
        let prefix = if is_last { "  " } else { "│ " };
        prefix.repeat(depth - 1) + if is_last { "└─" } else { "├─" }
    };

    let status_marker = match node.status {
        NodeStatus::Draft => "[D]",
        NodeStatus::Active => "[A]",
        NodeStatus::Pruned => "[X]",
        NodeStatus::Locked => "[L]",
    };

    println!(
        "{}{} {} (w:{:.2}, c:{:.2}, s:{:.2})",
        indent,
        status_marker,
        node.content.chars().take(30).collect::<String>(),
        node.weight,
        node.confidence,
        node.score()
    );

    let children = state.graph.get_children(&node.id);
    let child_count = children.len();
    for (i, child) in children.iter().enumerate() {
        let is_last_child = i == child_count - 1;
        print_node_tree(state, child, depth + 1, is_last_child);
    }
}

/// 顯示狀態統計
fn cmd_show_status(state: &mut ReplState) {
    println!("=== 狀態統計 ===");
    println!("節點數量: {}", state.graph.node_count());
    println!("邊數量: {}", state.graph.edge_count());
    println!("總複雜度: {:.2}", state.graph.total_complexity());
    println!(
        "複雜度預算: {:.2} / {:.2}",
        state.complexity_budget.current_complexity,
        state.complexity_budget.max_complexity
    );
    println!();

    println!("各狀態節點:");
    let nodes = state.graph.get_all_nodes();
    let mut counts = [0usize; 4];
    for node in nodes {
        let idx = match node.status {
            NodeStatus::Draft => 0,
            NodeStatus::Active => 1,
            NodeStatus::Pruned => 2,
            NodeStatus::Locked => 3,
        };
        counts[idx] += 1;
    }
    println!("  草稿 (Draft): {}", counts[0]);
    println!("  活躍 (Active): {}", counts[1]);
    println!("  已刪除 (Pruned): {}", counts[2]);
    println!("  鎖定 (Locked): {}", counts[3]);
}

/// 顯示節點詳細資訊
fn cmd_show_node(state: &ReplState, node_id: &str) -> Result<(), String> {
    let node = state
        .graph
        .get_node(node_id)
        .ok_or_else(|| format!("找不到節點: {}", node_id))?;

    println!("=== 節點資訊 ===");
    println!("ID: {}", node.id);
    println!("內容: {}", node.content);
    println!("步驟: {}", node.step);
    println!("權重: {:.4}", node.weight);
    println!("信心度: {:.4}", node.confidence);
    println!("複雜度貢獻: {:.4}", node.complexity);
    println!("分數: {:.4}", node.score());
    println!("狀態: {:?}", node.status);
    println!("父節點邊: {:?}", node.parent_edges);
    println!("子節點邊: {:?}", node.child_edges);

    // 顯示父節點
    let parents = state.graph.get_parents(node_id);
    if !parents.is_empty() {
        println!("\n父節點:");
        for p in parents {
            println!("  - {}", p.content);
        }
    }

    // 顯示子節點
    let children = state.graph.get_children(node_id);
    if !children.is_empty() {
        println!("\n子節點:");
        for c in children {
            println!("  - {}", c.content);
        }
    }

    Ok(())
}

/// 鎖定節點
fn cmd_lock_node(state: &mut ReplState, node_id: &str) -> Result<(), String> {
    if state.graph.lock_node(node_id) {
        println!("已鎖定節點: {}", node_id);
        Ok(())
    } else {
        Err(format!("找不到節點: {}", node_id))
    }
}

/// 刪除節點
fn cmd_prune_node(state: &mut ReplState, node_id: &str) -> Result<(), String> {
    if state.graph.prune_node(node_id) {
        println!("已刪除節點: {}", node_id);
        Ok(())
    } else {
        Err(format!("找不到節點: {}", node_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_state_new() {
        let state = ReplState::new();
        assert_eq!(state.graph.node_count(), 0);
        assert!(state.last_node_id.is_none());
    }

    #[test]
    fn test_print_help() {
        // just verify it doesn't panic
        print_help();
    }

    #[test]
    fn test_cmd_create_node() {
        let mut state = ReplState::new();
        let result = cmd_create_node(&mut state, "測試內容");
        assert!(result.is_ok());
        assert_eq!(state.graph.node_count(), 1);
    }

    #[test]
    fn test_cmd_add_child() {
        let mut state = ReplState::new();
        cmd_create_node(&mut state, "父節點").unwrap();
        let parent_id = state.last_node_id.clone().unwrap();

        let result = cmd_add_child(&mut state, &parent_id, "子節點");
        assert!(result.is_ok());
        assert_eq!(state.graph.node_count(), 2);
        assert_eq!(state.graph.edge_count(), 1);
    }

    #[test]
    fn test_cmd_diverge() {
        let mut state = ReplState::new();
        cmd_create_node(&mut state, "父親節點").unwrap();
        let parent_id = state.last_node_id.clone().unwrap();

        let result = cmd_diverge(&mut state, &parent_id, 3);
        assert!(result.is_ok());
        assert_eq!(state.graph.node_count(), 4); // 1 parent + 3 children
    }

    #[test]
    fn test_cmd_converge() {
        let mut state = ReplState::new();
        // 加入一些高低分節點
        cmd_create_node(&mut state, "高分節點").unwrap();
        state.graph.get_node_mut(&state.last_node_id.clone().unwrap()).unwrap().weight = 0.9;
        state.graph.get_node_mut(&state.last_node_id.clone().unwrap()).unwrap().confidence = 0.9;

        cmd_create_node(&mut state, "低分節點").unwrap();
        state.graph.get_node_mut(&state.last_node_id.clone().unwrap()).unwrap().weight = 0.1;
        state.graph.get_node_mut(&state.last_node_id.clone().unwrap()).unwrap().confidence = 0.1;

        let result = cmd_converge(&mut state, Some(0.3));
        assert!(result.is_ok());
        // 低分節點被刪除
        assert_eq!(state.graph.node_count(), 1);
    }

    #[test]
    fn test_cmd_lock_node() {
        let mut state = ReplState::new();
        cmd_create_node(&mut state, "測試").unwrap();
        let node_id = state.last_node_id.clone().unwrap();

        let result = cmd_lock_node(&mut state, &node_id);
        assert!(result.is_ok());
        assert_eq!(state.graph.get_node(&node_id).unwrap().status, NodeStatus::Locked);
    }

    #[test]
    fn test_cmd_prune_node() {
        let mut state = ReplState::new();
        cmd_create_node(&mut state, "測試").unwrap();
        let node_id = state.last_node_id.clone().unwrap();

        let result = cmd_prune_node(&mut state, &node_id);
        assert!(result.is_ok());
        assert!(state.graph.get_node(&node_id).unwrap().is_pruned());
    }

    #[test]
    fn test_cmd_show_node() {
        let mut state = ReplState::new();
        cmd_create_node(&mut state, "測試節點").unwrap();
        let node_id = state.last_node_id.clone().unwrap();

        let result = cmd_show_node(&state, &node_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_command_unknown() {
        let mut state = ReplState::new();
        let result = process_command(&mut state, "unknown_command");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("未知的指令"));
    }

    #[test]
    fn test_process_command_empty() {
        let mut state = ReplState::new();
        let result = process_command(&mut state, "");
        assert!(result.is_ok());
    }
}
