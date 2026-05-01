# html_compiler — 自然語言 → HTML/CSS 輸出

## 觸發條件
當使用者的需求涉及：
- UI 元件（按鈕、表單、卡片、列表）
- 網頁排版或響應式設計
- CSS 樣式或視覺效果（漸層、陰影、動畫）
- 互動行為（click、hover、input）

## 核心原則

### 1. 單一職責元件
每個 block 只專注做一件事，不要把所有功能塞進一個 HTML 檔。

### 2. 語意化 class 名稱
使用 `kebab-case`（如 `card-title`、`btn-primary`），不要用 `div1`、`box123`。

### 3. CSS 分離
內聯 `<style>` 在 `<head>` 內，保持 HTML 結構乾淨。

### 4. 響應式設計
最小支援 `320px` 寬度，使用 flexbox 或 grid，`.container { max-width: 1200px; margin: 0 auto; }`。

## 輸出格式

```html
<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>標題</title>
  <style>
    /* CSS 在這裡 */
  </style>
</head>
<body>
  <!-- HTML 結構在這裡 -->
</body>
</html>
```

## 常見模式

### 按鈕
```html
<button class="btn-primary" onclick="alert('Hello')">點擊我</button>
```
```css
.btn-primary {
  padding: 8px 16px;
  border-radius: 6px;
  border: none;
  background: #3b82f6;
  color: white;
  cursor: pointer;
}
```

### 卡片
```html
<div class="card">
  <div class="card-title">標題</div>
  <div class="card-body">內容</div>
</div>
```

### 表單輸入
```html
<div class="form-group">
  <label class="form-label" for="name">姓名</label>
  <input class="form-input" type="text" id="name" placeholder="輸入姓名">
</div>
```

### Toast 通知
```html
<div class="toast" id="toast">通知內容</div>
```
```javascript
// 顯示 toast
const toast = document.getElementById('toast');
toast.classList.add('show');
setTimeout(() => toast.classList.remove('show'), 3000);
```

## 陷阱

1. **不要用行內 style**（`style="..."`），全部集中到 `<style>` block
2. **不要用 table 排版**，用 flexbox 或 grid
3. **不要 hardcode 寬高**，用 padding / min-width / flex 讓內容自適應
4. **顏色要用變數或 hex**，不要用 `red`、`blue` 這種語意模糊的名稱
5. **不要在 HTML 放太多 JavaScript**，交互邏輯限 10 行以內，超過就提醒要重構

## 質量檢查清單（輸出前必讀）

- [ ] `<!DOCTYPE html>` 存在
- [ ] `<meta name="viewport" ...>` 存在（響應式必備）
- [ ] `<body>` 內只有一個根元素（或一個 `.app-container` 包住所有內容）
- [ ] 所有 CSS class 在 HTML 中都有對應使用
- [ ] 點擊按鈕有綁定 `onclick` 或 `addEventListener`
- [ ] Input 有 `id` 與 `label for` 對應（可及性）
- [ ] 測試：寬度縮小到 375px 是否仍然美觀
