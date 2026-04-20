import re
import sys

def parse_seed(file_path: str) -> dict:
    """
    新版 Seed Architect 解析器。
    解析格式:
        <meta>...<use name="..." />...</meta>
        <constraint>...<prohibit names="..." />...</constraint>
        <body>
            <sort-array name="arr" />
            <daemon-loop />
        </body>
    """
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    # 解析 <meta>
    meta_match = re.search(r"<meta>(.*?)</meta>", content, re.DOTALL)
    if not meta_match:
        print("[-] Parser: <meta> block is required.")
        sys.exit(1)
    meta_text = meta_match.group(1)

    # <env target="..." />
    env_match = re.search(r'<env\s+target="([^"]+)"\s*/>', meta_text)
    env = env_match.group(1) if env_match else "LOCAL"

    # <use name="..." /> 清單
    uses = re.findall(r'<use\s+name="([^"]+)"\s*/>', meta_text)

    # <use name="..." attr="val">...</use> 進階格式（未來擴展）
    uses_advanced = re.findall(r'<use\s+name="([^"]+)"([^>]*)>(.*?)</use>', meta_text, re.DOTALL)

    # 解析 <constraint>
    constraint_match = re.search(r"<constraint>(.*?)</constraint>", content, re.DOTALL)
    constraints = {}
    if constraint_match:
        ct = constraint_match.group(1)

        # <prohibit names="a,b,c" />
        prohibit_match = re.search(r'<prohibit\s+names="([^"]+)"(?:\s+solution="([^"]+)")?\s*/>', ct)
        if prohibit_match:
            constraints["prohibit"] = {
                "names": [x.strip() for x in prohibit_match.group(1).split(",")],
                "solution": prohibit_match.group(2) or None
            }

        # <max-temp value="100" />
        max_temp_match = re.search(r'<max-temp\s+value="([^"]+)"\s*/>', ct)
        if max_temp_match:
            constraints["max_temp"] = int(max_temp_match.group(1))

        # <memory policy="static" />
        mem_match = re.search(r'<memory\s+policy="([^"]+)"\s*/>', ct)
        if mem_match:
            constraints["memory"] = mem_match.group(1)

    # 解析 <body>
    body_match = re.search(r"<body>(.*?)</body>", content, re.DOTALL)
    if not body_match:
        print("[-] Parser: <body> block is required.")
        sys.exit(1)
    body_text = body_match.group(1).strip()

    # 解析 body 內的元件呼叫
    # 格式: <component-name attr="val" /> 或 <component-name attr="val"></component-name>
    body_components = re.findall(r'<([\w-]+)\s+([^>]*?)(?:/>|>)', body_text)

    invocations = []
    for comp_name, attrs_text in body_components:
        attrs = {}
        # 解析屬性: name="arr" -> {"name": "arr"}
        for k, v in re.findall(r'(\w+)="([^"]*)"', attrs_text):
            attrs[k] = v
        invocations.append({
            "component": comp_name,
            "attrs": attrs
        })

    # 解析 <evolution_ref>
    evo_match = re.search(r"<evolution_ref>(.*?)</evolution_ref>", content, re.DOTALL)
    evo_ref = evo_match.group(1).strip() if evo_match else "latest"

    return {
        "meta": {
            "env": env,
            "uses": uses,
            "uses_advanced": uses_advanced
        },
        "constraints": constraints,
        "evolution_ref": evo_ref,
        "body": {
            "raw": body_text,
            "invocations": invocations
        }
    }


if __name__ == "__main__":
    # 簡單測試
    if len(sys.argv) > 1:
        result = parse_seed(sys.argv[1])
        print("Parsed result:")
        import json
        print(json.dumps(result, indent=2))
