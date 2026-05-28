# todo.md — 待辦事項

> 更新日期：2026-05-05

---

## 規格設計階段

### 系統參數

- [ ] 確認 k（複雜度常數）初始值
- [ ] 確認 MAX_COMPLEXITY（最大複雜度）初始值
- [ ] 確認 CONVERGENCE_THRESHOLD（收斂觸發門檻）
- [ ] 確認 CONFIDENCE_WEIGHT（信心度權重 = 60%）

### 架構細節

- [ ] 定義視覺化面板的具體設計
- [ ] 定義人類 20% 介入點的時機
- [ ] 定義記憶系統的實作方式
- [ ] 定義 gemma4 串接方式

### 文件

- [ ] 撰寫 _wiki/ 詞條（預計詞項：Reasoning, Node, Diverge, Converge, Sublimation, Complexity Budget）
- [ ] 建立 _book/ 目錄結構（如果需要）

---

## 實作階段（v0.2+）

### 核心系統

- [ ] 實作 Node 資料結構
- [ ] 實作 Edge 資料結構
- [ ] 實作 Graph 結構
- [ ] 實作 Diverge Engine
- [ ] 實作 Converge Engine
- [ ] 實作 Complexity Budget System

### 整合

- [ ] 串接 Ollama gemma4
- [ ] 實作記憶系統
- [ ] 實作視覺化面板

### 測試

- [ ] 撰寫單元測試
- [ ] 建立 test.sh
- [ ] 跑通系統測試

---

## 完成

### v0.1

- [x] 討論並確認核心架構
- [x] 建立所有必要文件
- [x] 建立 _doc/ v0.1.md
