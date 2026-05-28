# Evolution Reasoning Tool v0.8

> ⚠️ **開發前必讀**：開始實作前請先閱讀 `code_skill.md` 和 `doc_skill.md`，了解本專案的開發規範。

純 Reasoning 工具框架，專注於發散/收斂推理。

## 功能特色

- **核心資料結構**：Node、Edge、Graph
- **發散引擎（Diverge Engine）**：從單一節點生成多個分支
- **收斂引擎（Converge Engine）**：評估並刪除低分節點
- **複雜度預算系統**：控制推理圖的複雜度上限
- **工具系統**：18 個可呼叫工具（diverge、converge、export_*、query_* 等）
- **HTTP API**：透過 `--api` 啟動，查詢狀態、匯出圖/記憶
- **stdin 外部觸發**：接收 JSON 任務，ToolExecutor 執行

## 安裝

```bash
cargo build --release
```

## 使用方式

### 四種執行模式

```bash
./evolution_reasoning              # REPL 互動模式
./evolution_reasoning --对话         # gemma4 對話模式
./evolution_reasoning --api [port]  # HTTP API 伺服器（預設 8080）
./evolution_reasoning --stdin       # stdin 外部觸發模式
```

### HTTP API 端點

```
GET /                     → 狀態摘要
GET /status               → 狀態摘要
GET /backtrack/checkpoints → 檢查點列表（JSON）
GET /backtrack/failures   → 失敗歷史（JSON）
GET /backtrack/hypotheses → 假設列表（JSON）
GET /backtrack/summary    → 統計摘要（JSON）
GET /export/graph?format=  → 推理圖（yaml/json/dsl）
GET /export/memory?format=→ 長期記憶（yaml/json/dsl）
```

### stdin 外部觸發

```bash
echo '{"task": "create node: 測試", "context": ""}' | ./evolution_reasoning --stdin
```

### REPL 指令

```
create node <內容>         - 建立新節點
add child <父節點ID> <內容> - 加入子節點
diverge <節點ID> [數量]    - 發散生成子節點（預設數量: 3）
converge [閾值]            - 收斂刪除低分節點
show graph                - 顯示圖結構
show status              - 顯示狀態統計
node <節點ID>             - 顯示節點詳細資訊
lock <節點ID>            - 鎖定節點
prune <節點ID>           - 刪除節點
help                     - 顯示指令說明
quit                     - 結束程式
```

## 測試

```bash
# 執行所有測試
cargo test --lib --tests

# 執行系統測試
./test.sh
```

## 專案結構

```
evolution_reasoning/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── models/          # 資料結構（node, edge, graph）
│   ├── engine/          # 引擎（diverge, converge, backtrack）
│   ├── cli/            # CLI（repl, gemma_repl, visual, server）
│   ├── tools/          # 工具系統（registry, executor）
│   ├── ollama/         # Ollama gemma4 客戶端
│   ├── controller/    # Gemma Controller
│   ├── workspace/      # Workspace 持久化
│   ├── memory/        # 長期記憶
│   └── export/         # 匯出工具（json/yaml/dsl）
├── tests/              # 整合測試
│   ├── test_graph.rs
│   ├── test_diverge.rs
│   └── test_converge.rs
├── _doc/               # 版本文件
│   ├── v0.1.md        # 初始版本，核心框架
│   ├── v0.2.md        # Ollama gemma4 整合、工具系統
│   ├── v0.3.md        # gemma4 對話介面
│   ├── v0.4.md        # 提問習慣系統（三階段模型）
│   ├── v0.5.md        # 長期記憶系統、視覺化面板
│   ├── v0.6.md        # 可觀測性系統（日誌、快照）
│   ├── v0.7.md        # 多主題並行、輸出優化
│   └── v0.8.md        # Export Tools + HTTP API + stdin
├── _wiki/              # 詞條文件
├── code_skill.md       # ⚠️ 開發前必讀
├── doc_skill.md        # ⚠️ 文件規範
├── AGENTS.md           # AI 協作指南
└── test.sh             # 系統測試（183 tests passed）
```

## 開發規範摘要

### 開始新任務前

1. 讀 `code_skill.md` — 了解測試、模組化、commit 規範
2. 讀 `doc_skill.md` — 了解文件格式要求
3. 讀 `_doc/v*.md` — 確認目前版本目標
4. 確認你的任務在哪個版本範圍內

### 單元測試位置

- **整合測試**：`tests/` 目錄（test_graph.rs 等）
- **單元測試**：`#[cfg(test)]` 在各 `.rs` 檔案內

### 開發流程

```
1. 讀取 _doc/vX.Y.md 確認版本目標
2. 實作功能
3. 寫測試（整合測試放 tests/，單元測試放 #[cfg(test)]）
4. 跑 test.sh 確認通過
5. 更新 _doc/vX.Y.md（標記完成）
6. commit
7. 規劃下一版本
```

### 1000 行規則

單一 `.rs` 檔案超過 1000 行 → 強制拆分模組

## 版本

| 版本 | 日期 | 說明 |
|------|------|------|
| v0.1 | 2026-05-05 | 初始版本，核心框架建立 |
| v0.2 | 2026-05-07 | Ollama gemma4 整合、工具系統、Workspace |
| v0.3 | 2026-05-07 | gemma4 對話介面（--对话 模式） |
| v0.4 | 2026-05-07 | 提問習慣系統（探索/發展/成熟三階段） |
| v0.5 | 2026-05-07 | 長期記憶系統、CLI 視覺化面板 |
| v0.6 | 2026-05-07 | 可觀測性系統（對話日誌、階段轉換、快照） |
| v0.7 | 2026-05-07 | 多主題並行、輸出優化 |
| v0.8 | 2026-05-07 | Export Tools + HTTP API + stdin 外部觸發 |

---

## 快速連結

- [code_skill.md](code_skill.md) — 程式開發規範
- [doc_skill.md](doc_skill.md) — 文件規範
- [_doc/v0.8.md](_doc/v0.8.md) — 當前版本詳細規格
