#!/usr/bin/env python3
"""
Hermes Chat App - 本地 AI 聊天介面
三欄：左(agent/session) / 中(聊天室) / 右(任務輸出)
三層記憶：Ephemeral(單次) / Working Memory(resume累積) / Selective Memory(書籤)
"""
from functools import wraps
import hashlib, hmac, os, json, subprocess, re, time, secrets
from pathlib import Path
from datetime import datetime, timedelta
from flask import Flask, render_template, request, jsonify, session, make_response
from flask_socketio import SocketIO, emit

app = Flask(__name__)
app.config['SECRET_KEY'] = os.environ.get('HERMES_SECRET', 'hermes-chat-dev-key-change-me')
app.config['SESSION_COOKIE_HTTPONLY'] = True
app.config['SESSION_COOKIE_SAMESITE'] = 'Lax'
socketio = SocketIO(app, cors_allowed_origins="*", ping_timeout=60, ping_interval=25)

# ─── Auth ──────────────────────────────────────────────────
_auth_file = Path.home() / ".hermes-chat" / "auth.json"

def _load_auth():
    if _auth_file.exists():
        return json.loads(_auth_file.read_text())
    # 首次架設：自動生成 random password 並寫入
    random_pass = secrets.token_hex(8)
    new_auth = {"users": {"admin": random_pass}}
    _auth_file.parent.mkdir(parents=True, exist_ok=True)
    _auth_file.write_text(json.dumps(new_auth, indent=2))
    os.chmod(str(_auth_file), 0o600)
    print(f"\n[Hermes Chat App] 首次架設，已生成帳號密碼：")
    print(f"  帳號：admin")
    print(f"  密碼：{random_pass}")
    print(f"  請登入後立即修改密碼\n")
    return new_auth

def _check_user(username, password):
    auth = _load_auth()
    stored = auth.get("users", {}).get(username)
    if not stored:
        return False
    # plain text comparison (auth.json should be owner-readable only)
    return hmac.compare_digest(stored, password)

@app.route("/api/login", methods=["POST"])
def login():
    data = request.get_json() or {}
    u, p = data.get("username", ""), data.get("password", "")
    if _check_user(u, p):
        session["username"] = u
        return jsonify({"ok": True, "username": u})
    return jsonify({"error": "無效帳號或密碼"}), 401

@app.route("/api/logout", methods=["POST"])
def logout():
    session.clear()
    return jsonify({"ok": True})

@app.route("/api/auth/config", methods=["GET"])
def auth_config():
    """Current auth status — is user logged in?"""
    return jsonify({"logged_in": "username" in session, "username": session.get("username", "")})

@app.route("/api/auth/users", methods=["GET"])
def list_users():
    if "username" not in session:
        return jsonify({"error": "Unauthorized"}), 401
    auth = _load_auth()
    return jsonify({"users": list(auth.get("users", {}).keys())})

@app.route("/api/auth/users", methods=["POST"])
def add_user():
    if "username" not in session:
        return jsonify({"error": "Unauthorized"}), 401
    data = request.get_json()
    u, p = data.get("username", "").strip(), data.get("password", "")
    if not u or not p:
        return jsonify({"error": "username and password required"}), 400
    auth = _load_auth()
    auth.setdefault("users", {})[u] = p
    _auth_file.parent.mkdir(parents=True, exist_ok=True)
    _auth_file.write_text(json.dumps(auth, indent=2))
    os.chmod(str(_auth_file), 0o600)
    return jsonify({"ok": True})

@app.route("/api/auth/users/<username>", methods=["DELETE"])
def delete_user(username):
    if "username" not in session:
        return jsonify({"error": "Unauthorized"}), 401
    if username == session["username"]:
        return jsonify({"error": "不能刪除自己"}), 400
    auth = _load_auth()
    if username in auth.get("users", {}):
        del auth["users"][username]
        _auth_file.write_text(json.dumps(auth, indent=2))
    return jsonify({"ok": True})

@app.before_request
def require_auth():
    # Skip auth for static files, socket.io, and login endpoint
    if request.endpoint in ("login", "static") or request.path.startswith("/socket.io"):
        return
    if request.path.startswith("/api/") and "username" not in session:
        return jsonify({"error": "Unauthorized", "redirect": "/"}), 401

