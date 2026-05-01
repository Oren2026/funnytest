# Web Search Skill

## 觸發條件
當用戶請求以下類型任務時自動啟用：
- 「查一下 XXX」
- 「幫我找 XXX 的資訊」
- 「最新消息」
- 「事實核查」
- 「這個怎麼運作的」

## 方法論

### 1. 明確搜尋目標
- 把請求轉成 2-3 個精確關鍵字
- 排除常見詞（的、了、是）
- 用英文關鍵字通常效果更好

### 2. 執行搜尋（優先使用 API）
- **DuckDuckGo API**：直接用 curl 發送到 `https://api.duckduckgo.io/`
  ```
  curl -s "https://api.duckduckgo.io/?q=KEYWORD&format=json&no_redirect=1"
  ```
- **Google Search API**（如果有 key）：`https://www.googleapis.com/customsearch/v1?q=KEYWORD&key=...&cx=...`
- **Fallback**：如果無 API，使用 `curl` fetch Wikipedia 或權威網站的 RSS/JSON

### 3. 解析與摘要
- 從 JSON 回應提取 AbstractText / RelatedTopics
- 選擇最相關的 2-3 個結果
- 組織成結構化回覆

### 4. 事實核查
- 交叉比對至少 2 個來源
- 明確標示「未確認」資訊
- 不臆測、不過度推論

## 輸出格式
```
## 搜尋結果：<主題>

### 關鍵發現
- **來源1標題** — URL
  簡短摘要（2-3句）

- **來源2標題** — URL
  簡短摘要（2-3句）

### 摘要
<3-5句綜合說明>

### 備註
<未確認或爭議事項>
```

## 質量清單
- [ ] 至少 2 個不同來源
- [ ] 每個來源有標題、URL、摘要
- [ ] 明確標示不確定的資訊
- [ ] 回覆 < 300 字（摘要類）
- [ ] 不添加與搜索主題無關的內容
- [ ] 使用 curl/API 而非瀏覽器
