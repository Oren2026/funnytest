# Hermes Chat App — 系統狀態文件
> 建立日期：2026-04-22 | 更新日期：2026-04-22

---

## 系統架構

```
使用者（瀏覽器）
 └── https://lite-sms-minneapolis-legislature.trycloudflare.com
      └── Hermes Chat App (Flask + Socket.IO)
           ├── 左側面板：Agent 切換 + Session 管理
           ├── 中間面板：Chat 訊息
           └── 右側面板：Log / Agent Skills / Subagents / HTML Preview

資料存放（~/Desktop/funnytest/）
 ├── skill_library/          ← Skill 圖書館（各技能實體）
 ├── agent_skills.json       ← Master Agent ↔ Skill ownership 映射
 ├── subagents.json           ← Subagent 範本（名稱/skills/system_prompt）
 └── hermes-chat/            ← Auth 設定
      └── auth.json            ← 使用者帳號密碼
```

---

## 右側面板 Tab 說明

| Tab | 功能 |
|-----|------|
| Log | 任務執行 log 輸出 |
| Agent Skills | Master Agent（Coder/Research/Creative）各自擁有哪些 Skills |
| Subagents | Subagent 範本管理（新增/編輯/刪除/分配 Skills） |
| HTML Preview | 當 Hermes 輸出「已產出：路徑」時，自動 render HTML 檔案 |

---

## 安全模型

### 目前防護
- HTTP Basic Auth（帳號密碼登入）
- Session Cookie（HttpOnly，Samesite=Lax）
- 預設帳密資訊已從文件中移除 — **架設後立即修改預設帳密**
- Socket.IO 連線也需登入，否則阻斷

### Credential 存放
- 位置：`~/.hermes-chat/auth.json`（權限 600）
- 格式：`{"users": {"帳號": "密碼"}}`（plain text，依賴 filesystem 安全性）
- **嚴禁將真實帳密寫入任何文件或聊天記錄**
- 新增使用者：透過 UI 或 `POST /api/auth/users`（需已登入）

### 未來強化方向
- [ ] HTTPS（目前 tunnel 已具備）
- [ ] 密碼 hash（目前是明文）
- [ ] Cloudflare Access（如果 tunnel 升級到 Zero Trust）
- [ ] 隔離 VM 方案（完全避免暴露本機）

---

## Skills 系統

### 設計概念
Skills 是**圖書館藏書**，可以被任意 Agent 借閱。
Skill 本體存在 `skill_library/<name>/` 目錄，包含：
- `SKILL.md` — 技能說明文件
- `metadata.json` — `{name, description, tags}`

### 現有 Skills
| Name | Description | Tags |
|------|-------------|------|
| html_compiler | 將意圖轉換為 HTML/CSS 輸出 | frontend, code_generation, ui |
| react_compiler | 生成 React 組件和頁面 | frontend, code_generation, react |
| arxiv_search | 搜尋學術論文 | research, academic, search |
| web_search | 網頁資料檢索 | research, web, search |
| code_generation | 通用程式碼生成能力 | coding, development, general |
| debug_assist | 錯誤分析與修復建議 | coding, debug, assist |

### Agent Skill Ownership
- Hermes Coder：`html_compiler, react_compiler, code_generation, debug_assist`
- Hermes Research：`arxiv_search, web_search, code_generation`
- Hermes Creative：`code_generation`

### Skill 生成觸發點
當 Subagent 輸出不滿意時：
1. 分析失敗原因（缺哪類能力）
2. 建議新增 Skill（提出 name + description + tags）
3. 確認後 → Skill 加入 Library → 指定 Subagent 該擁有它

---

## Subagent 系統（持續建設中）

### 設計概念
Subagent 是**短生命周期任務執行者**（Task Agent）。
- 每個 Subagent 有：`id, name, description, skills[], system_prompt`
- 執行流程：Master 分析任務 → 選擇 Subagent → 組合 prompt → 執行 → 結果返回 → Subagent 進程終止
- **不做完就死亡，不留長期狀態**
- 但 `subagents.json` 會記錄 `invocation_log`（歷史，叫用時可參考）

### 現有 Subagent 範本
| Name | Skills | 用途 |
|------|--------|------|
| Code Runner | code_generation, debug_assist | 負責執行和驗證程式碼 |
| UI Tester | html_compiler | 負責測試和驗證 UI 輸出 |

### 資料格式（subagents.json）
```json
[
  {
    "id": "sub_1",
    "name": "Code Runner",
    "description": "負責執行和驗證程式碼",
    "skills": ["code_generation", "debug_assist"],
    "status": "idle",
    "system_prompt": "你是程式碼執行專家。接收任務後，執行並報告結果。",
    "invocation_log": []
  }
]
```

### 持續建設項目
- [ ] Master Agent 主動叫用 Subagent 的 UI/邏輯
- [ ] invocation_log 寫入（每次 Subagent 執行完畢）
- [ ] Skill 自動建議觸發流程
- [ ] Subagent 執行結果的結構化回傳（而非純文字）

---

## Frontend URL
- **外部 URL**：https://lite-sms-minneapolis-legislature.trycloudflare.com
- **本機 URL**：http://localhost:5177
- **Tunnel 方式**：Cloudflare（trycloudflare.com，不保證長期穩定）

---

## 重要技術決策紀錄

### 2026-04-22
1. **工作區隔離**：Hermes Chat App 放在 `~/Desktop/funnytest/`（非 `Oren_own/`），避免汙染主要開發目錄
2. **Skill Library 設計**：Skills 為圖書館概念（共用），Agent 擁有權在 `agent_skills.json`
3. **Subagent 死亡模型**：不做長期存在的人格，純粹任務執行者，用完即棄
4. **安全即時修復**：發現零防護後立即加 HTTP Basic Auth
5. **HTML Preview 自動偵測**：後端傳 `full_response` 到 WebSocket，前端偵測「已產出：路徑」並 fetch render
6. **Session 熱鍵修復**：selectAgent 不再每次都建立新 session，改為復用該 Agent 的最後一個 session

---

## 已知 Bug / 待優化

| 項目 | 說明 | 狀態 |
|------|------|------|
| Session 每次點 Agent 都新增 | ✅ 已修（2026-04-22）|
| HTML Preview 無法 fetch | ✅ 已修（origin 取不到問題）|
| 密碼明文儲存 | ⚠️ 未來改 hash | 待辦 |
| Subagent 無法被 Master 叫用 | ⚠️ 僅做範本管理，未實作叫用 | 待辦 |
| Cloudflare tunnel 穩定性 | ⚠️ trycloudflare.com 不保證長期 | 待辦 |
| Hermes 回應逾時 | ⚠️ 120s → 300s，仍可能不夠 | 待觀察 |

---

## 快速指令

```bash
# 重啟 Server
lsof -ti :5177 | xargs kill -9 2>/dev/null
cd ~/Desktop/funnytest/hermes-chat-app && python3 app.py &

# 看 Server log
tail -f /tmp/hermes.log

# 看 Auth 設定
cat ~/.hermes-chat/auth.json
```
