//! Planner CLI — 任務規劃與分工決策命令行工具
//!
//! 用法：
//!   cargo run --bin planner -- "幫我建一個庫存管理系統"
//!   cargo run --bin planner -- --interactive
//!
//! 輸出：JSON Manifest

use evolution_os::planner::Manifest;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("用法: cargo run --bin planner -- <任務描述>");
        eprintln!("       cargo run --bin planner -- --interactive");
        std::process::exit(1);
    }

    // --interactive 模式
    if args[1] == "--interactive" || args[1] == "-i" {
        println!("📋 Evolution Planner — 互動模式");
        println!("輸入你的任務描述（空白行結束）：");
        let mut task = String::new();
        loop {
            let mut line = String::new();
            use std::io::Read;
            match std::io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.trim().is_empty() {
                        break;
                    }
                    task.push_str(&line);
                }
                Err(_) => break,
            }
        }
        if task.trim().is_empty() {
            eprintln!("任務描述為空");
            std::process::exit(1);
        }
        run_planner(&task);
        return;
    }

    // 直接模式：剩下的 args 組合成任務描述
    let task = args[1..].join(" ");
    if task.trim().is_empty() {
        eprintln!("任務描述為空");
        std::process::exit(1);
    }
    run_planner(&task);
}

fn run_planner(task: &str) {
    println!("\n🔍 分析任務：{}\n", task);
    println!("{}", "=".repeat(60));

    let manifest = Manifest::from_task(task);

    // 輸出摘要
    println!("\n📊 複雜度指標");
    println!("  推理分支數：{}", manifest.complexity.reasoning_branches);
    println!("  領域多樣性：{}", manifest.complexity.domain_diversity);
    println!("  語境複雜度：{:.2}", manifest.complexity.context_complexity);

    println!("\n🔀 分工模式");
    let mode_str = match manifest.work_mode {
        evolution_os::planner::WorkMode::Solo => "Solo（單一節點）",
        evolution_os::planner::WorkMode::Fork => "Fork（多節點分工）",
    };
    println!("  模式：{}", mode_str);
    println!("  理由：{}", manifest.dispatch.rationale);
    println!("  預估節點數：{}", manifest.dispatch.estimated_nodes);

    if !manifest.dispatch.domain_tags.is_empty() {
        println!("  領域標籤：{:?}", manifest.dispatch.domain_tags);
    }

    println!("\n📋 需求項目（{} 項）", manifest.requirements.len());
    for req in &manifest.requirements {
        println!("  [{}] {} ({})", req.priority, req.requirement, req.domain);
    }

    if !manifest.questions.is_empty() {
        println!("\n❓ 待確認問題（{} 項）", manifest.questions.len());
        for q in &manifest.questions {
            println!("  {}: {} ({})", q.id, q.question, q.category);
            println!("       影響：{}", q.impact);
        }
    } else {
        println!("\n✅ 所有需求已確認，無待決問題");
    }

    if manifest.work_mode == evolution_os::planner::WorkMode::Fork {
        println!("\n👥 預估節點結構（{} 個）", manifest.estimated_nodes.len());
        for node in &manifest.estimated_nodes {
            print!("  {}（{}）", node.id, node.role);
            if node.depends_on.is_empty() {
                println!(" - 無依賴");
            } else {
                println!(" → 依賴 {:?}", node.depends_on);
            }
            println!("    處理：{:?}", node.handles);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("\n📄 完整 JSON Manifest：\n");

    match manifest.to_json() {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("JSON 輸出失敗：{}", e),
    }
}