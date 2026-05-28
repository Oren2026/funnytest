//! HTTP API 伺服器 + stdin 外部觸發
//!
//! v0.8-C 新增：
//! - HTTP API：查詢 backtrack/checkpoint/hypothesis 狀態
//! - stdin：接收外部任務並執行

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::engine::BacktrackManager;
use crate::export::{query_backtrack, ExportFormat, export_graph, export_hypotheses, export_memory};
use crate::memory::MemoryManager;
use crate::models::Graph;
use crate::tools::{ToolExecutor, ToolRegistry};

// ─────────────────────────────────────────────────────────────────────────────
// HTTP API 伺服器
// ─────────────────────────────────────────────────────────────────────────────

/// 啟動 HTTP API 伺服器
pub fn run_api_server(port: u16) {
    println!("[Evolution HTTP API] 啟動中，port {}...", port);
    println!("[Evolution HTTP API] 端點：");
    println!("  GET /status              - 查詢狀態摘要");
    println!("  GET /backtrack/<resource> - checkpoints|failures|hypotheses|summary");
    println!("  GET /export/graph?format=  - yaml|json|dsl");
    println!("  GET /export/memory?format= - yaml|json|dsl");
    println!();

    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[Evolution HTTP API] 綁定失敗: {}", e);
            return;
        }
    };

    println!("[Evolution HTTP API] 監聽中 {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("[Evolution HTTP API] 連線錯誤: {}", e);
            }
        }
    }
}

/// 處理單一 HTTP 連線
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let path = extract_path(&request);

    let response = match path.as_str() {
        "/" | "/status" => build_json_response(&status_page()),
        p if p.starts_with("/backtrack/") => {
            let resource = p.trim_start_matches("/backtrack/");
            build_json_response(&query_backtrack_resource(resource))
        }
        p if p.starts_with("/export/graph") => {
            let format = extract_query_param(&request, "format");
            build_export_graph_response(&format)
        }
        p if p.starts_with("/export/memory") => {
            let format = extract_query_param(&request, "format");
            build_export_memory_response(&format)
        }
        _ => build_json_response(r#"{"error":"not found"}"#),
    };

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// 從 HTTP request 提取路徑
fn extract_path(request: &str) -> String {
    request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// 從 query string 提取參數
fn extract_query_param(request: &str, param: &str) -> String {
    let first_line = request.lines().next().unwrap_or("");
    if let Some(query) = first_line.split_whitespace().nth(1).and_then(|s| s.split('?').nth(1)) {
        for pair in query.split('&') {
            let mut parts = pair.split('=');
            if parts.next() == Some(param) {
                return parts.next().unwrap_or("yaml").to_string();
            }
        }
    }
    "yaml".to_string()
}

/// 建立 JSON HTTP 回應
fn build_json_response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\
         \r\n\
         {}",
        body.len(),
        body
    )
}

/// 建立匯出 HTTP 回應（根據格式調整 Content-Type）
fn build_export_response(body: &str, format: &str) -> String {
    let ct = match format {
        "json" => "application/json",
        "yaml" => "text/yaml",
        "dsl" => "text/plain",
        _ => "text/plain",
    };
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        ct,
        body.len(),
        body
    )
}

fn build_export_graph_response(format: &str) -> String {
    let fmt = match format {
        "json" => ExportFormat::Json,
        "dsl" => ExportFormat::Dsl,
        _ => ExportFormat::Yaml,
    };
    let graph = Graph::new();
    let output = export_graph(&graph, fmt);
    build_export_response(&output, format)
}

fn build_export_memory_response(format: &str) -> String {
    let fmt = match format {
        "json" => ExportFormat::Json,
        "dsl" => ExportFormat::Dsl,
        _ => ExportFormat::Yaml,
    };
    let mem = MemoryManager::new();
    let output = export_memory(&mem, fmt);
    build_export_response(&output, format)
}

