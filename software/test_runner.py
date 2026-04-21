#!/usr/bin/env python3
"""
L1 測試流程腳本
流程: context.md → Intent Classifier → Schema Inferrer → Skill Router → Composer → QA Checker

用法:
    python test_runner.py                    # 跑預設 L1 測試（todo-context.md）
    python test_runner.py --case <name>      # 跑指定測試案例
    python test_runner.py --list             # 列出所有測試案例
"""
import os
import sys
import json
import time
from pathlib import Path

# Add software/ to path for node imports
SOFTWARE_DIR = Path(__file__).parent
sys.path.insert(0, str(SOFTWARE_DIR))

from nodes import (
    classify_intent, infer_schema, route_skills,
    resolve_dependencies, compose_output, qa_check
)


TEST_CASES_DIR = SOFTWARE_DIR / "test_cases"
SKILLS_DIR = SOFTWARE_DIR / "skills"
DEMO_DIR = SOFTWARE_DIR / "demo"


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    BLUE = "\033[94m"
    BOLD = "\033[1m"
    RESET = "\033[0m"


def green(msg): return f"{Colors.GREEN}{msg}{Colors.RESET}"
def red(msg): return f"{Colors.RED}{msg}{Colors.RESET}"
def yellow(msg): return f"{Colors.YELLOW}{msg}{Colors.RESET}"
def blue(msg): return f"{Colors.BLUE}{msg}{Colors.RESET}"
def bold(msg): return f"{Colors.BOLD}{msg}{Colors.RESET}"


def parse_context_file(path: Path) -> dict:
    """解析 context.md，提取需求描述"""
    content = path.read_text(encoding="utf-8")

    result = {
        "path": str(path),
        "name": path.stem,
        "intent_text": "",
        "expected_skills": [],
        "expected_theme": "modern",
        "difficulty": "L1",
    }

    for line in content.splitlines():
        line = line.strip()
        if line.startswith("## 需求描述") or line.startswith("# Context:"):
            # Capture the description section
            continue
        if line.startswith("##"):
            break
        if any(line.startswith(k) for k in ["我要", "做一個", "幫我做", "幫我做"]):
            result["intent_text"] = line
        if "預期產出" in line or "主題" in line:
            result["expected_theme"] = "glass" if "glass" in line.lower() else result["expected_theme"]

    if not result["intent_text"]:
        # Fallback: take first non-header line
        for line in content.splitlines():
            line = line.strip()
            if line and not line.startswith("#") and not line.startswith("-"):
                result["intent_text"] = line
                break

    return result


def run_pipeline(intent_text: str, target: str = "html", theme: str = "modern") -> dict:
    """執行完整的多節點流水線"""
    stages = {}

    # Stage 1: Intent Classification
    t0 = time.time()
    profile = classify_intent(intent_text)
    stages["intent_classifier"] = time.time() - t0

    # Override target/theme if specified
    profile.target = target
    profile.theme = theme

    # Stage 2: Schema Inference
    t0 = time.time()
    schema = infer_schema(profile)
    stages["schema_inferrer"] = time.time() - t0

    # Stage 3: Skill Routing
    t0 = time.time()
    skill_chain = route_skills(profile, schema)
    stages["skill_router"] = time.time() - t0

    # Stage 4: Dependency Resolution
    # Extract skill names from skill_chain (List[Dict]) before passing
    skill_names = [s["skill"] for s in skill_chain]
    t0 = time.time()
    ordered_skills = resolve_dependencies(skill_names, SKILLS_DIR)
    stages["dependency_resolver"] = time.time() - t0

    # Stage 5: Compose Output
    t0 = time.time()
    compiled = compose_output(ordered_skills, schema, profile)
    stages["composer"] = time.time() - t0

    # Stage 6: QA Check
    t0 = time.time()
    qa_result = qa_check(compiled, profile, schema)
    stages["qa_checker"] = time.time() - t0

    return {
        "profile": profile,
        "schema": schema,
        "skill_chain": skill_chain,
        "ordered_skills": ordered_skills,
        "compiled": compiled,
        "qa": qa_result,
        "stages": stages,
    }


