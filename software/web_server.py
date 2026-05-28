#!/usr/bin/env python3
"""Evolution Compiler — Web UI Server

提供自然語言输入 → HTML 输出的可视化界面。

路由：
  GET  /              — UI 主页面
  GET  /preview/<id>  — 预览生成的 HTML
  POST /api/generate  — 执行完整 pipeline
  POST /api/classify  — 仅分类意图（快速）
  GET  /api/status    — 服务状态
"""
import os
import sys
import json
import time
import uuid
from pathlib import Path
from flask import Flask, request, jsonify, render_template_string, send_file

# Ollama path fix
sys.path.insert(0, '/Users/oren/Library/Python/3.9/lib/python/site-packages')

SOFTWARE_DIR = Path(__file__).parent
sys.path.insert(0, str(SOFTWARE_DIR))

from nodes import (
    classify_intent, infer_schema, route_skills,
    resolve_dependencies, compose_output, qa_check
)
from nodes.llm_intent_classifier import classify_intent_llm

app = Flask(__name__)
app.config['MAX_CONTENT_LENGTH'] = 1 * 1024 * 1024

# Output storage
OUTPUT_DIR = SOFTWARE_DIR / "output" / "web"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# Cache for generated results
RESULTS = {}


def run_pipeline(intent_text: str, use_llm: bool = False) -> dict:
    """执行完整 pipeline，返回结构化结果。"""
    t0 = time.time()

    # Stage 1: Intent Classification
    if use_llm:
        profile = classify_intent_llm(intent_text)
    else:
        profile = classify_intent(intent_text)
    classify_ms = (time.time() - t0) * 1000

    # Stage 2: Schema Inference
    t1 = time.time()
    schema = infer_schema(profile)
    schema_ms = (time.time() - t1) * 1000

    # Stage 3: Skill Routing
    t2 = time.time()
    skill_chain = route_skills(profile, schema)
    skills_used = [s["skill"] for s in skill_chain]
    router_ms = (time.time() - t2) * 1000

    # Stage 4: Dependency Resolution
    t3 = time.time()
    ordered_skills = resolve_dependencies(skills_used, SOFTWARE_DIR / "skills")
    deps_ms = (time.time() - t3) * 1000

    # Stage 5: Compose Output
    t4 = time.time()
    compiled = compose_output(ordered_skills, schema, profile, "html")
    compose_ms = (time.time() - t4) * 1000

    # Stage 6: QA Check
    t5 = time.time()
    qa = qa_check(compiled, profile, schema)
    qa_ms = (time.time() - t5) * 1000

    total_ms = (time.time() - t0) * 1000

    return {
        "id": str(uuid.uuid4())[:8],
        "intent": intent_text,
        "use_llm": use_llm,
        "profile": {
            "type": profile.type.value,
            "entities": profile.entities,
            "actions": profile.actions,
            "theme": profile.theme,
            "target": profile.target,
        },
        "schema": schema,
        "skills": skills_used,
        "stages": {
            "classify": round(classify_ms, 1),
            "schema": round(schema_ms, 1),
            "router": round(router_ms, 1),
            "deps": round(deps_ms, 1),
            "compose": round(compose_ms, 1),
            "qa": round(qa_ms, 1),
            "total": round(total_ms, 1),
        },
        "qa": {
            "passed": qa.get("passed", False),
            "errors": [i.message for i in qa.get("issues", []) if i.level == "error"],
            "warnings": [i.message for i in qa.get("issues", []) if i.level == "warning"],
        },
        "code": compiled.get("code", ""),
        "size": len(compiled.get("code", "")),
    }


@app.route('/api/status', methods=['GET'])
def status():
    """服務狀態檢查"""
    return jsonify({
        "status": "ok",
        "llm_available": True,
        "model": "mistral:7b-instruct",
        "output_dir": str(OUTPUT_DIR),
    })


