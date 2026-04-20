import os
import re

"""
watch.py — 輕量漂移控制 (Semantic Drift Control)

用「關鍵語意指紋」比對新舊程式碼，而非純編輯距離。
指紋包含：
    - 函式簽名（function signatures）
    - Static 變數名稱列表
    - 重要常數值
"""

def extract_fingerprint(code: str) -> dict:
    """提取關鍵語意指紋。"""
    fingerprint = {
        "functions": sorted(re.findall(r'\b\w+\s+\w+\s*\([^)]*\)\s*\{', code)),
        "statics": sorted(re.findall(r'static\s+\w+\s+\w+\s*=', code)),
        "consts": sorted(re.findall(r'static\s+const\s+int\s+\w+\s*=\s*\d+', code)),
    }
    return fingerprint


def fingerprint_similarity(a: dict, b: dict) -> float:
    """計算兩個指紋的相似度（0.0 ~ 1.0）。"""
    score = 0.0
    total = 0

    for key in ["functions", "statics", "consts"]:
        a_set = set(a.get(key, []))
        b_set = set(b.get(key, []))
        if a_set or b_set:
            intersection = len(a_set & b_set)
            union = len(a_set | b_set)
            score += intersection / union if union else 1.0
            total += 1

    return score / total if total else 1.0


def check_evolution_drift(new_code: str, snapshot_name: str = "latest") -> (float, str):
    """
    檢查新程式碼相對於快照的語意漂移。
    如果漂移 > 5%，阻斷編譯。
    """
    snapshot_path = f"snapshots/{snapshot_name}.c"
    os.makedirs("snapshots", exist_ok=True)

    if not os.path.exists(snapshot_path):
        with open(snapshot_path, "w") as f:
            f.write(new_code)
        return 0.0, "Initial snapshot created."

    with open(snapshot_path, "r") as f:
        old_code = f.read()

    old_fp = extract_fingerprint(old_code)
    new_fp = extract_fingerprint(new_code)

    similarity = fingerprint_similarity(old_fp, new_fp)
    drift_ratio = 1.0 - similarity

    if drift_ratio > 0.05:
        msg = f"⚠️ ALERT: Semantic drift detected ({drift_ratio:.2%})"
        return drift_ratio, msg
    else:
        # 在安全範圍內，更新快照
        with open(snapshot_path, "w") as f:
            f.write(new_code)
        return drift_ratio, f"✅ Stable evolution. Drift: {drift_ratio:.2%}"


if __name__ == "__main__":
    test = "static const int TEMP = 80;\nint main() { return 0; }"
    print(extract_fingerprint(test))
    print(check_evolution_drift(test))
