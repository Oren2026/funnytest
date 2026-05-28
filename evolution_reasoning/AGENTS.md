# AGENTS.md — AI 協作指南

> 給 subagent 或其他 AI 看過後就知道該做什麼

---

## 專案目標

建立 Evolution Reasoning Tool——一個純 Reasoning 的 AI 輔助工具，用於：
- 問題拆解
- 邏輯規劃
- 發散 / 收斂推理
- 不直接寫 Code（Code 交給專門 AI）

---

## 目前狀態

**v0.7 開發中**

核心功能已實作（v0.1~v0.6），正在朝 v0.7 前進：
- ✅ gemma4 對話介面
- ✅ 提問習慣系統（三階段）
- ✅ 長期記憶系統
- ✅ 可觀測性系統（日誌、快照）
- 🔨 多主題並行（規劃中）
- 🔨 Execution Feedback Loop（規劃中）

**主要缺口**：從「知道」到「做到」的鴻溝——缺乏執行與回饋驗證循環。

---

## 正在做的事

1. v0.7：多主題並行 + Execution Feedback Loop
2. 對齊 code_skill.md / doc_skill.md 規範
3. 補足 _wiki/ 詞條內容

---

## 如何貢獻

### 當你被分配任務時

1. 讀取 [[_doc/v0.7.md]] 了解當前版本目標
2. 讀取 [[doc_skill.md]] 了解文件規範
3. 讀取 [[code_skill.md]] 了解程式規範
4. 查看 `_doc/` 下目前的版本進度
5. 確認你的任務在哪個版本範圍內

### 文件格式要求

- 繁體中文（台灣用語）
- 專有名詞第一次出現時加註英文
- `_wiki/` 下的詞項至少 300 行
- 使用 `[[詞條名]]` 建立內部連結

### 程式格式要求

- 單元測試 + 系統測試（test.sh）
- 超過 1000 行要拆分模組
- 無語法警告

---

## 檔案結構

```
evolution_reasoning/
├── _wiki/           # 詞條（待補充）
├── _doc/            # 版本規劃
│   ├── plan.md
│   ├── todo.md
│   └── v0.1~v0.7.md
├── _book/           # 書籍（尚未開始）
├── src/             # 程式碼
├── tests/           # 測試
├── test.sh          # 系統測試（183 tests）
├── README.md
├── AGENTS.md
├── SPEC.md
├── doc_skill.md     # 文件規範
└── code_skill.md    # 程式規範
```

---

## 關鍵術語

| 術語 | 說明 |
|------|------|
| [[Diverge]] | 發散能力，多方向探索 |
| [[Converge]] | 收斂能力，鎖定有效路徑 |
| [[Node]] | 離散思考單位 |
| [[Complexity Budget]] | 複雜度預算系統 |
| [[Execution Feedback Loop]] | 執行→回饋→調整 的推理循環 |
| [[Multi-Topic]] | 多主題並行追蹤 |

---

## 聯絡人

- 黑皮 (Naro)：需求方
- Hermes Coder：執行 AI
