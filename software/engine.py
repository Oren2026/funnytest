#!/usr/bin/env python3
"""
engine.py — Evolution Compiler 協調器 v2.0

流程:
    1. 解析 seed 檔案
    2. 驗證（包含 skill 存在性 + 衝突檢查）
    3. 遞迴合成（依賴圖展開 + 拓撲排序）
    4. 語意指紋漂移檢查
    5. 寫入產物 + 記錄日誌
"""

import sys
import os

from parser import parse_seed
from validator import validate_seed, load_injection_data
from ai_module import synthesize_code
from watch import check_evolution_drift
from logger import log_evolution


def run_evolution(seed_file_path: str):
    if not os.path.exists(seed_file_path):
        print(f"[-] Error: Seed file not found: {seed_file_path}")
        sys.exit(1)

    print(f"\n[Engine] Parsing: {seed_file_path}")
    parsed = parse_seed(seed_file_path)

    # 解析依賴圖（需要先知道 all_prohibits）
    print("[Engine] Resolving dependencies...")
    _, all_prohibits = synthesize_code(parsed, {"temp": 0, "mode": 0})

    print("[Engine] Validating constraints...")
    valid, conflicts = validate_seed(parsed, all_prohibits)
    if not valid:
        print("[Engine] 🛑 Validation failed. Abort.")
        sys.exit(1)

    # 實際合成
    injection_data = load_injection_data("config/app.json")
    print("[Engine] Synthesizing code...")
    evolved_code, _ = synthesize_code(parsed, injection_data)

    # 輸出模組名
    module_name = os.path.basename(seed_file_path).replace(".seed.c", "")
    output_c = f"{module_name}.c"

    # 語意漂移檢查
    drift_ratio, drift_msg = check_evolution_drift(evolved_code, snapshot_name=module_name)
    print(f"[Engine] {drift_msg}")

    if drift_ratio > 0.05:
        print("[Engine] 🛑 Drift > 5%. Abort.")
        sys.exit(1)

    # 寫入產物
    with open(output_c, "w", encoding="utf-8") as f:
        f.write(evolved_code)
    print(f"[Engine] ✅ Output: {output_c}")

    # 日誌
    log_evolution(
        module_name=output_c,
        drift_ratio=drift_ratio,
        logic_delta=f"Skills: {[u for u in parsed['meta']['uses']]}",
        healing_note="v2.0 — recursive dependency resolver"
    )


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python engine.py <seed_file>")
        sys.exit(1)
    run_evolution(sys.argv[1])