# ─── Agent 設定 ────────────────────────────────────────────
AGENTS = {
    "Hermes Coder":   {"color": "#3b82f6", "emoji": "🔵", "session_prefix": "coder"},
    "Hermes Research": {"color": "#8b5cf6", "emoji": "🟣", "session_prefix": "research"},
    "Hermes Creative": {"color": "#ec4899", "emoji": "🩷", "session_prefix": "creative"},
}

# ─── 核心資料結構 ─────────────────────────────────────────
# ACTIVE_SESSIONS: {
#   "sess_xxx": {
#     "agent": str,
#     "mode": "accumulating" | "ephemeral" | "selective",
#     "hermes_sid": str | None,   # Hermes CLI session ID (for accumulating/selective)
#     "history": [],               # UI 顯示用
#     "created_at": str,
#   }
# }
ACTIVE_SESSIONS = {}

# SELECTIVE_MEMORIES: [
#   { "id": str, "agent": str, "content": str, "ts": str, "tags": [] }
# ]
SELECTIVE_MEMORIES = []

MODE_LABELS = {"accumulating": "累積", "ephemeral": "單次", "selective": "書籤"}

# ─── Conversations 持久化 ─────────────────────────────────────
CONV_FILE = Path.home() / "Desktop" / "funnytest" / "conversations.json"

def load_conversations():
    """讀取所有 server-side 儲存的 conversations"""
    if CONV_FILE.exists():
        try:
            return json.loads(CONV_FILE.read_text())
        except (json.JSONDecodeError, IOError):
            return {}
    return {}

def save_conversations(data):
    """寫入 conversations 到磁碟"""
    CONV_FILE.parent.mkdir(parents=True, exist_ok=True)
    CONV_FILE.write_text(json.dumps(data, indent=2, ensure_ascii=False))

def conv_key(agent, sess_id):
    return f"{agent}|{sess_id}"

def append_conversation(agent, sess_id, role, content):
    """將訊息附加到指定 conversation，並持久化"""
    data = load_conversations()
    key = conv_key(agent, sess_id)
    if key not in data:
        data[key] = []
    data[key].append({
        "role": role,
        "content": content,
        "ts": datetime.now().isoformat(),
    })
    save_conversations(data)

# ─── 工具函式 ──────────────────────────────────────────────

def strip_ansi(text):
    import re
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    return ansi_escape.sub('', text)

def clean_hermes_output(text):
    """清理 Hermes CLI 輸出：只留回覆本體，移除 git diff 噪聲"""
    text = strip_ansi(text)
    # 嘗試找 box drawing 範圍
    last_u = text.rfind('╭')
    last_L = text.rfind('╰')
    if last_u != -1 and last_L != -1 and last_L > last_u:
        block = text[last_u:last_L+1]
        lines = block.splitlines()
        content = '\n'.join(
            l for l in lines
            if l.strip() and '─' not in l and not l.strip().startswith('╭') and not l.strip().startswith('╰')
        ).strip()
        if content:
            return content
    # 嘗試找 Query: 以後的內容
    if "Query:" in text:
        qidx = text.rfind("Query:")
        snippet = text[qidx:]
        if "Resume" in snippet:
            snippet = snippet[:snippet.find("Resume")]
        return snippet.strip()
    # Fallback：移除 git diff 噪聲，取 actual content
    lines = text.splitlines()
    clean_lines = []
    skip_patterns = ("┊", "diff --git", "index ", "@@ ", "--- a/", "+++ b/",
                     "old mode", "new mode", "similarity", "rename from",
                     "rename to", "review diff", "Binary files")
    for l in lines:
        if any(l.strip().startswith(p) or p in l for p in skip_patterns):
            continue
        if re.match(r"^[+\-\s\\| ]*$", l):  # 只剩 diff +/+/空白
            continue
        clean_lines.append(l)
    result = '\n'.join(clean_lines).strip()
    # 如果清理後太短（<10字），用原始文字
    return result if len(result) > 10 else text.strip()

def stream_output(stream_type, content):
    socketio.emit('output', {
        "type": stream_type,
        "content": content,
        "timestamp": datetime.now().isoformat(),
    })

HERMES_BIN = "/Users/oren/.local/bin/hermes"

