# ⑥ QA Checker（品質檢查器）

**檔案：** `software/nodes/qa_checker.py`

## 職責

在產出送出去之前做最後品質把關。檢查結構完整性、JS 函式完整性、Schema 欄位覆蓋、安全性、以及 Theme 應用是否正確。

## 輸入

```python
compiled: Dict    # 來自 ⑤ Composer：{code, warnings, metadata}
profile: IntentProfile  # 來自 ① Intent Classifier
schema: List[Dict]     # 來自 ② Schema Inferrer
```

## 輸出

```python
{
    "passed": bool,     # 無 error 級別 issue 即 True
    "issues": [
        QAIssue(
            level="error"|"warning"|"info",
            message=str,
            location=str  # 問題所在：html | js | css | form | security
        )
    ]
}
```

## 控制邏輯（檢查清單）

### 1. 結構完整性（4 項，error 級別）

| 檢查 | 條件 |
|------|------|
| DOCTYPE | `code.startswith("<!DOCTYPE")` 或 `"<!doctype"` |
| `<html>` | `"<html" in code.lower()` |
| `<head>` | `"<head>" in code.lower()` |
| `<body>` | `"<body>" in code.lower()` |

任一失敗 → error，passed = False。

### 2. JS 函式完整性（error 級別）

| 函式 | 條件 |
|------|------|
| `openAdd()` | `function openAdd` 或 `openAdd =` |
| `openEdit()` | 同上 |
| `openDelete()` | 同上 |

缺失 → error。

### 3. Toggle Complete（warning 級別）

若 schema 含 `completed` / `checkbox` 欄位但代碼無 `toggleComplete` → warning。

### 4. Schema 欄位覆蓋（warning 級別）

對每個 `editable=True` 且 `type != "action"` 的欄位，檢查 `code` 中是否有對應的 `field-{name}` id。

```python
for field in schema:
    if field.get("editable") and field.get("type") not in ("action",):
        if f'field-{field["name"]}' not in code:
            issues.append(QAIssue("warning", f"欄位 {name} 沒有對應的 input"))
```

### 5. 安全性（error 級別）

| 問題模式 | 條件 |
|---------|------|
| `eval()` 使用 | `"eval(" in code` |
| `innerHTML` 字串拼接 | `"innerHTML =" in code` 且拼接部分有 `+` |
| `document.write()` | `"document.write(" in code` |

任一 → error，passed = False。

### 6. Theme 應用（warning/info 級別）

若使用 theme skill：
```python
if theme_var_count < 3:
    issues.append(QAIssue("warning", "CSS 變數少於 3 個"))
```

若無 theme skill 且無 CSS 變數：
```python
if "--bg" not in code and "--primary" not in code:
    issues.append(QAIssue("info", "建議加入預設主題"))
```

### 7. JS 語法基礎檢查（error 級別）

對 `<script>` 區塊內容，檢查 `()` `{}` `[]` 是否匹配。

```python
for open_char, close_char in [("(", ")"), ("{", "}"), ("[", "]")]:
    count = 0
    for ch in js:
        if ch == open_char: count += 1
        elif ch == close_char: count -= 1
        if count < 0:  # 提前關閉
            issues.append(QAIssue("error", "括號不匹配"))
```

### 8. 表單 ID（warning 級別）

若使用 `modal-form` 但表單 id 不是 `inventory-form` → warning（handler 可能綁定失敗）。

### 9. Toast 通知（info 級別）

若無 `showToast` → info 提示（UX 反饋問題）。

### 10. Viewport（info 級別）

若無 `width=device-width` → info 提示（響應式問題）。

## 通過標準

```
passed = True  當且僅當 無 error 級別 issue
warning 不影響 passed
info 不影響 passed
```

## 依賴節點

無下游（這是最終節點）。產出直接寫入 `demo/` 目錄。

## 已知缺口

- **Phase 3 Data Flow 未實作**：`validate_data_flow()` 是 stub，需等 context.md 加入 `## 初始資料` 段落才能驗證 seed data 是否正確注入
- **JS 語法檢查太基礎**：只檢查括號匹配，無法發現語意錯誤（如 `STATE.items.find(x => x.id === id)` 少分號）
- **Security 檢查可繞過**：如 `innerHTML = \`...${userInput}...\`` 不會被現有正則捕獲
- **無效能檢查**：如 render() 中無防抖、localStorage 同步寫入等
- **React 無 QA**：目前只對 HTML 輸出做檢查，React 輸出無對應驗證