@app.route('/api/classify', methods=['POST'])
def api_classify():
    """快速意圖分類（keyword 版，無 LLM）"""
    data = request.json
    intent = data.get('intent', '')
    if not intent:
        return jsonify({"error": "intent required"}), 400

    profile = classify_intent(intent)
    return jsonify({
        "type": profile.type.value,
        "entities": profile.entities,
        "actions": profile.actions,
        "theme": profile.theme,
        "target": profile.target,
    })


@app.route('/api/generate', methods=['POST'])
def api_generate():
    """執行完整 pipeline，生成 HTML"""
    data = request.json
    intent = data.get('intent', '')
    use_llm = data.get('use_llm', False)

    if not intent:
        return jsonify({"error": "intent required"}), 400

    try:
        result = run_pipeline(intent, use_llm=use_llm)

        # Save output HTML
        output_path = OUTPUT_DIR / f"{result['id']}.html"
        output_path.write_text(result['code'], encoding='utf-8')
        result['output_url'] = f"/preview/{result['id']}"

        # Cache result
        RESULTS[result['id']] = result

        return jsonify(result)
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route('/preview/<result_id>', methods=['GET'])
def preview(result_id):
    """預覽生成的 HTML"""
    path = OUTPUT_DIR / f"{result_id}.html"
    if not path.exists():
        return f"Result {result_id} not found", 404
    return send_file(path, mimetype='text/html')


