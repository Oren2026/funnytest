# debug_assist — 錯誤分析與修復

## 觸發條件
當使用者的程式碼出現：
- 錯誤訊息（Error、Exception、SyntaxError）
- 預期外的行為（邏輯錯誤）
- 程式 hang 住或無回應
- 效能問題（太慢、記憶體爆掉）

## 除錯流程

### Step 1：理解錯誤
- 錯誤類型（SyntaxError → 語法問題，Exception → 執行期問題）
- 錯誤訊息第一行（這行通常指出問題所在）
- 錯誤發生在哪一行（line number）

### Step 2：隔離問題
- 錯誤程式碼能否獨立抽出來重現？
- 逐步注釋（comment out）找出是哪一行觸發
- 最小重現範例（Minimal Reproducible Example）

### Step 3：修復與驗證
- 只改最少的程式碼達到目的
- 修復後重新執行，確認錯誤不再出現
- 確認沒有引入新錯誤

## 常見錯誤與修復模式

### Python

| 錯誤 | 原因 | 修復 |
|------|------|------|
| `IndentationError` | 縮排不一致 | 用 4 spaces 統一 |
| `NameError: name 'x' is not defined` | 變數未定義或 typo | 檢查變數名拼寫 |
| `TypeError: unsupported operand` | 型別不合 | 加 `int()` 或 `.strip()` |
| `IndexError: list index out of range` | 索引超界 | 加邊界檢查 `if i < len(list)` |
| `FileNotFoundError` | 路徑錯 | 用 `Path(...).resolve()` 確認存在 |
| `json.JSONDecodeError` | JSON 格式錯 | `print(raw_text[:200])` 先看原始格式 |

### JavaScript

| 錯誤 | 原因 | 修復 |
|------|------|------|
| `Cannot read property 'x' of undefined` | 物件是 undefined | 加可選串聯 `obj?.prop` |
| `SyntaxError: Unexpected token` | JSON 解析錯 | `JSON.parse()` 包 `try/catch` |
| `CORS error` | 跨域限制 | 確認 API 允許 CORS 或用 server-side |
| `Promise is not defined` | 忘包 async | 函式加 `async`，調用加 `await` |

### HTML/CSS

| 問題 | 原因 | 修復 |
|------|------|------|
| 樣式沒生效 | 選擇器錯或 specificity 低 | 檢查 CSS 選擇器、加 `!important` 測試 |
| Flex 排列錯 | `flex-direction` 設反 | 預設是 `row`，要 `column` 記得改 |
| 高度異常 | parent 無固定高度 | 確認 container 有 `height` 或 `min-height` |

## 提供修復時的格式

```
❌ 問題程式碼（貼出有問題的那幾行）

原因：[一行說明為什麼壞]

✅ 修復後程式碼（只改那幾行，其他不動）

驗證：[如何確認修復成功]
```

## 陷阱

1. **不要只說「這行有錯」，要說「為什麼」**
2. **不要用 `print()` 當除錯**，用 logging 或 breakpoint
3. **不要忽略 stack trace**，往上翻通常有更多 context
4. **不要假設錯誤只在一處**，有時多個問題同時存在
5. **不要快速下結論**，同一個錯誤訊息可能由完全不同的原因造成
