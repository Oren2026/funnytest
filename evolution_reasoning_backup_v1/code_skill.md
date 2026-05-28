# code_skill.md — 程式開發規範

> 最後更新：2026-05-05
> 適用於 Evolution Reasoning Tool 專案

---

## 測試要求

### 單元測試

- 每個模組都要有對應的測試檔
- 命名：`[模組名].test.js`、`[模組名]_test.py`、`[模組名].rs`
- 測試檔放在 `tests/` 目錄

### 系統測試

- 必須寫 `test.sh` 做專案測試
- 包含所有單元測試的執行
- 回傳正確的 exit code（0 = 成功）

### test.sh 範例

```bash
#!/bin/bash
set -e

echo "=== Evolution Reasoning Tool Tests ==="

echo "Running unit tests..."
# 單元測試命令

echo "Running system tests..."
# 系統測試命令

echo "All tests passed!"
exit 0
```

---

## 模組化要求

### 1000 行規則

```
單一檔案超過 1000 行 → 強制拆分模組
```

### 拆分原則

- 功能相近的程式放同一模組
- 每個模組有明確的職責
- 模組之間透過公開介面溝通

### 範例結構

```
src/
├── mod_a/
│   ├── __init__.py
│   ├── core.py        # 主要邏輯
│   ├── utils.py       # 工具函式
│   └── mod_a_test.py  # 測試
├── mod_b/
│   ├── __init__.py
│   ├── core.py
│   └── mod_b_test.py
└── main.py
```

---

## 版本規劃

### 存放位置

`_doc/` 目錄下

### 命名規則

```
_doc/v0.1.md   # 小版本前進 0.1
_doc/v0.2.md
_doc/v1.0.md   # 大版本前進 1.0
_doc/v1.1.md
```

### 版本文件內容

每個版本文件要包含：

- 版本號
- 日期
- 新增功能
- 修改內容
- 預定目標

```markdown
# v0.1 — 初始版本

日期：2026-05-05

## 完成

- 專案初始化
- 基礎資料結構定義

## 預定

- [ ] Diverge Engine 實作
- [ ] 節點管理系統
```

---

## 程式語法要求

### Python

- 無語法警告（warnings）
- 使用 `flake8`、`pylint` 檢查
- 型別標註（type hints）

### Rust

```rust
// 如果有未使用的程式碼，用這個允許
#![allow(dead_code, unused)]
```

### JavaScript

- 使用 `eslint` 檢查
- 遵循 standard style 或 airbnb

---

## commit 規範

### 格式

```
<type>: <subject>

<body>
```

### Type 類型

| type | 說明 |
|------|------|
| `feat` | 新功能 |
| `fix` | 修 bug |
| `docs` | 文件更新 |
| `refactor` | 重構 |
| `test` | 測試相關 |
| `chore` | 瑣事 |

### 範例

```bash
git commit -m "feat: add Diverge Engine

- 實作發散邏輯
- 新增複雜度計算
- 新增閾值觸發"
```

---

## 程式碼註解規範

### 原則

- 註解要寫「為什麼」，不是「做了什麼」
- 程式碼本身已表達的邏輯不需要重複註解
- 使用中文繁體

### 範例

```python
# 為什麼這樣設計：
# 因為 Rust 的 ownership 規則，在某種情境下需要先轉移所有權
# 這裡手動管理記憶體是為了避免在熱路徑上產生額外開銷
def process_node(node):
    ...
```

---

## 資料夾結構建議

```
evolution_reasoning/
├── _wiki/           # 文件（名詞解釋）
├── _doc/            # 規劃文件（版本記錄）
├── _book/           # 書籍
├── src/             # 程式碼
│   ├── __init__.py
│   ├── main.py
│   ├── models/      # 資料模型
│   ├── engine/      # 引擎實作
│   └── utils/       # 工具
├── tests/           # 測試
│   ├── unit/
│   └── system/
├── test.sh         # 系統測試脚本
├── README.md        # 專案總覽
└── AGENTS.md        # AI 協作指南
```

---

## 開發流程

```
1. 讀取 _doc/vX.Y.md 確認目前版本目標
2. 實作功能
3. 寫單元測試
4. 跑 test.sh 確認通過
5. 更新 _doc/vX.Y.md（標記完成）
6. commit
7. 規劃下一版本
```

---

## Rust 專案額外規範

### Cargo.toml

- dependency 版本寫死（不用 `*`）
- 必要的 crate：
  - `serde`（序列化）
  - `tokio`（async）
  - `tracing`（日誌）

### 專案初始化

```bash
cargo init --lib
```

### 測試

```bash
cargo test
```

### 文件生成

```bash
cargo doc --open
```
