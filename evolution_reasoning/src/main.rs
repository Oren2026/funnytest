//! Evolution Reasoning Tool - 主程式入口
//!
//! 提供 CLI 介面操作推理圖。

use std::process;

use evolution_reasoning::cli;

fn main() {
    // 解析命令列參數
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!("Evolution Reasoning Tool v0.8");
                println!();
                println!("用法:");
                println!("  evolution              - 啟動互動式 REPL（CLI 模式）");
                println!("  evolution --对话       - 啟動 gemma4 對話模式");
                println!("  evolution --api [port] - 啟動 HTTP API 伺服器（預設 port 8080）");
                println!("  evolution --stdin      - 啟動 stdin 外部觸發模式");
                println!("  evolution -h          - 顯示此說明");
                println!();
                println!("REPL 指令（CLI 模式）:");
                println!("  create node <內容>         - 建立新節點");
                println!("  add child <父ID> <內容>   - 加入子節點");
                println!("  diverge <節點ID> [數量]   - 發散生成子節點");
                println!("  converge [閾值]           - 收斂刪除低分節點");
                println!("  show graph                - 顯示圖結構");
                println!("  show status              - 顯示狀態統計");
                println!("  help                     - 顯示詳細說明");
                println!("  quit                     - 結束程式");
                println!();
                println!("gemma4 對話模式指令:");
                println!("  <任何文字>                - 傳送給 gemma4 分析");
                println!("  status / s               - 顯示當前推理圖狀態");
                println!("  graph / show             - 顯示推理圖結構");
                println!("  mode                     - 切換發散/收斂模式");
                println!("  quit                     - 結束對話");
                process::exit(0);
            }
            "--对话" => {
                // 啟動 gemma4 對話模式
                cli::run_gemma_repl();
                process::exit(0);
            }
            "--api" => {
                // 啟動 HTTP API 伺服器（狀態查詢端點）
                let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8080);
                cli::run_api_server(port);
                process::exit(0);
            }
            "--stdin" => {
                // 啟動 stdin 外部觸發模式
                cli::run_stdin_mode();
                process::exit(0);
            }
            _ => {
                eprintln!("未知的參數: {}，使用 -h 取得說明", args[1]);
                process::exit(1);
            }
        }
    }

    // 啟動 CLI REPL（預設）
    cli::run_repl();
}