def call_hermes(agent_key, message, history=None, hermes_sid=None, mode="accumulating"):
    """
    叫用 Hermes CLI。
    - ephemeral: 純單次，無歷史
    - accumulating/selective: 將 history prepend 成 context
    - hermes_sid: 追蹤用，不傳給 CLI
    返回 (response_text, hermes_sid)
    """
    is_acc = mode in ("accumulating", "selective")

    # 安全模式：Agent 只能透過對話輸出程式碼，不能寫入任何檔案
    SAFE_CONTEXT = (
        "【系統指示】\n"
        "你是一個安全的程式碼助手。嚴禁將任何內容寫入磁碟。\n"
        "所有輸出必須直接顯示在對話回覆中：\n"
        "- HTML/CSS/JS：使用 ```html code block``` 包裹\n"
        "- Python：使用 ```python code block``` 包裹\n"
        "- 其他語言：使用對應的 ```語言名 code block```\n"
        "如果需要預覽，用 iframe 直接 render HTML，不留檔案。\n"
        "禁止使用 write_file、echo、cat > 等任何寫檔指令。\n"
        "禁止路徑：禁止寫入 ~/Desktop/、/tmp/、/var/ 或任何本機路徑。\n\n"
    )

    # 組合 prompt：歷史 context + 當前訊息
    # 注意：assistant 回應中的 code block 要移除，避免大型 HTML/Python code 搞亂 prompt
    if is_acc and history:
        context_lines = []
        for m in history[-20:]:
            role = "User" if m["role"] == "user" else "Assistant"
            content = m["content"]
            # 移除 code block 避免 prompt 膨脹且混淆模型
            content = re.sub(r"```[\s\S]*?```", "[程式碼 block 已移除]", m["content"])
            content = re.sub(r"`[^`]+`", "", content)
            context_lines.append(f"{role}: {content}")
        context = "\n".join(context_lines)
        full_prompt = SAFE_CONTEXT + f"[對話歷史]\n{context}\n\n[本次訊息]\nUser: {message}\nAssistant:"
    else:
        full_prompt = SAFE_CONTEXT + message

    cmd = [HERMES_BIN, "chat", "-q", full_prompt, "-Q"]

    stream_output("status", f"🤖 {agent_key} [{MODE_LABELS.get(mode, mode)}] 處理中...")

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300,
            env={**os.environ, "HOME": str(Path.home())},
        )

        stdout = result.stdout
        stderr = result.stderr

        if stdout:
            stream_output("stdout", stdout)
        if stderr:
            stream_output("stderr", stderr)

        raw = stdout if stdout else stderr
        if not raw:
            return "[無回應]", hermes_sid

        # 解析 session_id（-Q 模式下：{"session_id": "..."}\nresponse）
        sid = hermes_sid
        sid_m = re.search(r'"session_id":\s*"([^"]+)"', raw)
        if sid_m:
            sid = sid_m.group(1)

        response = clean_hermes_output(raw)
        return response if response else "[無回應]", sid

    except subprocess.TimeoutExpired:
        stream_output("stderr", "⏱️  回應逾時（300秒）")
        return "抱歉，回應時間過長（300秒）。", hermes_sid
    except Exception as e:
        stream_output("stderr", f"❌ 錯誤：{str(e)}")
        return f"執行錯誤：{str(e)}", hermes_sid

# ─── API 路由 ─────────────────────────────────────────────

@app.route("/")
def index():
    return render_template("index.html", agents=list(AGENTS.keys()))

@app.route("/api/chat", methods=["POST"])
def chat():
    data = request.json
    agent_key = data.get("agent", "Hermes Coder")
    message = data.get("message", "").strip()
    sess_id = data.get("session_id", "").strip()
    mode = data.get("mode", "accumulating")

    if not message:
        return jsonify({"error": "空訊息"}), 400

    # 初始化或更新 session
    if sess_id not in ACTIVE_SESSIONS:
        ACTIVE_SESSIONS[sess_id] = {
            "agent": agent_key,
            "mode": mode,
            "hermes_sid": None,
            "history": [],
            "created_at": datetime.now().isoformat(),
        }
    else:
        ACTIVE_SESSIONS[sess_id]["mode"] = mode

    sess = ACTIVE_SESSIONS[sess_id]
    hermes_sid = sess["hermes_sid"]

    if mode == "ephemeral":
        # 單次：不留歷史、不追 session
        response, new_sid = call_hermes(agent_key, message, history=None, hermes_sid=None, mode="ephemeral")
        return jsonify({
            "response": response,
            "agent": agent_key,
            "session_id": sess_id,
            "mode": "ephemeral",
            "hermes_sid": None,
            "timestamp": datetime.now().isoformat(),
        })
    else:
        # 累積 / 書籤：帶歷史 context
        response, new_sid = call_hermes(
            agent_key, message,
            history=sess["history"],
            hermes_sid=hermes_sid,
            mode=mode,
        )
        sess["hermes_sid"] = new_sid
        sess["history"].append({"role": "user", "content": message})
        sess["history"].append({"role": "assistant", "content": response})
        # 持久化到 server-side file
        append_conversation(agent_key, sess_id, "user", message)
        append_conversation(agent_key, sess_id, "assistant", response)

        return jsonify({
            "response": response,
            "agent": agent_key,
            "session_id": sess_id,
            "mode": mode,
            "hermes_sid": new_sid,
            "timestamp": datetime.now().isoformat(),
        })