fn query_backtrack_resource(resource: &str) -> String {
    let bt = BacktrackManager::new();
    query_backtrack(&bt, resource)
}

fn status_page() -> String {
    format!(
        r#"{{"name":"Evolution Reasoning Tool","version":"0.8","endpoints":["/status","/backtrack/<resource>","/export/graph","/export/memory"]}}"#
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// stdin 外部觸發模式
// ─────────────────────────────────────────────────────────────────────────────

/// 啟動 stdin 外部觸發模式
///
/// 讀取 stdin 中的 JSON 任務，執行後輸出結果到 stdout。
/// 任務格式：{"task": "...", "context": "..."}
pub fn run_stdin_mode() {
    println!("[Evolution stdin] 外部觸發模式已啟動");
    println!("[Evolution stdin] 等待 JSON 任務輸入...");

    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF
                break;
            }
            Ok(_) => {
                let line = input.trim();
                if line.is_empty() {
                    continue;
                }

                // 嘗試解析 JSON
                match serde_json::from_str::<StdinTask>(&line) {
                    Ok(task) => {
                        let result = execute_stdin_task(&task);
                        println!("[RESULT]");
                        println!("{}", result);
                        println!("[END]");
                    }
                    Err(e) => {
                        eprintln!("[Evolution stdin] JSON 解析失敗: {}", e);
                        eprintln!("[RESULT]");
                        eprintln!(r#"{{"error": "invalid JSON", "detail": "{}"}}"#, e);
                        eprintln!("[END]");
                    }
                }
            }
            Err(e) => {
                eprintln!("[Evolution stdin] 讀取失敗: {}", e);
                break;
            }
        }
    }
}

/// 執行 stdin 任務
fn execute_stdin_task(task: &StdinTask) -> String {
    // 使用 ToolExecutor 執行任務
    let executor = ToolExecutor::new();
    let registry = ToolRegistry::new();

    // 根據 task 内容選擇工具
    // 簡單路由：task 字串內容 -> 對應工具
    let result = if task.task.contains("export") || task.task.contains("匯出") {
        execute_export_task(&executor, &task.task)
    } else if task.task.contains("query") || task.task.contains("查詢") {
        execute_query_task(&executor, &task.task)
    } else {
        // 預設：當作 gemma4 對話任務（需要初始化 gemma4）
        format!(r#"{{"status": "ok", "task_received": "{}", "mode": "gemma4_required"}}"#, task.task)
    };

    result
}

/// 執行匯出任務
fn execute_export_task(executor: &ToolExecutor, task: &str) -> String {
    let format = if task.contains("json") {
        "json"
    } else if task.contains("dsl") {
        "dsl"
    } else {
        "yaml"
    };

    let graph = executor.get_graph();
    let output = export_graph(&graph, match format {
        "json" => ExportFormat::Json,
        "dsl" => ExportFormat::Dsl,
        _ => ExportFormat::Yaml,
    });

    format!(
        r#"{{"status": "exported", "format": "{}", "content": "{}"}}"#,
        format,
        escape_json_string(&output)
    )
}

/// 執行查詢任務
fn execute_query_task(executor: &ToolExecutor, task: &str) -> String {
    let resource = if task.contains("checkpoint") {
        "checkpoints"
    } else if task.contains("failure") || task.contains("fail") {
        "failures"
    } else if task.contains("hypothesis") || task.contains("假設") {
        "hypotheses"
    } else {
        "summary"
    };

    let mgr = executor.get_backtrack_manager();
    let result = query_backtrack(&mgr, resource);
    format!(
        r#"{{"status": "ok", "resource": "{}", "data": {}}}"#,
        resource,
        result
    )
}

/// 逸出 JSON 字串中的特殊字元
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            other => result.push(other),
        }
    }
    result
}

/// stdin 任務結構（從 export 模組重新匯出）
#[derive(Debug, serde::Deserialize)]
struct StdinTask {
    task: String,
    #[allow(dead_code)]
    context: Option<String>,
}
