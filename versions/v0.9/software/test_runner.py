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
# demo/ lives at repo root (alongside software/), not inside software/
DEMO_DIR = SOFTWARE_DIR.parent / "demo"


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
    compiled = compose_output(ordered_skills, schema, profile, "html")
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
            rebuild_demo_index(DEMO_DIR, SOFTWARE_DIR)

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


def rebuild_demo_index(demo_dir: Path, software_dir: Path):
    """Scan demo/ and regenerate index.html with all HTML files.
    Reads context.md files to extract descriptions for L1 test cases."""
    index_path = demo_dir / "index.html"
    files = sorted(demo_dir.glob("*.html"))

    # L1 test case descriptions from context.md
    context_descs = {}
    for ctx_file in software_dir.glob("*-context.md"):
        name = ctx_file.stem.replace("-context", "")
        try:
            content = ctx_file.read_text(encoding="utf-8")
            # Extract first line after ## 需求描述 or the raw intent line
            m = re.search(r"## 需求描述\s*\n(.+?)(?=\n##|\n#)", content, re.DOTALL)
            if m:
                lines = [l.strip() for l in m.group(1).splitlines() if l.strip() and not l.strip().startswith("-")]
                context_descs[name] = lines[0][:60] if lines else name
            else:
                context_descs[name] = name
        except:
            context_descs[name] = name

    # Extract title from HTML content
    def get_title(path):
        try:
            content = path.read_text(encoding="utf-8")
            m = re.search(r'<title>(.*?)</title>', content)
            return m.group(1) if m else path.stem
        except:
            return path.stem

    # Group: context-based (L1 tests) vs legacy
    l1_files = [f for f in files if "-context" in f.name]
    legacy_files = [f for f in files if "-context" not in f.name and f.name != "index.html"]

    cards_html = ""

    # L1 tests section
    if l1_files:
        cards_html += '  <p class="section-title">L1 測試案例</p>\n  <div class="grid">\n'
        for f in l1_files:
            title = get_title(f)
            # Derive case name from filename: books-context.html → books
            case_name = f.stem.replace("-context", "")
            desc = context_descs.get(case_name, title)
            kb = f.stat().st_size // 1024
            cards_html += f'''    <a class="card" href="{f.name}">
      <div class="card-title">{title}</div>
      <div class="card-desc">{desc}</div>
      <div class="card-meta">
        <span class="tag new">L1</span>
        <span class="tag">{kb}KB</span>
      </div>
    </a>\n'''
        cards_html += "  </div>\n\n  <hr>\n\n  "

    # Legacy section
    if legacy_files:
        cards_html += '  <p class="section-title">早期產出</p>\n  <div class="grid">\n'
        for f in legacy_files:
            title = get_title(f)
            kb = f.stat().st_size // 1024
            cards_html += f'''    <a class="card" href="{f.name}">
      <div class="card-title">{title}</div>
      <div class="card-desc">{f.name}</div>
      <div class="card-meta">
        <span class="tag">{kb}KB</span>
      </div>
    </a>\n'''
        cards_html += "  </div>\n"

    html = HTML_TEMPLATE.replace("<!-- %%CARDS%% -->", cards_html)
    index_path.write_text(html, encoding="utf-8")
    total = len(l1_files) + len(legacy_files)
    print(f"\n  {green('📝 Updated index.html')} — {total} page(s) indexed")


HTML_TEMPLATE = """<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Evolution Compiler — Demo Index</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
      background: #0f1117;
      color: #e2e8f0;
      min-height: 100vh;
      padding: 40px;
    }
    h1 { font-size: 28px; font-weight: 700; color: #fff; margin-bottom: 8px; }
    .subtitle { color: #64748b; margin-bottom: 40px; font-size: 14px; }
    .section-title {
      font-size: 11px; font-weight: 600; text-transform: uppercase;
      letter-spacing: 0.1em; color: #6366f1; margin-bottom: 16px;
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
      gap: 16px;
      margin-bottom: 48px;
    }
    .card {
      background: #1e293b;
      border: 1px solid #334155;
      border-radius: 10px;
      padding: 18px 20px;
      text-decoration: none;
      color: inherit;
      transition: all 0.15s;
      display: block;
    }
    .card:hover {
      border-color: #6366f1;
      background: #263345;
      transform: translateY(-2px);
    }
    .card-title { font-size: 15px; font-weight: 600; color: #fff; margin-bottom: 4px; }
    .card-desc { font-size: 13px; color: #94a3b8; margin-bottom: 10px; }
    .card-meta { display: flex; gap: 6px; flex-wrap: wrap; }
    .tag {
      font-size: 11px; padding: 2px 7px;
      border-radius: 4px; background: #334155; color: #94a3b8;
    }
    .tag.new { background: rgba(99,102,241,0.25); color: #818cf8; }
    hr { border: none; border-top: 1px solid #1e293b; margin: 40px 0; }
  </style>
</head>
<body>
  <h1>Evolution Compiler — Demo Index</h1>
  <p class="subtitle">軟體是 AI 用的工具，輸出要有品質，内部不需要 UI</p>

<!-- %%CARDS%% -->
</body>
</html>
"""


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