@app.route('/', methods=['GET'])
def index():
    """主頁面"""
    html = '''
<!DOCTYPE html>
<html lang="zh-Hant">
<head>
<meta charset="UTF-8">
<title>Evolution Compiler — Web UI</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, sans-serif; background: #0f0f1a; color: #e0e0e0; min-height: 100vh; }
.container { max-width: 1100px; margin: 0 auto; padding: 20px; }
header { text-align: center; padding: 30px 0 20px; }
header h1 { font-size: 1.8em; color: #7dd3fc; margin-bottom: 6px; }
header p { color: #888; font-size: 0.9em; }
.input-section { background: #1a1a2e; border-radius: 12px; padding: 20px; margin-bottom: 20px; }
.input-row { display: flex; gap: 10px; }
#intent-input { flex: 1; background: #252540; border: 1px solid #333; border-radius: 8px; padding: 14px 16px; color: #fff; font-size: 1em; outline: none; }
#intent-input:focus { border-color: #7dd3fc; }
.btn { background: #3b82f6; color: #fff; border: none; border-radius: 8px; padding: 12px 24px; cursor: pointer; font-size: 0.95em; transition: background 0.2s; }
.btn:hover { background: #2563eb; }
.btn:disabled { background: #555; cursor: not-allowed; }
.btn-secondary { background: #374151; }
.btn-secondary:hover { background: #4b5563; }
.checkbox-row { display: flex; align-items: center; gap: 8px; margin-top: 12px; color: #aaa; font-size: 0.88em; }
.checkbox-row input { width: 16px; height: 16px; }
.results { display: grid; grid-template-columns: 340px 1fr; gap: 20px; }
.panel { background: #1a1a2e; border-radius: 12px; overflow: hidden; }
.panel-header { background: #252540; padding: 12px 16px; font-weight: 600; font-size: 0.9em; color: #7dd3fc; border-bottom: 1px solid #333; display: flex; justify-content: space-between; align-items: center; }
.panel-body { padding: 16px; font-size: 0.88em; }
.info-grid { display: grid; grid-template-columns: auto 1fr; gap: 8px 16px; }
.info-key { color: #888; }
.info-val { color: #fff; }
.stage-bar { display: flex; align-items: center; gap: 8px; margin: 4px 0; font-size: 0.85em; }
.stage-name { width: 80px; color: #aaa; }
.bar-bg { flex: 1; height: 6px; background: #333; border-radius: 3px; overflow: hidden; }
.bar-fill { height: 100%; background: #3b82f6; border-radius: 3px; }
.stage-ms { color: #666; width: 50px; text-align: right; }
.skills-list { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
.skill-tag { background: #1e3a5f; color: #7dd3fc; padding: 3px 10px; border-radius: 20px; font-size: 0.82em; }
.error-tag { background: #5f1e1e; color: #f87171; }
.warning-tag { background: #5f4a1e; color: #fbbf24; }
.preview-iframe { width: 100%; height: 500px; border: none; border-radius: 0 0 12px 12px; background: #fff; }
.qa-badge { padding: 2px 8px; border-radius: 10px; font-size: 0.8em; }
.qa-pass { background: #1a4d1a; color: #4ade80; }
.qa-fail { background: #4d1a1a; color: #f87171; }
.loading { text-align: center; padding: 40px; color: #888; }
.hidden { display: none; }
</style>
</head>
<body>
<div class="container">
  <header>
    <h1>⚡ Evolution Compiler</h1>
    <p>自然語言 → HTML 網頁</p>
  </header>

  <div class="input-section">
    <div class="input-row">
      <input id="intent-input" type="text" placeholder="例如：我要一個代辦事項清單 with 毛玻璃效果" />
      <button class="btn" id="generate-btn" onclick="generate()">生成</button>
    </div>
    <div class="checkbox-row">
      <input type="checkbox" id="use-llm" />
      <label for="use-llm">使用 LLM 增強分類（mistral:7b-instruct，第一次較慢）</label>
    </div>
  </div>

  <div id="loading" class="loading hidden">生成中...</div>

  <div id="results" class="results hidden">
    <!-- Left: Profile + Stages -->
    <div>
      <div class="panel">
        <div class="panel-header">
          <span>📋 Intent Profile</span>
          <span id="llm-badge" class="qa-badge qa-pass hidden">LLM</span>
        </div>
        <div class="panel-body">
          <div class="info-grid">
            <span class="info-key">Type</span>
            <span class="info-val" id="profile-type">—</span>
            <span class="info-key">Entities</span>
            <span class="info-val" id="profile-entities">—</span>
            <span class="info-key">Actions</span>
            <span class="info-val" id="profile-actions">—</span>
            <span class="info-key">Theme</span>
            <span class="info-val" id="profile-theme">—</span>
            <span class="info-key">Target</span>
            <span class="info-val" id="profile-target">—</span>
          </div>
        </div>
      </div>

      <div class="panel" style="margin-top: 16px;">
        <div class="panel-header">
          <span>⏱️ Stage Timings</span>
          <span id="total-ms" style="color:#888;font-weight:normal;font-size:0.85em">—</span>
        </div>
        <div class="panel-body" id="stages-body"></div>
      </div>

      <div class="panel" style="margin-top: 16px;">
        <div class="panel-header">
          <span>🛠️ Skills Used</span>
          <span id="skills-count" style="color:#888;font-weight:normal;font-size:0.85em"></span>
        </div>
        <div class="panel-body">
          <div class="skills-list" id="skills-list"></div>
        </div>
      </div>

      <div class="panel" style="margin-top: 16px;">
        <div class="panel-header">
          <span>🔍 QA Check</span>
          <span id="qa-badge" class="qa-badge">—</span>
        </div>
        <div class="panel-body">
          <div id="qa-errors"></div>
          <div id="qa-warnings"></div>
        </div>
      </div>
    </div>

    <!-- Right: Preview -->
    <div>
      <div class="panel">
        <div class="panel-header">
          <span>👁️ Preview</span>
          <div style="display:flex;gap:8px;">
            <button class="btn btn-secondary" onclick="openPreview()" style="padding:6px 14px;font-size:0.82em;">新視窗</button>
            <button class="btn btn-secondary" onclick="copyCode()" style="padding:6px 14px;font-size:0.82em;">複製</button>
          </div>
        </div>
        <iframe id="preview-frame" class="preview-iframe"></iframe>
      </div>
    </div>
  </div>
</div>

<script>
let currentResult = null;

async function generate() {
  const input = document.getElementById('intent-input');
  const useLlm = document.getElementById('use-llm').checked;
  const btn = document.getElementById('generate-btn');
  const loading = document.getElementById('loading');
  const results = document.getElementById('results');

  const intent = input.value.trim();
  if (!intent) return;

  btn.disabled = true;
  loading.classList.remove('hidden');
  results.classList.add('hidden');

  try {
    const res = await fetch('/api/generate', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ intent, use_llm: useLlm })
    });
    const data = await res.json();
    if (data.error) { alert('Error: ' + data.error); return; }
    currentResult = data;
    renderResult(data);
  } finally {
    btn.disabled = false;
    loading.classList.add('hidden');
    results.classList.remove('hidden');
  }
}

function renderResult(data) {
  // Profile
  document.getElementById('profile-type').textContent = data.profile.type;
  document.getElementById('profile-entities').textContent = data.profile.entities.join(', ') || '—';
  document.getElementById('profile-actions').textContent = data.profile.actions.join(', ') || '—';
  document.getElementById('profile-theme').textContent = data.profile.theme;
  document.getElementById('profile-target').textContent = data.profile.target;

  // LLM badge
  const llmBadge = document.getElementById('llm-badge');
  llmBadge.classList.toggle('hidden', !data.use_llm);

  // Stages
  const stages = data.stages;
  const total = stages.total;
  document.getElementById('total-ms').textContent = total + 'ms';
  const stageNames = ['classify', 'schema', 'router', 'deps', 'compose', 'qa'];
  const stageLabels = ['Classification', 'Schema', 'Router', 'Dependencies', 'Composer', 'QA'];
  let stagesHtml = '';
  stageNames.forEach((k, i) => {
    const ms = stages[k];
    const pct = Math.round((ms / total) * 100);
    stagesHtml += `<div class="stage-bar">
      <span class="stage-name">${stageLabels[i]}</span>
      <div class="bar-bg"><div class="bar-fill" style="width:${pct}%"></div></div>
      <span class="stage-ms">${ms}ms</span>
    </div>`;
  });
  document.getElementById('stages-body').innerHTML = stagesHtml;

  // Skills
  document.getElementById('skills-count').textContent = data.skills.length + ' skills';
  document.getElementById('skills-list').innerHTML = data.skills.map(s =>
    `<span class="skill-tag">${s}</span>`
  ).join('');

  // QA
  const qa = data.qa;
  const qaBadge = document.getElementById('qa-badge');
  if (qa.passed) {
    qaBadge.textContent = 'PASSED';
    qaBadge.className = 'qa-badge qa-pass';
  } else {
    qaBadge.textContent = 'FAILED';
    qaBadge.className = 'qa-badge qa-fail';
  }
  document.getElementById('qa-errors').innerHTML = qa.errors.map(e =>
    `<div class="skill-tag error-tag">✗ ${e}</div>`
  ).join('');
  document.getElementById('qa-warnings').innerHTML = qa.warnings.map(w =>
    `<div class="skill-tag warning-tag">⚠ ${w}</div>`
  ).join('');

  // Preview
  document.getElementById('preview-frame').src = data.output_url;
}

function openPreview() {
  if (currentResult) window.open(currentResult.output_url, '_blank');
}

function copyCode() {
  if (currentResult) {
    navigator.clipboard.writeText(currentResult.code);
    alert('已複製到剪貼簿');
  }
}

// Enter key to generate
document.getElementById('intent-input').addEventListener('keydown', e => {
  if (e.key === 'Enter') generate();
});
</script>
</body>
</html>
    '''
    return render_template_string(html)


if __name__ == '__main__':
    port = 3847
    print(f"Starting Evolution Compiler Web UI: http://localhost:{port}")
    app.run(host='0.0.0.0', port=port, debug=False)