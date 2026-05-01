#!/usr/bin/env python3
"""
Mini Evolution Compiler — Standalone Test Runner
================================================
Extract from main Evolution Compiler for minimal framework demo.

用法:
    python test_runner.py                    # 預設 todo-context
    python test_runner.py --case <name>     # 跑指定測試案例
    python test_runner.py --list            # 列出所有案例
    python test_runner.py --gen <intent>    # 直接用自然語法生成

流程: context.md → Intent → Schema → Skill Router → Composer → QA
"""
import os, re, sys, json, time
from pathlib import Path

# Setup paths
SCRIPT_DIR = Path(__file__).parent.resolve()
MINI_DIR = SCRIPT_DIR
CONTEXTS_DIR = MINI_DIR / "contexts"
SKILLS_DIR = MINI_DIR / "skills"
OUTPUT_DIR = MINI_DIR / "output"

# Add nodes/ to path
sys.path.insert(0, str(MINI_DIR))

from nodes import (
    classify_intent, infer_schema, route_skills,
    resolve_dependencies, compose_output, qa_check
)


def parse_context_file(path: Path) -> dict:
    """解析 context.md，提取需求描述"""
    content = path.read_text(encoding="utf-8")
    result = {
        "name": path.stem,
        "intent_text": "",
        "theme": "modern",
    }
    for line in content.splitlines():
        s = line.strip()
        if s.startswith("## 需求描述") or s.startswith("# Context:"):
            continue
        if s.startswith("##"):
            break
        if s and not s.startswith("#") and not s.startswith("-"):
            result["intent_text"] = s
            break
    for line in content.splitlines():
        if "glass" in line.lower():
            result["theme"] = "glass"
    return result


def run_pipeline(intent_text: str, target: str = "html", theme: str = "modern") -> dict:
    """執行完整流水線"""
    stages = {}

    t0 = time.time()
    profile = classify_intent(intent_text)
    stages["intent"] = time.time() - t0

    profile.target = target
    profile.theme = theme

    t0 = time.time()
    schema = infer_schema(profile)
    stages["schema"] = time.time() - t0

    t0 = time.time()
    skill_chain = route_skills(profile, schema)
    stages["router"] = time.time() - t0

    skill_names = [s["skill"] for s in skill_chain]
    t0 = time.time()
    ordered_skills = resolve_dependencies(skill_names, SKILLS_DIR)
    stages["deps"] = time.time() - t0

    t0 = time.time()
    compiled = compose_output(ordered_skills, schema, profile, "html")
    stages["compose"] = time.time() - t0

    t0 = time.time()
    qa_result = qa_check(compiled, profile, schema)
    stages["qa"] = time.time() - t0

    return {
        "profile": profile,
        "schema": schema,
        "ordered_skills": ordered_skills,
        "compiled": compiled,
        "qa": qa_result,
        "stages": stages,
    }


def print_result(result: dict):
    """格式化輸出"""
    p = result["profile"]
    qa = result["qa"]
    stages = result["stages"]
    compiled = result["compiled"]

    print()
    print("=" * 56)
    print("🔬 Mini Framework — L1 Test Result")
    print("=" * 56)

    print(f"\n📌 Intent Profile")
    print(f"  Type:     {p.type.value}")
    print(f"  Entities: {', '.join(p.entities) or 'none'}")
    print(f"  Actions:  {', '.join(p.actions) or 'none'}")
    print(f"  Theme:    {p.theme}")
    print(f"  Target:   {p.target}")

    schema = result["schema"]
    print(f"\n📊 Schema ({len(schema)} fields)")
    for f in schema[:6]:
        print(f"  - {f.get('name','?')}: {f.get('type','text')}")
    if len(schema) > 6:
        print(f"  ... +{len(schema)-6} more")

    skills = [s["skill"] for s in result["ordered_skills"]]
    print(f"\n🛠️  Skills ({len(skills)})")
    for s in skills:
        print(f"  ✓ {s}")

    total = sum(stages.values()) * 1000
    print(f"\n⏱️  Timings")
    for name, dur in stages.items():
        bar = "█" * max(1, int(dur / max(sum(stages.values()), 0.001) * 20))
        print(f"  {name:<12} {dur*1000:>6.1f}ms {bar}")

    issues = qa.get("issues", [])
    passed = qa.get("passed", False)
    errors = [i for i in issues if i.level == "error"]
    warnings = [i for i in issues if i.level == "warning"]

    print(f"\n🔍 QA Check")
    if passed:
        print(f"  ✅ PASSED — {len(issues)} issue(s)")
    else:
        print(f"  ❌ FAILED — {len(errors)} error(s), {len(warnings)} warning(s)")

    code = compiled.get("code", "")
    meta_warnings = compiled.get("warnings", [])
    if meta_warnings:
        print(f"\n⚠️  Compiler Warnings:")
        for w in meta_warnings:
            print(f"    {w}")
    else:
        print(f"\n  (no compiler warnings)")

    print(f"\n📦 Output")
    print(f"  Size: {len(code):,} bytes")

    return passed, len(errors), len(warnings)


