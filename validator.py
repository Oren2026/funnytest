import os
import json

"""
validator.py — 形式化驗證器 v2

職責：
    1. Skill 存在性
    2. memory=static 時，skill 自己是否用了 malloc（看 code，不看 header）
    3. Constraint prohibit 清單是否覆蓋 skill 宣告的 prohibits
    4. Body invocations 語法
    5. Max-temp 邊界
"""


def validate_seed(parsed: dict, all_prohibits: list = None) -> (bool, list):
    if all_prohibits is None:
        all_prohibits = []

    conflicts = []
    constraints = parsed.get("constraints", {})
    uses = parsed["meta"]["uses"]
    invocations = parsed["body"]["invocations"]

    # ---------- 1. Skill 存在性 ----------
    available = get_available_skills()
    for skill_name in uses:
        if skill_name not in available:
            conflicts.append(f"Skill not found: '{skill_name}'")

    # ---------- 2. Memory policy = static 時，skill 不能自己 call malloc ----------
    if constraints.get("memory") == "static":
        for skill_name in uses:
            code = load_skill_code(skill_name)
            if code and uses_malloc(code):
                conflicts.append(
                    f"Conflict: memory=static but skill '{skill_name}' "
                    f"calls malloc/calloc/realloc/free in its implementation."
                )

    # ---------- 3. Constraint prohibit 覆蓋檢查 ----------
    # constraint 宣告的禁止清單，必須涵蓋 skill 自己說它避免的東西
    constraint_prohibits = set(constraints.get("prohibit", {}).get("names", []))
    for skill_name in uses:
        skill_prohibits = set(get_skill_header_prohibits(skill_name))
        if skill_prohibits and not skill_prohibits.issubset(constraint_prohibits):
            missing = skill_prohibits - constraint_prohibits
            conflicts.append(
                f"Conflict: constraint prohibits {list(constraint_prohibits)} "
                f"but skill '{skill_name}' also avoids {list(missing)} — "
                f"constraint must cover all."
            )

    # ---------- 4. Body invocations ----------
    known = {
        "sort-array", "daemon-loop", "queue-demo",
        "bubble-sort", "insertion-sort", "quick-sort",
        "search-linear", "search-binary",
        "ll-demo", "stack-demo", "queue-demo"
    }
    for inv in invocations:
        if inv["component"] not in known:
            print(f"[Validator] ⚠️ Unknown component: '{inv['component']}'")

    # ---------- 5. Max-temp ----------
    injection = load_injection_data("config/app.json")
    max_temp = constraints.get("max_temp")
    if max_temp and injection.get("temp", 0) > max_temp:
        conflicts.append(f"Security: temp={injection['temp']} > max-temp={max_temp}")

    # ---------- 結果 ----------
    if conflicts:
        print("\n[Validator] 🛑 Conflicts:")
        for c in conflicts:
            print(f"  - {c}")
        return False, conflicts

    print("[Validator] ✅ Passed.")
    print(f"  - Uses: {uses}")
    return True, []


def get_available_skills() -> set:
    skill_dirs = ["skills/core", "skills/algorithms", "skills/structures", "skills/system", "skills/c_lang"]
    available = set()
    for d in skill_dirs:
        if os.path.exists(d):
            for f in os.listdir(d):
                if f.endswith(".skill"):
                    available.add(f.replace(".skill", ""))
    return available


def load_skill_code(skill_name: str) -> str:
    """載入 skill 原始碼（不含 header）。"""
    import glob
    matches = glob.glob(f"skills/**/{skill_name}.skill", recursive=True)
    if not matches:
        return ""
    with open(matches[0], "r", encoding="utf-8") as f:
        content = f.read()
    # 移除 header lines
    lines = []
    for line in content.split("\n"):
        if line.strip().startswith("#"):
            continue
        lines.append(line)
    return "\n".join(lines)


def uses_malloc(code: str) -> bool:
    """檢查程式碼是否直接呼叫了 malloc/calloc/realloc/free。"""
    import re
    # 去除字串常數中的干擾
    code_no_strings = re.sub(r'"[^"]*"', '', code)
    return any(f in code_no_strings for f in ["malloc", "calloc", "realloc", "free"])


def get_skill_header_prohibits(skill_name: str) -> list:
    """從 header 讀取 prohibits（不含 none）。"""
    import glob
    matches = glob.glob(f"skills/**/{skill_name}.skill", recursive=True)
    if not matches:
        return []
    prohibits = []
    with open(matches[0], "r", encoding="utf-8") as f:
        for line in f:
            if line.strip().startswith("# prohibit:"):
                parts = line.split(":", 1)[1].strip()
                prohibits = [p.strip() for p in parts.split(",") if p.strip() and p.strip() != "none"]
    return prohibits


def load_injection_data(source: str) -> dict:
    defaults = {"temp": 25, "mode": 0}
    if not os.path.exists(source):
        return defaults
    try:
        with open(source, "r") as f:
            defaults.update(json.load(f))
    except (json.JSONDecodeError, IOError):
        pass
    return defaults


if __name__ == "__main__":
    from parser import parse_seed
    import sys
    if len(sys.argv) > 1:
        parsed = parse_seed(sys.argv[1])
        ok, conflicts = validate_seed(parsed)
        print(f"Valid: {ok}")
