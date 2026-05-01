# ArXiv Search Skill

## 觸發條件
當用戶請求以下類型任務時自動啟用：
- 「找相關論文」
- 「最新的 AI 研究」
- 「這個領域的 SOTA 是什麼」
- 「幫我搜 arxiv」
- 「論文摘要」

## 方法論

### 1. 轉換查詢
- 把研究主題轉成 2-3 個精確關鍵字
- 可加上領域標籤：cs.AI, cs.LG, cs.CV, cs.CL

### 2. 構建 ArXiv API 查詢
使用 ArXiv API（https://export.arxiv.org/api/query）而非網頁瀏覽：
- 搜索 URL：`https://export.arxiv.org/api/query?search_query=all:KEYWORD&start=0&max_results=5&sortBy=submittedDate&sortOrder=descending`
- 直接用 curl/wgetfetch 獲取 XML/JSON 格式結果

### 3. 解析與摘要
從回應中提取：標題、作者（前三）、摘要（第一段）、發布日期、arXiv ID

### 4. 優先排序
- 新論文優先（最近 1-2 年）
- 知名機構/作者

## 輸出格式
```
## ArXiv 搜尋：<主題>

### 精選論文

**1. [論文標題](URL)**
- 作者：xxx, xxx
- 發布：YYYY-MM-DD
- 領域：cs.XX
- 摘要：<核心貢獻 2-3 句>

**2. [論文標題](URL)**
...

### 快速結論
<該領域現狀 2-3 句>
```

## 質量清單
- [ ] 使用 ArXiv API（非網頁爬蟲）
- [ ] 至少 3 篇論文
- [ ] 每篇有標題、URL、作者、摘要
- [ ] 標註發布日期
- [ ] 按相關性/新舊排序
- [ ] 總回覆 < 400 字
