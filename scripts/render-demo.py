#!/usr/bin/env python3
"""Render a terminal card from real demo output; no invented command results."""
import html
from pathlib import Path
import re

source = Path('assets/demo.txt')
text = source.read_text()
text = re.sub(r'(?m)^(  REPOSITORY  ).+$', r'\1/tmp/nivra-demo/repo', text)
source.write_text(text)
lines = text.strip('\n').splitlines()
height = 182 + len(lines) * 23
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="1080" height="{height}" viewBox="0 0 1080 {height}">',
'<rect width="1080" height="100%" rx="20" fill="#0b1018"/>',
'<rect x="1" y="1" width="1078" height="100%" rx="20" fill="none" stroke="#253044"/>',
'<text x="40" y="48" fill="#9fe8da" font-family="monospace" font-size="15" letter-spacing="5">NIVRA</text>',
'<text x="40" y="89" fill="#f2f5fa" font-family="sans-serif" font-size="28" font-weight="600">It worked. Then it broke. See what changed.</text>',
'<text x="40" y="116" fill="#8291a7" font-family="sans-serif" font-size="14">LOCAL DEVELOPMENT TIMELINE · REAL CLI OUTPUT · EARLY ALPHA</text>',
'<path d="M40 138H1040" stroke="#253044"/>']
for index, line in enumerate(lines):
    color = '#b4c0d2'
    if line.startswith('$'): color = '#9fe8da'
    elif '✕' in line: color = '#f69ea6'
    elif '✓' in line: color = '#a5d6b0'
    elif '◆' in line or 'WORKTREE CHANGES' in line: color = '#d4b0ff'
    elif 'N I V R A' in line: color = '#f2f5fa'
    svg.append(f'<text xml:space="preserve" x="40" y="{169+index*23}" fill="{color}" font-family="Menlo,Consolas,monospace" font-size="15">{html.escape(line)}</text>')
svg.append('</svg>')
Path('assets/demo.svg').write_text('\n'.join(svg))
