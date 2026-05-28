"""節點 1b：LLM Intent Classifier（gemma4:e4b via Ollama）

替換意圖：將 keyword matching 替換為 LLM 推斷，提高泛化能力。
用途：複雜/模糊輸入時使用，簡單輸入仍用 keyword 版（速度快）。
"""
import sys
sys.path.insert(0, '/Users/oren/Library/Python/3.9/lib/python/site-packages')

from .intent_classifier import IntentProfile, IntentType
import ollama
import json
import logging

logger = logging.getLogger(__name__)

OLLAMA_MODEL = "mistral:7b-instruct"


def classify_intent_llm(intent_text: str) -> IntentProfile:
    """
    使用 gemma4:e4b 推斷意圖。

    Prompt 設計：
    - 輸出 JSON，結構與 IntentProfile 一致
    - 明確列出支援的 type，避免幻觉
    - 限制 response length，防止過度推理
    """
    system_prompt = """你是一個意圖分類專家。分析使用者輸入，輸出 JSON。

輸出格式（只能輸出 JSON，無其他文字）：
{
  "type": "CRUD | DASHBOARD | GAME | TOOL | API | UNKNOWN",
  "entities": ["實體1", "實體2"],
  "actions": ["新增", "刪除", "查詢"],
  "target": "html | react | flutter | swift",
  "theme": "modern | glass | soft | brutal"
}

規則：
- type 只能是 CRUD/DASHBOARD/GAME/TOOL/API/UNKNOWN 其中之一
- entities 從 [任務, 書籍, 庫存, 客戶, 報表, 遊戲, 計時, 計算, 密碼, 認證, 資料] 中選擇
- actions 從 [新增, 刪除, 編輯, 查詢, 列表, 排序, 匯出, 圖表, 統計, 完成, 審核, 通知] 中選擇
- theme 只能是 modern/glass/soft/brutal 其中之一
- 只輸出 JSON，不要任何其他文字"""

    try:
        response = ollama.generate(
            model=OLLAMA_MODEL,
            prompt=f"{system_prompt}\n\n使用者輸入：{intent_text}",
            options={
                "num_predict": 500,
                "temperature": 0.1,
            }
        )

        raw = response.response.strip()
        logger.info(f"LLM raw response: {repr(raw)}")

        # 解析 JSON（可能有 Markdown 包覆）
        if raw.startswith("```"):
            raw = raw.split("```")[1]
            if raw.startswith("json"):
                raw = raw[4:]
        raw = raw.strip()

        parsed = json.loads(raw)

        return IntentProfile(
            type=IntentType(parsed.get("type", "UNKNOWN")),
            entities=parsed.get("entities", []),
            actions=parsed.get("actions", []),
            context=intent_text,
            target=parsed.get("target", "html"),
            theme=parsed.get("theme", "modern"),
        )

    except json.JSONDecodeError as e:
        logger.warning(f"LLM JSON parse failed: {e}, raw: {raw[:100]}")
        return IntentProfile(
            type=IntentType.UNKNOWN,
            entities=[],
            actions=[],
            context=intent_text,
            target="html",
            theme="modern",
        )
    except Exception as e:
        logger.warning(f"Ollama call failed: {e}")
        return IntentProfile(
            type=IntentType.UNKNOWN,
            entities=[],
            actions=[],
            context=intent_text,
            target="html",
            theme="modern",
        )