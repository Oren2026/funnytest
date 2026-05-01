"""節點 4：Dependency Resolver（依賴解析器）"""
from typing import List, Dict, Set
from pathlib import Path
import re


def load_skill_depends(skill_name: str, skills_base: Path) -> List[str]:
    """從 skill 檔案讀取 # depends: 宣告。"""
    for subdir in ["ui", "styles", "core", "algorithms", "structures", "system", "behaviors"]:
        skill_dir = skills_base / subdir
        if not skill_dir.exists():
            continue
        matches = list(skill_dir.glob(f"{skill_name}.skill"))
        if matches:
            content = matches[0].read_text()
            match = re.search(r'^#\s*depends:\s*(.+)$', content, re.MULTILINE)
            if match:
                deps = match.group(1).strip()
                if deps.lower() == "none":
                    return []
                return [d.strip() for d in deps.split(",")]
            return []
    return []


def resolve_dependencies(skill_names: List[str], skills_base: Path) -> List[Dict]:
    """
    Kahn's topological sort：根據 # depends 宣告排序技能。

    Returns:
        [{skill, depends, resolved}] 按依賴順序排列

    Raises:
        ValueError: 檢測到循環依賴
    """
    # 建立鄰接表和入度表
    graph: Dict[str, List[str]] = {}
    in_degree: Dict[str, int] = {}
    all_nodes: Set[str] = set(skill_names)

    for skill in skill_names:
        graph[skill] = []
        in_degree[skill] = 0

    # 擴展依賴：遞迴載入每個技能的依賴
    def expand_deps(skill: str, visited: Set[str]) -> List[str]:
        """返回 skill 的完整依賴列表（遞迴）。"""
        if skill in visited:
            return []
        visited.add(skill)
        deps = load_skill_depends(skill, skills_base)
        all_nodes.update(deps)
        result = deps[:]
        for dep in deps:
            result.extend(expand_deps(dep, visited))
        return result

    expanded = {}
    for skill in skill_names:
        deps = load_skill_depends(skill, skills_base)
        expanded[skill] = deps
        all_nodes.update(deps)

    # 建立圖
    for skill, deps in expanded.items():
        graph[skill] = deps
        for dep in deps:
            if dep not in graph:
                graph[dep] = []
                in_degree[dep] = 0
            in_degree[dep] += 1

    # Kahn's algorithm
    # 初始佇列：所有入度=0 的節點，按字母順序排序（穩定 tiebreaker）
    queue = sorted([n for n in all_nodes if in_degree.get(n, 0) == 0])
    ordered = []

    while queue:
        # 取佇列第一個（已排序），並對鄰居遞減 in-degree
        node = queue.pop(0)
        ordered.append(node)
        for neighbor in graph.get(node, []):
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0 and neighbor not in ordered:
                # 維持佇列排序（in-degree=0 的新節點插入維持順序）
                queue.append(neighbor)
                queue.sort()  # 保持字母順序

    # 檢查循環
    if len(ordered) != len(all_nodes):
        # 找出循環中的節點
        remaining = set(all_nodes) - set(ordered)
        raise ValueError(f"循環依賴檢測：{remaining}")

    # 按原始順序穩定輸出（已按依賴順序）
    ordered = [n for n in ordered if n in skill_names or n in set().union(*[set(expanded.get(s, [])) for s in skill_names])]

    result = []
    for skill in ordered:
        deps = expanded.get(skill, [])
        # 只包含在原始列表或已擴展依賴中的
        if skill in all_nodes:
            result.append({"skill": skill, "depends": deps})

    # 過濾：只保留原始技能名單（但按依賴順序）
    final = []
    for item in result:
        if item["skill"] in skill_names:
            final.append(item)

    # 加入沒有依賴但可能被擴展出的技能
    for skill in skill_names:
        if skill not in [f["skill"] for f in final]:
            final.append({"skill": skill, "depends": []})

    return final