@app.route("/api/sessions", methods=["GET"])
def list_sessions():
    """列出所有 sessions，合併 Active Sessions + conversations.json"""
    conv_data = load_conversations()
    result = []

    # 1. Active Sessions
    for sid, s in ACTIVE_SESSIONS.items():
        result.append({
            "id": sid,
            "agent": s["agent"],
            "mode": s["mode"],
            "hermes_sid": s["hermes_sid"],
            "history_len": len(s["history"]),
            "created_at": s["created_at"],
            "source": "active",
        })

    # 2. conversations.json 裡有但 active 沒有的（Server 重啟後的殘留 session）
    seen_ids = {s["id"] for s in result}
    for key, msgs in conv_data.items():
        if "|" not in key:
            continue
        agent, sid = key.split("|", 1)
        if sid in seen_ids:
            continue
        first_msg = msgs[0] if msgs else {}
        result.append({
            "id": sid,
            "agent": agent,
            "mode": "accumulating",
            "hermes_sid": None,
            "history_len": len(msgs),
            "created_at": first_msg.get("ts", datetime.now().isoformat()),
            "source": "file",
        })

    # 按 created_at 降序
    result.sort(key=lambda x: x.get("created_at", ""), reverse=True)
    return jsonify({"sessions": result})

@app.route("/api/sessions/<sess_id>", methods=["DELETE"])
def delete_session(sess_id):
    """刪除 session（從 Active Sessions 和 conversations.json 同時移除）"""
    if sess_id in ACTIVE_SESSIONS:
        del ACTIVE_SESSIONS[sess_id]
    # 也從 conversations.json 移除
    data = load_conversations()
    keys_to_remove = [k for k in data if k.endswith(f"|{sess_id}")]
    for k in keys_to_remove:
        del data[k]
    if keys_to_remove:
        save_conversations(data)
    return jsonify({"ok": True})

@app.route("/api/sessions/<sess_id>", methods=["PATCH"])
def update_session(sess_id):
    """更新 session 模式"""
    if sess_id not in ACTIVE_SESSIONS:
        return jsonify({"error": "session 不存在"}), 404
    data = request.json
    if "mode" in data:
        ACTIVE_SESSIONS[sess_id]["mode"] = data["mode"]
    return jsonify({"ok": True, "session": ACTIVE_SESSIONS[sess_id]})

@app.route("/api/conversations/<path:agent>/<sess_id>", methods=["GET"])
def get_conversation(agent, sess_id):
    """讀取指定 agent|session 的完整對話歷史（server-side persistence，跨瀏覽器共享）"""
    conv_data = load_conversations()
    key = conv_key(agent, sess_id)
    messages = conv_data.get(key, [])
    return jsonify({"agent": agent, "session_id": sess_id, "messages": messages})

@app.route("/api/memories", methods=["GET"])
def list_memories():
    return jsonify({"memories": SELECTIVE_MEMORIES})

@app.route("/api/memories", methods=["POST"])
def add_memory():
    data = request.json
    memory = {
        "id": f"mem_{int(time.time()*1000)}",
        "agent": data.get("agent", ""),
        "content": data.get("content", ""),
        "tags": data.get("tags", []),
        "ts": datetime.now().isoformat(),
    }
    SELECTIVE_MEMORIES.append(memory)
    return jsonify({"ok": True, "memory": memory})

@app.route("/api/memories/<mem_id>", methods=["DELETE"])
def delete_memory(mem_id):
    global SELECTIVE_MEMORIES
    SELECTIVE_MEMORIES = [m for m in SELECTIVE_MEMORIES if m["id"] != mem_id]
    return jsonify({"ok": True})

# ─── Skill Library ──────────────────────────────────────────

SKILL_LIB = Path.home() / "Desktop" / "funnytest" / "skill_library"
AGENT_SKILLS_FILE = Path.home() / "Desktop" / "funnytest" / "agent_skills.json"

