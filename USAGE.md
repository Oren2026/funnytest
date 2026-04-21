# Evolution Compiler — 使用說明

## 這是什麼

「能做軟體的軟體」—— 輸入自然語法意圖，輸出完整、可用的軟體。

軟體是 AI 用的工具，輸出要有品質，内部不需要 UI。

## 核心流程

```
自然語法意圖 → Intent Classifier → Schema Inferrer → Skill Router
→ Dependency Resolver → Composer → QA Checker → HTML/React 產出
```

## 快速開始

### 1. 建立測試案例（context.md）

放在 `software/` 下：

```markdown
# Context: <專案名稱>

## 需求描述
我要一個代辦事項清單，功能包含：
- 顯示代辦事項列表（標題、截止日期、優先權）
- 可以新增代辦事項
- 可以刪除代辦事項
- 可以用關鍵字搜尋

## 預期產出
- HTML 單頁應用，功能完整可運作
- 主題：glass（深色毛玻璃風格）

## 難度等級
L1（單一頁面，現有技能可組合覆蓋）

## 技能需求
- layout-header
- table-data
- button-primary
- button-danger
- modal-form
- toast-notify
- search-bar
- badge-status
- theme: glass
```

### 2. 命名規則

- Context 檔案：`software/<name>-context.md`
- 產出 HTML：`demo/<name>-context.html`（自動生成）
- seed 檔案（可選）：`software/<name>.seed`

### 3. 執行測試

```bash
cd software/
python3 test_runner.py --case <name>        # 跑單一案例
python3 test_runner.py --list               # 列出所有案例
```

### 4. 自動產出

test_runner.py 會：
1. 執行完整 pipeline
2. 生成 HTML 到 `demo/`
3. 自動更新 `demo/index.html`（包含所有產出連結）

## 軟體目錄結構

```
evolution_compiler/
  software/           ← 軟體本體（代碼）
    nodes/            ← 6 個節點實作
      composer.py     ← 技能組合引擎
      html_compiler.py
      react_compiler.py
      ...
    skills/           ← 技能庫
      ui/             ← UI 技能（table-data, modal-form, ...）
      styles/         ← 主題（theme-glass, theme-modern, ...）
    test_runner.py    ← L1 測試執行器
  demo/               ← 產出（HTML 文件）
  versions/           ← 版本快照
    v0.9/             ← v0.9 完成狀態
  skills/             ← 共享技能庫（舊位置，相容用）
  knowledge/          ← 知識庫
  USAGE.md           ← 本檔案
```

**重要：software/ = 軟體本體，demo/ = 測試產出，兩者必須分開。**

## 技能庫格式（.skill 檔案）

每個 skill 檔案使用 section markers：

```markdown
# skill: table-data
# depends: badge-status, button-primary
# prohibit: none

[html]
<div class="inventory-table-wrapper">
  <table class="inventory-table">
    <thead><!-- slot:thead --></thead>
    <tbody id="inventory-body"><!-- slot:tbody --></tbody>
  </table>
</div>

[style]
.inventory-table { width: 100%; border-collapse: collapse; }
...

[react]
const Table = ({ items }) => ( ... );
```

**注意**：`[html]` 區塊沒有 `[/html]` 結尾標記。Section 邊界是 `\n[` 或檔案結尾。

## Composer slot injection 約定

- Skill HTML 使用 `data-slot="<name>"` 屬性
- Composer 用 `_inject_slot(html, slot_name, content)` 注入
- 組合順序：`layout-page → header → search → table-data → modal → confirm → toast`

## QA 驗證標準（L1）

12 項檢查：
1. Page title 動態解析
2. Header 標題正確
3. Theme CSS 正確（如 glass 的 `--glass-bg`）
4. backdrop-filter 存在
5. Checkbox 欄位存在
6. toggleComplete 函式存在
7. 初始資料正確（空陣列）
8. completed 欄位值正確
9. Sort 無殘留 placeholder
10. Schema-driven sort direction
11. Search bar 存在
12. Toast system 存在

**第三階段驗證（TODO）**：
- Seed 資料正確注入（不只是結構，是實際 render 出來的資料）
- 新增/編輯/刪除操作正確運作

## 給 AI Agent 的使用框架

當你（AI）要使用 Evolution Compiler 幫用戶生成軟體：

### Step 1：理解意圖
把用戶的自然語法需求轉寫成 `software/<name>-context.md`

### Step 2：選擇意圖類型
- CRUD（代辦事項、庫存、聯絡人）→ L1 適用
- 遊戲、資料視覺化 → 需要更多技能

### Step 3：選擇主題
- `glass` — 毛玻璃深色風格
- `modern` — 現代簡潔
- `brutal` — 粗獷風格
- `soft` — 柔和暖色

### Step 4：執行並驗證
```bash
cd software/
python3 test_runner.py --case <name>
```

### Step 5：交付
- 產出在 `demo/<name>-context.html`
- index.html 自動更新
- Push 前確認 `software/` 和 `demo/` 分開

## 常見問題

**Q：新增功能沒反應？**
A：檢查 `render()` 的 tbody selector 是否匹配 skill 的 table id（目前用 `.inventory-table tbody` class selector）

**Q：modal 裡沒有輸入框？**
A：檢查 `_build_form_from_schema()` 是否正確注入 slot

**Q：Skill 組合順序？**
A：resolve_dependencies 自動處理，但 Composer 注入順序要正確

## 版本維護

- `versions/<ver>/` — 各版本完整快照
- GitHub main — 最新版本
- 升級時：把 current 快照進 `versions/<new_ver>/`，從乾淨狀態開始新版本開發
