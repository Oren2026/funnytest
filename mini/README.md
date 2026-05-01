# Evolution Compiler — Mini Framework

從骯髒的完整 codebase 中提取的精簡核心。
**只做一件事：自然語法 → 可運作的 HTML 單頁應用。**

---

## 快速開始

```bash
cd ~/Desktop/funnytest/mini

# 執行 L1 測試（代辦事項清單）
python3 test_runner.py

# 直接用自然語法生成
python3 test_runner.py --gen "我要一個代辦事項清單，包含新增、刪除、完成標記"

# 跑指定案例
python3 test_runner.py --case todo-context

# 列出所有可用案例
python3 test_runner.py --list
```

---

## 架構

```
mini/
├── test_runner.py       # 入口，流水線調度
├── contexts/            # 測試案例（.md 格式）
├── skills/              # 技能庫
│   ├── ui/              # UI 構件（badge、button、modal、table...）
│   ├── styles/          # 主題（theme-glass、theme-modern...）
│   └── layout/          # 頁面佈局
├── nodes/               # 流水線節點
│   ├── intent_classifier.py   # 意圖分類
│   ├── schema_inferrer.py      # 推斷資料結構
│   ├── skill_router.py         # 選擇技能
│   ├── dependency_resolver.py  # 解析依賴順序
│   ├── composer.py             # 組合輸出（核心）
│   └── qa_checker.py           # 品質檢查
└── output/             # 生成的 HTML
```

### 流水線

```
自然語法意圖
    ↓
Intent Classifier     →  意圖類型（CRUD/工具/遊戲...）
    ↓
Schema Inferrer       →  資料結構（欄位、類型）
    ↓
Skill Router          →  選擇需要的技能組合
    ↓
Dependency Resolver   →  排序技能（符合依賴關係）
    ↓
Composer              →  組合 HTML/CSS/JS
    ↓
QA Checker            →  檢查問題
    ↓
HTML 輸出
```

---

## Skill Spec 格式

每個 `.skill` 檔案現在有兩層：

```
1. ## Spec 區（讓 AI 理解合約）
   ## Contract      — 這個技能做什麼
   ## Dependencies — 依賴哪些其他技能
   ## Slots        — 支援哪些插 slot 點
   ## Boundaries   — 操作邊界
   ## Examples     — 用法範例

2. [html]/[react]/[style] 區（讓 Composer 讀取原始碼）
```

Spec-driven composition 流程：
1. Composer's `SkillRegistry` 掃描所有 `.skill` 檔案
2. 解析每個檔案的 `## Slots` 區塊，建立 `slot → skill` 映射表
3. `_generate_page()` 根據 slot name 自動找到對應的 skill
4. 若無 Spec，fallback 到 skill 名稱 matching（向後兼容）

---

## 已知限制（extraction 過程中發現）

### 1. Schema 推斷是 generic
目前 `Schema Inferrer` 對中文意圖的欄位識別有限，經常回退到 `title/dueDate/id/createdAt` 等通用欄位。需要豐富領域關鍵字庫。

### 2. Composer's page layout 是 hardcoded
`_generate_page()` 的 slot 注入邏輯（header/search/content/modal/confirm/toast）是 Composer 內部 hardcoded，未來應改為從 `layout-page.skill` 的 Spec 動態讀取。

### 3. 34/41 skills 仍是舊格式
還有 34 個 skill 未轉 Spec， Composer's `slot_map` 仍是 fallback 機制（不影響功能，但 Spec-driven 鏈未閉合）。

### 4. Phase 3 Data Flow 未實現
`validate_data_flow()` 是 stub，context.md 缺少 `## 初始資料` 區段。

### 5. `skill_library/` 和 `software/skills/` 兩個技能庫重疊
舊版 `skill_library/` 已無用，應評估廢棄。

---

## 提取過程學到的教訓

| 發現 | 意義 |
|------|------|
| Composer's `slot_map` 是 hardcoded fallback | Spec 解析正確但未被主要邏輯使用 |
| `table-data` 和 `search-bar` 都聲明 `filter` slot | slot 職責重疊，需要 `Boundaries` 區分 |
| 6 個 slot 呼叫 (`search`/`content`/`modal`/`header`/`confirm`/`toast`) 沒有一個在 Spec 中直接聲明 | Composer's expected slot names 與 Spec 設計脫節 |
| `theme-glass.skill` 在 `styles/` 而非 `theme/` 子目錄 | 目錄結構與 composer's subdir list 不一致 |
| Phase 3 Data Flow validation 是 stub | 框架有「資料流入」的概念但未實作 |

---

## 下一步擴展方向

1. **Schema Inferrer 強化** — 擴充中文 domain keywords（庫存/電子商務/代辦）
2. **Composer's slot_map 移除** — 完全從 Spec 驅動 slot resolution
3. **Skill rollout** — 繼續把其餘 34 個 skill 轉 Spec 格式
4. **layout-page.skill** — 讓 Composer 從 Spec 動態讀取頁面佈局
5. **Phase 3 Data Flow** — 實作 seed data 驗證
