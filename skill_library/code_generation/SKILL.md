# code_generation — 通用程式碼生成

## 觸發條件
當使用者要求：
- Python / JavaScript / 其他語言的程式碼
- 演算法實現
- 資料處理 / 格式轉換
- API 串接範例
- 工具腳本

## 核心原則

### 1. 最小可執行
程式碼要能直接執行（或只補少許就能跑），不要提供「概念性 pseudocode」。

### 2. 附帶用法範例
每個函式後面加 `if __name__ == "__main__":` 或 `// 測試` 區塊，秀出輸入輸出。

### 3. 型別標註
Python 加 type hints，JavaScript 可能的話加 JSDoc，減少未來 debug 時間。

### 4. 錯誤處理
I/O、網路、檔案操作要有 `try/except` 或 `.catch()`，不要假設永遠成功。

## Python 範例結構

```python
def process_data(data: list[dict], key: str) -> list:
    """
    根據 key 過濾並排序資料。

    參數:
        data: 原始資料 list
        key: 要排序的欄位名稱

    回傳:
        排序後的 list
    """
    try:
        return sorted(
            [item for item in data if key in item],
            key=lambda x: x[key]
        )
    except (KeyError, TypeError) as e:
        print(f"處理錯誤: {e}")
        return []

# 測試
if __name__ == "__main__":
    sample = [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
    print(process_data(sample, "age"))
    # 輸出: [{"name": "Bob", "age": 25}, {"name": "Alice", "age": 30}]
```

## JavaScript 範例結構

```javascript
/**
 * 從 API 取得資料並快取
 * @param {string} url - API 端點
 * @returns {Promise<object>} - JSON 響應
 */
async function fetchWithCache(url) {
  const cacheKey = `cache_${btoa(url)}`;
  const cached = sessionStorage.getItem(cacheKey);
  if (cached) return JSON.parse(cached);

  try {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    sessionStorage.setItem(cacheKey, JSON.stringify(data));
    return data;
  } catch (err) {
    console.error("fetchWithCache 失敗:", err);
    return null;
  }
}
```

## 陷阱

1. **不要用全域變數**，封裝在 function/module 內
2. **不要忽略 edge case**（空輸入、None、empty string）
3. **不要用已廢棄的語法**（如 Python 2 print statement、var）
4. **不要 hardcode 敏感資料**（token、password、api key）
5. **長程式碼要分塊**，每個 block < 50 行，加註解說明目的

## 質量檢查清單

- [ ] 程式碼可直接執行（不需要補寫其他人才能跑）
- [ ] 有輸入輸出範例
- [ ] 錯誤處理存在
- [ ] 無 hardcoded credential
- [ ] 沒有 `print()` 留在正式函式內（除非是工具用途）
