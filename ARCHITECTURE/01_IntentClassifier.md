# ① Intent Classifier（意圖分類器）

**檔案：** `software/nodes/intent_classifier.py`

## 職責

將自然語言意圖文字轉換為結構成的 `IntentProfile`。這是管道的第一個節點，決定了後續所有節點的行為基準。

## 輸入

| 欄位 | 類型 | 說明 |
|------|------|------|
| `intent_text` | `str` | 自然語言需求描述（如：「顯示書籍列表，包含書名、作者、分類」） |

## 輸出

```python
@dataclass
class IntentProfile:
    type: IntentType    # 6 種之一
    entities: List[str] # 提取的實體關鍵詞
    actions: List[str]  # 提取的動作關鍵詞
    context: str        # 原始輸入（未處理）
    target: str         # 輸出平台："html" | "react" | "flutter" | "swift"
    theme: str          # 主題偏好："modern" | "glass" | "brutal" | "soft"
```

## IntentType 列舉

| 值 | 觸發關鍵詞 | 典型場景 |
|----|-----------|----------|
| `CRUD` | 代辦、庫存、客戶、商品、資料管理 | 列表 + 新增 + 編輯 + 刪除 |
| `DASHBOARD` | 儀表板、dashboard、統計、數據概覽 | 圖表卡 + 統計卡 |
| `GAME` | 遊戲、game、貪吃蛇、俄羅斯方塊、2048 | Canvas 遊戲 + 分數系統 |
| `TOOL` | 計時器、計算機、密碼產生器、倒數 | 單一工具功能 |
| `API` | REST、API、JWT、認證服務 | 後端 API 路由 |
| `UNKNOWN` | 無匹配 | 預設降級為 CRUD |

## 控制邏輯（決策樹）

```
1. 全文 lowercase
2. TYPE_PATTERNS 優先匹配（GAME > DASHBOARD > API > TOOL > CRUD）
3. 如果 UNKNOWN → ENTITY_PATTERNS 有匹配 → 提升為 CRUD
4. ENTITY_PATTERNS 提取實體（可重複）
5. ACTION_PATTERNS 提取動作
6. Target 推斷：react / (ios|android|app) / swiftui
7. Theme 推斷：glass / dark / soft / brutal
```

### 關鍵詞字典

**ENTITY_PATTERNS（實體提取）**
```python
{
    "任務": ["代辦", "待辦", "任務", "事項", "工作"],
    "書籍": ["書籍", "書", "圖書", "book"],
    "庫存": ["庫存", "商品", "存貨", "物料"],
    "客戶": ["客戶", "顧客", "會員", "使用者", "帳號"],
    "報表": ["報表", "報告", "統計", "數據"],
    "遊戲": ["遊戲", "game", "貪吃蛇", "俄羅斯方塊", "2048", "射擊"],
    "計時": ["計時", "倒數", "stopwatch", "timer"],
    "計算": ["計算", "計算機", "calculator", "單位換算"],
    "密碼": ["密碼", "password", "產生器"],
    "認證": ["登入", "登出", "註冊", "auth", "jwt"],
}
```

**ACTION_PATTERNS（動作提取）**
```python
{
    "新增": ["新增", "建立", "創建", "create", "add"],
    "刪除": ["刪除", "移除", "remove", "delete"],
    "編輯": ["編輯", "修改", "更新", "edit", "update"],
    "查詢": ["查詢", "搜尋", "尋找", "search", "find", "過濾"],
    "列表": ["列表", "清單", "list", "瀏覽"],
    "排序": ["排序", "sort", "由大到小", "由小到大"],
    "匯出": ["匯出", "export", "下載", "download"],
    "圖表": ["圖表", "chart", "視覺化"],
    "統計": ["統計", "statistic", "分析"],
    "完成": ["完成", "completed", "done", "勾選"],
    "審核": ["審核", "審批", "approve"],
    "通知": ["通知", "notification", "推播"],
}
```

## 錯誤處理

- **無匹配**：所有 pattern 都沒命中 → `IntentType.UNKNOWN`，後續降級為 CRUD schema
- **多實體**：同一文字含多個實體關鍵詞 → 全部加入 `entities` 列表

## 依賴節點

**下游：**
- ② Schema Inferrer（接收 `IntentProfile.type` 分支）
- ③ Skill Router（接收完整 `IntentProfile`）
- ⑤ Composer（接收 `profile.theme`）
- ⑥ QA Checker（接收 `profile.type`）

## 已知缺口

- `UNKNOWN` 類型降級為 CRUD 但缺少實體 → schema 推斷會使用預設 "項目"
- Target/theme 只看 keyword，沒有 fallback 到 `profile.context` 語意分析
- 純規則系統，無法處理多意圖混合（如「遊戲 + 排行榜」應同時是 GAME + DASHBOARD）
