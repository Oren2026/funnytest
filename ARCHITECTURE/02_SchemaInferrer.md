# ② Schema Inferrer（資料結構推斷器）

**檔案：** `software/nodes/schema_inferrer.py`

## 職責

根據 `IntentProfile.type` 推斷資料模型欄位集合。決定了 CRUD 的表單欄位、表格欄位、以及可編輯屬性。

## 輸入

```python
IntentProfile  # 來自 ① Intent Classifier
```

## 輸出

```python
List[Dict]  # 欄位定義列表，每個 Dict 如：
{
    "name": str,        # 英文欄位名（camelCase）
    "label": str,       # 中文顯示標題
    "type": str,        # "text" | "badge" | "date" | "checkbox" | "action" | "sortable"
    "required": bool,   # 是否必填
    "editable": bool,   # 是否可編輯
    "options": List[str], # badge/select 類型的選項
    "placeholder": str,
    "default": str,
}
```

## 控制邏輯（分支規則）

### CRUD 分支（最常用）

1. **解析 `context` 中的明確欄位**：`（XX、XX、XX）` 格式 → 正則提取
   - 第一欄 → `title`
   - 其餘欄位根據名稱推斷類型（category/stockStatus/author/quantity/price/contact）

2. **無明確欄位** → 根據 `entities[0]` 調整 title label：
   - 任務 → "任務名稱"
   - 庫存 → "商品名稱"
   - 客戶 → "客戶名稱"
   - 書籍 → "書名"

3. **根據實體加欄位**：
   - 任務/待辦/工作 → 加 `priority`（badge）、`status`（badge）
   - 任務/待辦/工作/庫存 → 加 `dueDate`（date）

4. **通用欄位**：`id`、`createdAt`、`updatedAt`、`actions`

### Dashboard 分支

返回：metric, value, change, trend, chartType

### Game 分支

返回：score, level, lives, gameState, actions

### Tool 分支（根據實體進一步分支）

- 計時/倒數 → time, result, actions
- 計算 → num1, num2, result, actions
- 一般 → input, result, actions

### API 分支

返回：endpoint, method, authRequired, description, actions

## 欄位類型說明

| type 值 | 說明 | 在 Composer 中的處理 |
|---------|------|---------------------|
| `text` | 文字輸入 | `<input type="text">` |
| `badge` | 狀態標籤 | `<select>` + badge 樣式 |
| `date` | 日期 | `<input type="date">` |
| `checkbox` | 勾選框 | `<input type="checkbox">` |
| `action` | 操作按鈕 | 不產生 input，純顯示 |
| `sortable` | 可排序欄位 | `<th>` 含 `data-sort` 屬性 |

## 已知缺口

- `editable: False` 的欄位（id, createdAt, updatedAt）→ `composer.py` 中 `modal-form` 跳過不產 input
- `schema_inferrer.py` 不處理 camelCase/snake_case 轉換 → 直接使用推斷名稱作為 JS 物件 key
- 推斷邏輯全規則，無法理解「暱稱（可選）」vs「名稱（必填）」之類的語意細節
- **CRUD 欄位硬編碼**：`STANDARD_CRUD_FIELDS` 不是動態的，無法自訂擴展