def get_agent_skills():
    if AGENT_SKILLS_FILE.exists():
        return json.loads(AGENT_SKILLS_FILE.read_text())
    return {}

def save_agent_skills(data):
    AGENT_SKILLS_FILE.write_text(json.dumps(data, indent=2))

def get_all_skills():
    """列出 skill_library 中所有 skill 的 metadata"""
    result = []
    if not SKILL_LIB.exists():
        return result
    for d in sorted(SKILL_LIB.iterdir()):
        meta_path = d / "metadata.json"
        if meta_path.exists():
            result.append(json.loads(meta_path.read_text()))
        else:
            result.append({"name": d.name, "description": "", "tags": [], "path": str(d)})
    return result

@app.route("/api/skills", methods=["GET"])
def list_skills():
    return jsonify(get_all_skills())

@app.route("/api/agent-skills", methods=["GET"])
def get_agent_skills_api():
    """取得每個 Agent 擁有的 Skills（含詳細 metadata）"""
    agent_map = get_agent_skills()
    all_skills = {s["name"]: s for s in get_all_skills()}
    result = {}
    for agent, skill_names in agent_map.items():
        result[agent] = [all_skills.get(s, {"name": s, "description": "", "tags": []}) for s in skill_names]
    return jsonify(result)

@app.route("/api/agent-skills/<agent>/<skill>", methods=["POST"])
def add_skill_to_agent(agent, skill):
    """將 skill 新增到 agent"""
    agent_map = get_agent_skills()
    if agent not in agent_map:
        agent_map[agent] = []
    if skill not in agent_map[agent]:
        agent_map[agent].append(skill)
    save_agent_skills(agent_map)
    return jsonify({"ok": True, "agent": agent, "skill": skill})

@app.route("/api/agent-skills/<agent>/<skill>", methods=["DELETE"])
def remove_skill_from_agent(agent, skill):
    """將 skill 從 agent 移除"""
    agent_map = get_agent_skills()
    if agent in agent_map and skill in agent_map[agent]:
        agent_map[agent].remove(skill)
        save_agent_skills(agent_map)
    return jsonify({"ok": True, "agent": agent, "skill": skill})

# ─── Subagent 管理 ──────────────────────────────────────────

SUBAGENTS_FILE = Path.home() / "Desktop" / "funnytest" / "subagents.json"

def get_subagents():
    if SUBAGENTS_FILE.exists():
        return json.loads(SUBAGENTS_FILE.read_text())
    return []

def save_subagents(data):
    SUBAGENTS_FILE.write_text(json.dumps(data, indent=2))

@app.route("/api/subagents", methods=["GET"])
def list_subagents():
    return jsonify(get_subagents())

@app.route("/api/subagents", methods=["POST"])
def create_subagent():
    body = request.get_json()
    name = body.get("name", "").strip()
    if not name:
        return jsonify({"error": "name is required"}), 400
    subagents = get_subagents()
    new_id = "sub_" + str(len(subagents) + 1)
    subagents.append({
        "id": new_id,
        "name": name,
        "description": body.get("description", ""),
        "skills": body.get("skills", []),
        "status": "idle",
        "system_prompt": body.get("system_prompt", ""),
        "created_at": datetime.now().isoformat() + "Z",
    })
    save_subagents(subagents)
    return jsonify({"ok": True, "id": new_id}), 201

@app.route("/api/subagents/<sub_id>", methods=["PATCH"])
def update_subagent(sub_id):
    body = request.get_json()
    subagents = get_subagents()
    for s in subagents:
        if s["id"] == sub_id:
            if "name" in body: s["name"] = body["name"]
            if "description" in body: s["description"] = body["description"]
            if "skills" in body: s["skills"] = body["skills"]
            if "status" in body: s["status"] = body["status"]
            if "system_prompt" in body: s["system_prompt"] = body["system_prompt"]
            save_subagents(subagents)
            return jsonify({"ok": True})
    return jsonify({"error": "not found"}), 404

@app.route("/api/subagents/<sub_id>", methods=["DELETE"])
def delete_subagent(sub_id):
    subagents = get_subagents()
    before = len(subagents)
    subagents = [s for s in subagents if s["id"] != sub_id]
    if len(subagents) == before:
        return jsonify({"error": "not found"}), 404
    save_subagents(subagents)
    return jsonify({"ok": True})

