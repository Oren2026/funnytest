# Evolution Reasoning Tool 技術規格

## 版本：v0.1

日期：2026-05-05

---

## 技術選擇

### 程式語言：Rust

- 記憶體安全，編譯時檢查
- 執行效率高
- 適合作為個人工具的底層

### 主要 Library

- `serde`：序列化
- `uuid`：節點 ID
- `rand`：隨機性注入
- `tracing`：日誌
- `tokio`：async runtime

---

## 核心資料結構

### Node

```rust
struct Node {
    id: String,              // UUID
    step: i32,              // 步驟編號
    content: String,         // 節點內容
    weight: f64,             // 權重
    confidence: f64,         // 信心度（0.0 ~ 1.0）
    complexity: f64,         // 複雜度貢獻
    parent_edges: Vec<String>, // 連入的邊
    child_edges: Vec<String>,  // 連出的邊
    status: NodeStatus,
}

enum NodeStatus {
    Draft,    // 草稿
    Active,   // 活躍
    Pruned,   // 已刪除
    Locked,   // 鎖定
}
```

### Edge

```rust
struct Edge {
    id: String,
    from: String,
    to: String,
    edge_type: EdgeType,
    weight: f64,
}

enum EdgeType {
    Reasoning,   // 推理關係
    Constraint,  // 約束關係
    Divergence,  // 分叉關係
}
```

### Graph

```rust
struct Graph {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}
```

---

## 功能範圍

### Must Have（已實作）

1. **核心資料結構**：Node、Edge、Graph
2. **基本操作**：add/remove node/edge, get children/parents
3. **複雜度計算系統**：ComplexityBudget（Complex = a × k × m）
4. **閾值觸發系統**：ThresholdGate
5. **Diverge Engine**：發散生成多個子節點
6. **Converge Engine**：收斂刪除低分節點

### Should Have（已實作）

7. **CLI 介面**：REPL 互動模式
8. **基本視覺化輸出**：文字樹狀圖

### Could Have（未實作）

9. Ollama gemma4 串接
10. 記憶系統
11. 視覺化面板

---

## 驗收標準

- [x] Graph 可以新增/刪除 Node
- [x] Graph 可以新增/刪除 Edge
- [x] 複雜度計算正確
- [x] 閾值觸發邏輯正確
- [x] Diverge Engine 可以生成多個子節點
- [x] Converge Engine 可以刪除低分節點
- [x] CLI 可以輸入基本指令
- [x] CLI 可以顯示節點圖
- [x] `cargo build` 成功，無 warning
- [x] `cargo test` 全部通過
- [x] `test.sh` 執行成功
