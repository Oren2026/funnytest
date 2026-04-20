import datetime
import os

LOG_DIR = "logs"
LOG_FILE = os.path.join(LOG_DIR, "evolution_history.md")


def log_evolution(module_name: str, drift_ratio: float, logic_delta: str, healing_note: str = "N/A"):
    """記錄每次 evolution 的狀態。"""
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    if not os.path.exists(LOG_DIR):
        os.makedirs(LOG_DIR, exist_ok=True)

    if not os.path.exists(LOG_FILE):
        with open(LOG_FILE, "w", encoding="utf-8") as f:
            f.write("# Evolution History\n\n")

    entry = f"## [{timestamp}] {module_name}\n"
    entry += f"- **Drift**: {drift_ratio:.2%}\n"
    entry += f"- **Delta**: {logic_delta}\n"
    entry += f"- **Note**: {healing_note}\n\n"

    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(entry)
