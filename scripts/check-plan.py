#!/usr/bin/env python3
"""Offline validation of the planning graph and repository/wiki links."""
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
data = json.loads((ROOT / 'docs/BACKLOG.json').read_text())
tasks = data['tasks']
by_key = {task['key']: task for task in tasks}
assert len(by_key) == len(tasks), 'Duplicate work keys'
assert len({task['issue'] for task in tasks}) == len(tasks), 'Duplicate issue IDs'
required = {'key', 'title', 'milestone', 'agent_tier', 'size', 'priority', 'dependencies',
            'files', 'outcome', 'acceptance', 'validation', 'out_of_scope', 'issue', 'url',
            'execution_mode', 'initial_status'}
for task in tasks:
    assert required <= task.keys(), f"Missing fields: {task['key']}"
    assert task['agent_tier'] in {'small', 'standard', 'specialist'}
    assert task['size'] in {'S', 'M', 'L'}
    assert task['priority'] in {'P0', 'P1', 'P2'}
    assert task['acceptance'] and task['validation'] and task['out_of_scope']
    assert task['url'] == f"https://github.com/{data['repository']}/issues/{task['issue']}"
    for dep in task['dependencies']:
        assert dep == 'CORE' or dep in by_key, f"Unknown dependency {dep}"
        if dep != 'CORE':
            assert by_key[dep]['milestone'] <= task['milestone'], f"Backward milestone dependency: {task['key']}"
    if task['size'] == 'L':
        assert task['execution_mode'] == 'split-before-code'
        assert task['initial_status'] == 'deferred'
visiting, done = set(), {'CORE'}
def visit(key):
    if key in done:
        return
    assert key not in visiting, f'Dependency cycle at {key}'
    visiting.add(key)
    for dep in by_key[key]['dependencies']:
        visit(dep)
    visiting.remove(key)
    done.add(key)
for key in by_key:
    visit(key)

plan = (ROOT / 'docs/PLAN.md').read_text()
roadmap = (ROOT / 'docs/wiki/Roadmap.md').read_text()
for task in tasks:
    assert task['url'] in plan, f"Issue missing from plan: {task['key']}"
    assert task['url'] in roadmap, f"Issue missing from wiki: {task['key']}"

# This checks local file targets. External URLs are verified at publication time;
# anchor validity is intentionally not claimed by this simple Markdown check.
links = 0
for source in [ROOT / 'AGENTS.md', *sorted((ROOT / 'docs').rglob('*.md'))]:
    for target in re.findall(r'(?<!!)\[[^\]]+\]\(([^)\s]+)\)', source.read_text()):
        if target.startswith(('https://', 'http://', '#', 'mailto:')):
            continue
        destination = (source.parent / target.split('#', 1)[0]).resolve()
        assert destination.is_relative_to(ROOT), f'Link escapes checkout: {source}: {target}'
        assert destination.exists(), f'Broken link: {source}: {target}'
        links += 1
print(f'PASS: {len(tasks)} unique issues, acyclic dependencies, complete roadmap coverage, {links} local links')
