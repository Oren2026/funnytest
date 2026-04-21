import os
import re
import glob
from collections import OrderedDict

"""
ai_module.py — 遞迴依賴解析引擎 (Recursive Dependency Resolver)

核心流程：
    1. 解析每個 .skill 的 header（# depends:, # prohibit:）
    2. 遞迴展開依賴圖（DFS + 環檢測）
    3. 拓撲排序 → 依序組合程式碼
    4. 收集所有 prohibit 交給 validator
"""

SKILLS_DIR = "skills"


class Skill:
    def __init__(self, name: str, code: str, depends: list, prohibit: list):
        self.name = name
        self.code = code
        self.depends = depends
        self.prohibit = prohibit


def parse_skill_header(skill_path: str) -> tuple:
    """解析 .skill 檔案的 header，回傳 (depends, prohibit)。"""
    depends = []
    prohibit = []

    with open(skill_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    for line in lines:
        line = line.strip()
        if line.startswith("# depends:"):
            deps = line.split(":", 1)[1].strip()
            depends = [d.strip() for d in deps.split(",") if d.strip()]
        elif line.startswith("# prohibit:"):
            pros = line.split(":", 1)[1].strip()
            prohibit = [p.strip() for p in pros.split(",") if p.strip()]

    return depends, prohibit


def load_skill(skill_name: str) -> Skill:
    """從知識庫載入 skill，包含 header 解析。"""
    search_pattern = os.path.join(SKILLS_DIR, "**", f"{skill_name}.skill")
    matches = glob.glob(search_pattern, recursive=True)
    if not matches:
        print(f"[Crystallizer] ⚠️ Skill not found: {skill_name}")
        return None

    skill_path = matches[0]
    with open(skill_path, "r", encoding="utf-8") as f:
        code = f.read()

    depends, prohibit = parse_skill_header(skill_path)
    print(f"[Crystallizer] Loaded: {skill_name} (depends: {depends})")
    return Skill(skill_name, code, depends, prohibit)


def resolve_dependencies(skill_name: str, visited: set = None, resolution_order: list = None) -> list:
    """
    遞迴解析依賴圖，DFS 展開。
    回傳拓撲排序後的 skill 清單（由底層到上層）。
    """
    if visited is None:
        visited = set()
    if resolution_order is None:
        resolution_order = []

    if skill_name in visited:
        return  # 已處理過，跳過

    skill = load_skill(skill_name)
    if skill is None:
        return

    visited.add(skill_name)

    # 先遞迴處理所有依賴
    for dep in skill.depends:
        if dep == "none" or not dep:
            continue
        resolve_dependencies(dep, visited, resolution_order)

    # 依賴處理完後才加入清單（這樣清單是由底層到上層）
    resolution_order.append(skill_name)


def resolve_all_dependencies(top_level_skills: list) -> tuple:
    """
    解析所有頂層技能的完整依賴圖。
    回傳: (ordered_skills, all_prohibits)
    """
    visited = set()
    ordered = []
    all_prohibits = []

    for skill_name in top_level_skills:
        resolve_dependencies(skill_name, visited, ordered)

    # 依據 ordered 清單取得完整的 Skill 物件
    skills_map = {}
    for name in ordered:
        s = load_skill(name)
        if s:
            skills_map[name] = s
            all_prohibits.extend(s.prohibit)

    # 按拓撲順序取出
    ordered_skills = [skills_map[name] for name in ordered if name in skills_map]
    return ordered_skills, all_prohibits


def synthesize_code(parsed: dict, injection_data: dict = None) -> tuple:
    """
    根據 parsed seed 生成 C 程式碼。
    回傳: (code, all_prohibits)
    """
    if injection_data is None:
        injection_data = {}

    top_level_skills = parsed["meta"]["uses"]
    invocations = parsed["body"]["invocations"]

    # 解析依賴並拓撲排序
    ordered_skills, all_prohibits = resolve_all_dependencies(top_level_skills)

    # 先處理 invocations 確定需要的元件
    has_daemon = any(inv["component"] == "daemon-loop" for inv in invocations)
    has_sort = any(inv["component"].startswith("sort-") for inv in invocations)
    has_queue = any(inv["component"] == "queue-demo" for inv in invocations)

    output = ""

    # 1. 通用 includes
    output += "#include <stdio.h>\n"
    output += "#include <stdlib.h>\n"
    output += "#include <signal.h>\n"
    output += "#include <unistd.h>\n"
    if any(s.name == "timer-periodic" for s in ordered_skills):
        output += "#include <sys/time.h>\n"
    output += "\n"

    # 2. Static const 注入
    for key, val in injection_data.items():
        output += f"static const int {key.upper()} = {val};\n"
    if injection_data:
        output += "\n"

    # 2b. Daemon tick function（需要在 skill code 之前，因為 daemon-loop 依賴它）
    if has_daemon:
        output += "void tick_default(void) {\n"
        output += '    printf("[Kernel] tick\\r");\n'
        output += "    fflush(stdout);\n"
        output += "}\n\n"

    # 3. 按拓撲順序注入 skill code
    for skill in ordered_skills:
        code = strip_header(skill.code)
        if code.strip():
            output += code + "\n"

    # 4. 處理 body invocations → 組合 main

    output += "int main() {\n"

    if has_daemon:
        output += "    signal_setup();\n"
        output += '    printf("[Kernel] Started. TEMP=%d\\n", TEMP);\n'
        output += "    daemon_loop_body(tick_default);\n"
        output += '    printf("\\n[Kernel] Shutdown complete.\\n");\n'

    elif has_sort:
        target_arr = "arr"
        for inv in invocations:
            if inv["component"].startswith("sort-"):
                target_arr = inv["attrs"].get("name", "arr")
                break
        output += f'    int {target_arr}[] = {{5, 2, 8, 1, 9, 3, 7, 4, 6, 0}};\n'
        output += f'    int n = sizeof({target_arr}) / sizeof({target_arr}[0]);\n'
        output += f'    printf("Before: "); print_array({target_arr}, n); printf("\\n");\n'

        sort_fn, sort_args = get_sort_fn(ordered_skills)
        output += f'    {sort_fn}({sort_args});\n'
        output += f'    printf("After: "); print_array({target_arr}, n); printf("\\n");\n'

    elif has_queue:
        output += "    struct StaticQueue q;\n"
        output += "    queue_init(&q);\n"
        output += "    queue_enqueue(&q, 10);\n"
        output += "    queue_enqueue(&q, 20);\n"
        output += "    queue_enqueue(&q, 30);\n"
        output += '    printf("Dequeue: %d\\n", queue_dequeue(&q));\n'
        output += '    printf("Dequeue: %d\\n", queue_dequeue(&q));\n'

    else:
        output += '    printf("Evolution complete.\\n");\n'

    output += "    return 0;\n"
    output += "}\n"

    return output, all_prohibits


def strip_header(code: str) -> str:
    """移除 skill header 的 comment 行，保留純 code。"""
    lines = code.split("\n")
    result = []
    for line in lines:
        if line.strip().startswith("# skill:") or \
           line.strip().startswith("# depends:") or \
           line.strip().startswith("# prohibit:"):
            continue
        result.append(line)
    return "\n".join(result)


def get_sort_fn(ordered_skills: list) -> tuple:
    """從已解析的 skills 中，找出可用的 sort 函式名稱和參數形式。"""
    for skill in reversed(ordered_skills):
        if skill.name == "quick_sort":
            return "quick_sort", "arr, 0, n-1"
        elif skill.name == "insertion_sort":
            return "insertion_sort", "arr, n"
        elif skill.name == "bubble_sort":
            return "bubble_sort", "arr, n"
    return "bubble_sort", "arr, n"


if __name__ == "__main__":
    from parser import parse_seed
    import sys
    if len(sys.argv) > 1:
        parsed = parse_seed(sys.argv[1])
        code, prohibits = synthesize_code(parsed, {"temp": 80, "mode": 1})
        print("=== Generated C code ===")
        print(code)
        print(f"\n=== Prohibits: {prohibits} ===")