def gen_from_text(intent_text: str, theme: str = "modern") -> Path:
    """直接從自然語法文字生成 HTML"""
    print(f"\n{'='*56}")
    print(f"▶ Generating from intent text")
    print(f"{'='*56}")
    print(f"Intent: {intent_text[:80]}...")

    result = run_pipeline(intent_text, target="html", theme=theme)
    passed, errors, warnings = print_result(result)

    code = result["compiled"].get("code", "")
    output_name = f"gen_{int(time.time())}.html"
    output_path = OUTPUT_DIR / output_name
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    output_path.write_text(code, encoding="utf-8")

    status = "✅" if passed else "❌"
    print(f"\n  {status} Saved → {output_path.relative_to(MINI_DIR)}")
    return output_path


def run_case(case_name: str, save: bool = True) -> dict:
    """執行單一測試案例"""
    ctx_path = CONTEXTS_DIR / f"{case_name}.md"
    if not ctx_path.exists():
        return {"name": case_name, "status": "SKIP", "reason": f"Context not found: {ctx_path}"}

    ctx = parse_context_file(ctx_path)
    print(f"\n{'='*56}")
    print(f"▶ Running: {case_name}")
    print(f"{'='*56}")
    print(f"Intent: {ctx['intent_text'][:80]}...")

    try:
        result = run_pipeline(
            ctx["intent_text"],
            target="html",
            theme=ctx.get("theme", "modern")
        )
        passed, errors, warnings = print_result(result)

        if save:
            code = result["compiled"].get("code", "")
            output_path = OUTPUT_DIR / f"{case_name}.html"
            OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
            output_path.write_text(code, encoding="utf-8")
            print(f"\n  📦 Saved → {output_path.relative_to(MINI_DIR)}")

        return {
            "name": case_name,
            "status": "PASS" if passed else "FAIL",
            "errors": errors,
            "warnings": warnings,
        }
    except Exception as e:
        import traceback
        print(f"\n  ❌ EXCEPTION: {e}")
        traceback.print_exc()
        return {"name": case_name, "status": "ERROR", "reason": str(e)}


def list_cases():
    print("\n📂 Available contexts:\n")
    for p in sorted(CONTEXTS_DIR.glob("*.md")):
        ctx = parse_context_file(p)
        print(f"  {ctx['name']:<30} — {ctx['intent_text'][:55]}...")
    print()


if __name__ == "__main__":
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    args = sys.argv[1:]

    if "--list" in args:
        list_cases()

    elif "--gen" in args:
        idx = args.index("--gen")
        intent_text = " ".join(args[idx+1:])
        gen_from_text(intent_text)

    elif "--case" in args:
        idx = args.index("--case")
        case = args[idx+1] if idx+1 < len(args) else "todo"
        run_case(case)

    elif len(args) == 0:
        # Default: run todo-context
        run_case("todo-context")

    else:
        # Treat as intent text
        intent_text = " ".join(args)
        gen_from_text(intent_text)