def print_result(result: dict, verbose: bool = False):
    """格式化輸出測試結果"""
    profile = result["profile"]
    qa = result["qa"]
    stages = result["stages"]
    compiled = result["compiled"]

    # Header
    print()
    print(bold("=" * 60))
    print(f"{bold('🔬 L1 Test Result')}")
    print(bold("=" * 60))

    # Intent Profile
    print(f"\n{blue('📌 Intent Profile')}")
    print(f"  Type: {profile.type.value}")
    print(f"  Entities: {', '.join(profile.entities) or 'none'}")
    print(f"  Actions: {', '.join(profile.actions) or 'none'}")
    print(f"  Theme: {profile.theme}")
    print(f"  Target: {profile.target}")

    # Schema
    schema = result["schema"]
    print(f"\n{blue('📊 Inferred Schema')} ({len(schema)} fields)")
    for field in schema[:6]:
        print(f"  - {field.get('name','?')}: {field.get('type','text')}")
    if len(schema) > 6:
        print(f"  ... and {len(schema) - 6} more")

    # Skills
    skills_used = [s["skill"] for s in result["ordered_skills"]]
    print(f"\n{blue('🛠️  Skills Used')} ({len(skills_used)})")
    for s in skills_used:
        print(f"  ✓ {s}")

    # Stage timings
    print(f"\n{blue('⏱️  Stage Timings')}")
    total = sum(stages.values())
    for stage, dur in stages.items():
        bar = "█" * int(dur / max(total, 0.001) * 20)
        print(f"  {stage:<22} {dur*1000:>6.1f}ms {bar}")

    # QA Result
    print(f"\n{blue('🔍 QA Check')}")
    issues = qa.get("issues", [])
    passed = qa.get("passed", False)
    errors = [i for i in issues if i.level == "error"]
    warnings = [i for i in issues if i.level == "warning"]
    infos = [i for i in issues if i.level == "info"]

    if passed:
        print(f"  {green('✅ PASSED')} — {len(issues)} issue(s)")
    else:
        print(f"  {red('❌ FAILED')} — {len(errors)} error(s), {len(warnings)} warning(s)")

    if errors:
        print(f"\n  {red('Errors:')}")
        for i in errors:
            loc = f" [{i.location}]" if i.location else ""
            print(f"    ✗ {i.message}{loc}")

    if warnings:
        print(f"\n  {yellow('Warnings:')}")
        for i in warnings[:5]:
            loc = f" [{i.location}]" if i.location else ""
            print(f"    ⚠ {i.message}{loc}")

    if infos:
        for i in infos[:3]:
            print(f"    ℹ {i.message}")

    # Output size
    code = compiled.get("code", "")
    print(f"\n{blue('📦 Output')}")
    print(f"  Size: {len(code):,} bytes")
    print(f"  Skills: {len(skills_used)}")

    # Metadata warnings
    meta_warnings = compiled.get("warnings", [])
    if meta_warnings:
        print(f"\n{yellow('⚠️  Compiler Warnings:')}")
        for w in meta_warnings:
            print(f"    {w}")

    return passed, len(errors), len(warnings)


def run_test_case(case_name: str, verbose: bool = False, save_output: bool = True) -> dict:
    """執行單一測試案例"""
    case_path = TEST_CASES_DIR / case_name
    context_path = SOFTWARE_DIR / f"{case_name}-context.md"

    if not context_path.exists():
        # Try test_cases dir
        context_path = case_path

    if not context_path.exists():
        return {"name": case_name, "status": "SKIP", "reason": f"Context file not found"}

    ctx = parse_context_file(context_path)
    intent_text = ctx["intent_text"]
    theme = ctx.get("expected_theme", "modern")

    print(f"\n{'='*60}")
    print(f"{bold('▶ Running: ' + ctx['name'])}")
    print(f"{'='*60}")
    print(f"Intent: {intent_text[:80]}{'...' if len(intent_text) > 80 else ''}")

    try:
        result = run_pipeline(intent_text, target="html", theme=theme)
        passed, errors, warnings = print_result(result, verbose=verbose)

        if save_output:
            code = result["compiled"].get("code", "")
            output_path = DEMO_DIR / f"{ctx['name']}.html"
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(code, encoding="utf-8")

        return {
            "name": ctx["name"],
            "status": "PASS" if passed else "FAIL",
            "errors": errors,
            "warnings": warnings,
            "output_size": len(result["compiled"].get("code", "")),
        }

    except Exception as e:
        print(f"\n{red(f'✗ EXCEPTION: {e}')}")
        import traceback
        traceback.print_exc()
        return {"name": ctx["name"], "status": "ERROR", "reason": str(e)}


def list_test_cases():
    """列出所有可用測試案例"""
    print(bold("\n📂 Available Test Cases:\n"))
    for p in sorted(TEST_CASES_DIR.glob("*-context.md")):
        name = p.stem.replace("-context", "")
        ctx = parse_context_file(p)
        print(f"  {name:<30} — {ctx['intent_text'][:50]}...")
    print()


if __name__ == "__main__":
    args = sys.argv[1:]

    if "--list" in args:
        list_test_cases()
        sys.exit(0)

    # Default: run todo
    case = "todo"
    for i, arg in enumerate(args):
        if arg == "--case" and i + 1 < len(args):
            case = args[i + 1]

    verbose = "--verbose" in args or "-v" in args

    result = run_test_case(case, verbose=verbose)

    # Exit code
    if result["status"] == "PASS":
        sys.exit(0)
    elif result["status"] == "FAIL":
        sys.exit(1)
    else:
        sys.exit(2)