# ─── Subagent Invocation ─────────────────────────────────────

def load_skills_for_subagent(skill_names):
    """讀取指定 skills 的 SKILL.md 內容，組合成提示文本"""
    if not skill_names:
        return ""
    lines = ["[Skills Available]", "你可以使用以下技能來完成任務："]
    for name in skill_names:
        skill_path = SKILL_LIB / name / "SKILL.md"
        if skill_path.exists():
            content = skill_path.read_text().strip()
            lines.append(f"\n--- {name} ---\n{content}")
        else:
            lines.append(f"\n--- {name} ---\n（Skill 檔案不存在）")
    return "\n".join(lines) + "\n"

def invoke_subagent(sub_id, task_description):
    """執行一個 Subagent，返回 (result_text, error_or_none)"""
    subagents = get_subagents()
    sub = next((s for s in subagents if s["id"] == sub_id), None)
    if not sub:
        return None, "Subagent not found"

    # Subagent 安全提示（與主 agent 相同約束）
    SAFE_CONTEXT = (
        "【系統指示】\n"
        "你是一個安全的程式碼助手。嚴禁將任何內容寫入磁碟。\n"
        "所有輸出必須直接顯示在對話回覆中：\n"
        "- HTML/CSS/JS：使用 ```html code block``` 包裹\n"
        "- Python：使用 ```python code block``` 包裹\n"
        "- 其他語言：使用對應的 ```語言名 code block```\n"
        "禁止使用 write_file、echo、cat > 等任何寫檔指令。\n"
        "禁止路徑：禁止寫入 ~/Desktop/、/tmp/、/var/ 或任何本機路徑。\n\n"
    )

    # 組合 prompt
    skills_block = load_skills_for_subagent(sub.get("skills", []))
    system_prompt = sub.get("system_prompt", "")

    full_prompt = (
        SAFE_CONTEXT + "\n" +
        f"{system_prompt}\n\n"
        f"{skills_block}\n"
        f"[Task]\n{task_description}\n\n"
        f"[Output Format]\n"
        f"直接輸出結果，不需要前置解釋。\n"
    )

    # 執行 Hermes CLI
    try:
        result = subprocess.run(
            [HERMES_BIN, "chat", "-q", full_prompt, "-Q"],
            capture_output=True,
            text=True,
            timeout=120,
            env={**os.environ, "HOME": str(Path.home())},
        )
        output = clean_hermes_output(result.stdout) if result.stdout else (result.stderr or "[無輸出]")
    except subprocess.TimeoutExpired:
        output = "[Subagent 執行逾時（120秒）]"
    except Exception as e:
        return None, str(e)

    # 寫入 invocation_log
    sub["invocation_log"] = sub.get("invocation_log", [])
    sub["invocation_log"].append({
        "task": task_description[:100],
        "result": output[:200] if len(output) > 200 else output,
        "at": datetime.now().isoformat() + "Z",
    })
    # 只保留最近 10 筆
    sub["invocation_log"] = sub["invocation_log"][-10:]
    sub["status"] = "idle"
    save_subagents(subagents)

    return output, None

@app.route("/api/subagent/invoke", methods=["POST"])
def api_invoke_subagent():
    """叫用指定 Subagent 處理任務"""
    if "username" not in session:
        return jsonify({"error": "Unauthorized"}), 401
    data = request.get_json() or {}
    sub_id = data.get("subagent_id", "")
    task = data.get("task", "").strip()
    if not sub_id or not task:
        return jsonify({"error": "subagent_id and task required"}), 400

    # 標記 subagent 為 busy
    subagents = get_subagents()
    for s in subagents:
        if s["id"] == sub_id:
            s["status"] = "busy"
            save_subagents(subagents)
            break

    result, err = invoke_subagent(sub_id, task)
    if err:
        return jsonify({"error": err}), 400
    return jsonify({"ok": True, "result": result, "subagent_id": sub_id})

# ─── WebSocket ─────────────────────────────────────────────

@socketio.on("connect")
def on_connect():
    if "username" not in session:
        return False  # reject connection
    emit("connected", {"status": "ok"})

@socketio.on("disconnect")
def on_disconnect():
    pass

# ─── 啟動 ──────────────────────────────────────────────────

if __name__ == "__main__":
    print("🚀 Hermes Chat App — 三層記憶系統")
    print("   http://localhost:5177")
    socketio.run(app, host="0.0.0.0", port=5177, debug=True, allow_unsafe_werkzeug=True)
