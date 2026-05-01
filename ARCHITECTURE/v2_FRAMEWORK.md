# Evolution Compiler v2 — 框架雛型

## 核心願景

> 讓 AI 更容易讀懂「人類想要的」，並產出「人類預期的」軟體

框架本身要能被 AI 讀懂。框架的輸出（Spec）是 AI 和人都讀同一份的合約。

---

## 三層架構

```
┌─────────────────────────────────────────────────────────┐
│  Layer 1: 意圖解析層（Intent Parser）                    │
│  把模糊描述 → 結構化 Spec                                │
│  不需要 AI，用規則 + keyword 做到 80% 覆蓋               │
│  剩下 20% 靠對話補                                     │
└────────────────────┬────────────────────────────────────┘
                     │ Spec { entity, action, state, flow }
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Layer 2: 生成執行層（Generator）                         │
│  把 Spec → 可運行程式                                    │
│  這層用 LLM，讓 AI 充分發揮                             │
│  輸出受 Spec 的 schema 約束                             │
└────────────────────┬────────────────────────────────────┘
                     │ Artifact { code, language, tests }
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Layer 3: 驗證迭代層（Verifier）                         │
│  把產出對照 Spec 檢查，自動提出修改點                     │
│  最多 N 輪，超過就停並標注未解決的 Gap                   │
└─────────────────────────────────────────────────────────┘
```

---

## Layer 1 的核心概念：Spec

Spec 是框架的「語義合約」，定義以下五個維度：

### 1. Entity（實體）
「系統裡有哪些東西」

```json
{
  "entity": [
    { "name": "任務", "fields": ["標題", "截止日", "優先權", "完成狀態"] },
    { "name": "標籤", "fields": ["名稱", "顏色"] }
  ]
}
```

### 2. Action（操作）
「對實體能做什麼」

```json
{
  "action": [
    { "on": "任務", "do": "新增", "requires": ["標題"] },
    { "on": "任務", "do": "刪除", "requires": ["任務ID"] },
    { "on": "任務", "do": "切換完成", "requires": ["任務ID"] },
    { "on": "任務", "do": "篩選", "requires": ["關鍵字"] }
  ]
}
```

### 3. State（狀態）
「系統需要記住什麼」

```json
{
  "state": {
    "tasks": { "type": "array", "init": [] },
    "filter": { "type": "string", "init": "" }
  }
}
```

### 4. Flow（流程）
「操作的順序依賴」

```json
{
  "flow": [
    { "trigger": "新增任務", "then": ["更新 tasks 陣列", "顯示 toast 通知"] },
    { "trigger": "刪除任務", "then": ["更新 tasks 陣列", "顯示 toast 通知"] }
  ]
}
```

### 5. Boundary（邊界）
「系統跟外界的接口」

```json
{
  "boundary": {
    "input": "使用者操作（滑鼠、鍵盤）",
    "output": "DOM 變化 + Toast",
    "persistence": "localStorage"
  }
}
```

---

## Spec 格式的設計原則

1. **AI 讀得懂** — 每個欄位有明確語義，不是自由文字
2. **人類也能讀** — 不需要懂程式也能看懂系統在做什麼
3. **機器可解析** — JSON schema，可以校驗完整性
4. **有預設值** — 不必每次都全部填，沒填的用預設

---

## Layer 2：Generator 的設計

Generator 不直接生成程式碼，而是：

1. 接收 Spec（約束）
2. 接收 Context（技能庫、樣式約定、輸出語言）
3. 輸出 Artifact（語義完整的程式碼區塊）

```python
class Generator:
    def generate(spec: Spec, context: Context) -> Artifact:
        prompt = build_prompt(spec, context)
        code = llm.generate(prompt, schema=OutputSchema)
        return Artifact(code=code, language=spec.target_language)
```

**Context 裡有什麼**
- 可用技能庫（Skills as Reference）
- 樣式主題（Themes）
- 輸出語言約束
- 過去產出過什麼（避免重複破壞既有功能）

---

## Layer 3：Verifier 的設計

Verifier 不是「跑測試」，而是「對照 Spec 檢查」：

```python
def verify(artifact: Artifact, spec: Spec) -> GapReport:
    gaps = []
    for action in spec.action:
        if not implements(artifact, action):
            gaps.append(f"缺少：{action.do} 的實作")
    for entity in spec.entity:
        if not has_fields(artifact, entity):
            gaps.append(f"實體 {entity.name} 缺少欄位：{missing_fields}")
    return GapReport(gaps=gaps, resolved=gaps==[])
```

---

## 迭代 Loop

```
User Input
    │
    ▼
┌──────────────┐     Spec      ┌──────────────┐    Artifact    ┌──────────────┐
│ Intent Parser │ ────────────▶│  Generator   │ ────────────▶│  Verifier    │
│   (Layer 1)   │              │   (Layer 2)  │               │   (Layer 3)  │
└──────────────┘              └──────────────┘               └──────┬───────┘
                                                                     │
                                                       Gap Report    │
                                                                     │ gaps > 0
                                                                     ▼
                                                              最多 N 輪
                                                              (例如 3 輪)
                                                                     │
                                                             gaps == 0
                                                                     ▼
                                                                  交付 Artifact
```

---

## 與舊版本的差異

| 維度 | v1（Skill 拼接） | v2（Spec-Driven） |
|---|---|---|
| 核心單元 | Skill（靜態模板） | Spec（動態語義描述） |
| LLM 使用 | ❌ 不使用 | ✅ Generator 層核心 |
| 新需求支援 | 需先寫對應 Skill | 直接生成 |
| 驗證 | QA Checker（規則比對） | Verifier（Spec 覆蓋檢查） |
| 技能庫 | 拼接材料 | Context 參考 |
| 歧義處理 | 無 | Intent Parser 標注不確定處 |

---

## 待定義

- Spec 的 JSON schema 詳細欄位定義
- Intent Parser 的規則引擎設計
- Generator 的 prompt template
- Context 的組成與如何從 Skill 庫建構
- 第一個 pilot：用哪個案例測這個新架構
