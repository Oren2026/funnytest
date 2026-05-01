# Evolution Compiler — Skill Spec 格式標準

## 目的

讓每個 Skill 不是「程式碼區塊」，而是「有語意合約的構件」。
AI 在組裝時，能根據合約判斷「什麼情境用這個 skill」、「輸入輸出是什麼」、「失敗信號是什麼」。

---

## 格式結構

每個 `.skill` 檔案包含以下五個區塊：

```markdown
# skill: <skill_name>

## Contract
## Dependencies
## Slots
## Boundaries
## Examples
```

---

## Contract（語意合約）— 核心區塊

```markdown
## Contract
- **語義承諾**：一句話說清楚這個 skill 做什麼
- **輸入資料格式**：JSON Schema 或自然語言描述
- **輸出語義**：這個 skill 的輸出對調用者代表什麼
- **操作邊界**：這個 skill 承諾做什麼、不做什麼
- **失敗信號**：什麼情況算是這個 skill 失敗了
```

### 範例

```markdown
## Contract
- **語義承諾**：渲染一個可排序的資料表格
- **輸入資料格式**：`Array<{id: number, name: string, category: string, quantity: number, status: string}>`
- **輸出語義**：HTML `<table>` 元素，帶有 thead/tbody slots
- **操作邊界**：
  - ✅ 做：根據 Schema 動態渲染欄位、支援排序切換、顯示空狀態
  - ❌ 不做：新增/刪除/編輯背後的資料邏輯、不處理 API 呼叫
- **失敗信號**：
  - `items` 不是陣列 → 渲染 empty-state slot
  - `schema` 為空 → 拋出 warning 但仍渲染基本結構
```

---

## Dependencies（依賴宣告）

```markdown
## Dependencies
- **依賴**：列出這個 skill 正常運作所需的其他 skills
- **可選依賴**：列出有則更好、無則降級的 skills
- **排斥**：列出不能與這個 skill 共存的 skills
```

### 範例

```markdown
## Dependencies
- **依賴**：`badge-status`（狀態顯示）、`button-primary`（操作按鈕）、`search-bar`（搜尋過濾）
- **可選依賴**：`pagination`（超過一頁時自動顯示分頁）
- **排斥**：無
```

---

## Slots（對外接口）

```markdown
## Slots
- **slot:<name>**：描述這個 slot 的用途、注入什麼內容、誰負責注入
```

### 範例

```markdown
## Slots
- **slot:thead**：欄位標題列，由 Schema Inferrer 根據欄位定義注入靜態標題
- **slot:tbody**：動態資料行，由 Composer 注入 `_renderRows(items, schema)`
- **slot:actions**：操作按鈕區（如「編輯」「刪除」），由 parent skill 或 Composer 注入
- **slot:filter**：搜尋/過濾控制區，由 search-bar skill 注入
```

---

## Boundaries（邊界定義）

```markdown
## Boundaries
- **系統邊界**：這個 skill 屬於系統哪個層次（presentation / business / data）
- **操作邊界**：這個 skill 可以操作什麼、不可以操作什麼
- **狀態邊界**：這個 skill 是否保有狀態（stateful）、還是純函式（stateless）
```

### 範例

```markdown
## Boundaries
- **系統邊界**：Presentation Layer（只處理 UI 渲染，不管資料邏輯）
- **操作邊界**：
  - ✅ 操作：DOM 渲染、使用者互動（點擊排序、切換分頁）
  - ❌ 不操作：localStorage、網路請求、函式級變數
- **狀態邊界**：Stateless（不保有內部狀態，視覺狀態由 UI 操作類處理）
```

---

## Examples（範例）

```markdown
## Examples

### 基本用法
**輸入**：
```json
{
  "items": [{ "id": 1, "name": "螺絲", "quantity": 100 }],
  "schema": [{ "name": "name", "label": "名稱", "type": "text" }]
}
```
**輸出**：
```html
<table>
  <thead><tr><th>名稱</th></tr></thead>
  <tbody><tr><td>螺絲</td></tr></tbody>
</table>
```

### 空資料
**輸入**：`{ "items": [], "schema": [...] }`
**輸出**：`<div class="empty-state">尚無資料</div>`（注入 empty-state slot）

### 排序切換
**觸發**：點擊欄位標題
**行為**：呼叫 `sortBy(field)`，重新注入 sorted items 到 tbody slot
```

---

## 格式轉換原則（舊 → 新）

| 舊格式 | 新格式對應 |
|---|---|
| `# depends: ...` | `## Dependencies` 區塊 |
| `[html]` 區塊內容 | 轉為 `## Examples` 的「輸出」參考 |
| `# prohibit: ...` | 轉為 `## Contract > 操作邊界 > ❌ 不做` |
| slot injection (`<!-- slot:tbody -->`) | 正式定義在 `## Slots` 區塊，標注誰注入什麼 |

---

## 檔案命名

- Spec 格式 skill：`*.skill`（維持副檔名）
- 格式文件：`_SPEC_FORMAT.md`（置於 skills/ 目錄）
- Pilot 優先順序：先改 `table-data.skill` → 驗證後再推廣
