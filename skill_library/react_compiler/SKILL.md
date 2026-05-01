# React Compiler Skill

## 觸發條件
當用戶請求以下類型任務時自動啟用：
- 「用 React 做」
- 「React 元件」
- 「React Hook」
- 「useState / useEffect」
- 「React 組件」
- 前端框架相關任務

## 產出標準

### 基礎元件結構
```jsx
import React, { useState, useEffect } from 'react';

export default function ComponentName() {
  const [state, setState] = useState(initialValue);

  useEffect(() => {
    // side effects
    return () => {}; // cleanup
  }, [dependencies]);

  return (
    <div className="component">
      {/* JSX */}
    </div>
  );
}
```

### 質量清單
- [ ] 匯入完整的 React API（不用 default import 就不用）
- [ ] 元件名稱大寫開頭
- [ ] `useState` 初始化值型別正確（[] for array, {} for object, "" for string, 0 for number, null for uncertain）
- [ ] `useEffect` 有正確的依賴陣列，清理函式（如果需要）
- [ ] JSX 語法正確（className 而非 class，htmlFor 而非 for）
- [ ] 條件 render 用 `&&` 或三元，不在 JSX 裡直接 if
- [ ] 列表 render 有 key prop
- [ ] 錯誤邊界考慮
- [ ] 響應式考慮（breakpoints if needed）
- [ ] Accessible（aria-label, role, tabIndex if needed）

## 常見模式速查

### 資料 Fetch
```jsx
useEffect(() => {
  fetch(url)
    .then(r => r.json())
    .then(setData)
    .catch(setError);
}, []);
```

### 表單處理
```jsx
const [form, setForm] = useState({});
const handleChange = (e) => setForm({ ...form, [e.target.name]: e.target.value });
const handleSubmit = (e) => { e.preventDefault(); /* submit */ };
```

### Modal/Dialog
```jsx
const [isOpen, setIsOpen] = useState(false);
// render: isOpen && <div className="modal">...</div>
```

## 陷阱
- 不要在 useEffect 裡直接 setState（會造成無限迴圈）
- 不要在條件式裡呼叫 Hook
- 物件作為 useEffect 依賴會造成頻繁觸發，用 useRef 或精確依賴
- 組件內定義子組件每次 render 都會重建
