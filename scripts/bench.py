#!/usr/bin/env python3
"""Measure Nivra capture overhead against synthetic fixtures.

Everything runs in a temporary directory with its own HOME, Git config and
NIVRA_DATA_DIR. No user repository, history or database is touched.

Usage: cargo build --release --locked && python3 scripts/bench.py [--reps 30]
"""
import argparse
import datetime
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
# The same Git queries git::capture issues, in order, so their cost can be
# separated from Nivra's own work (process start, hashing, SQLite).
GIT_QUERIES = [
    ['rev-parse', '--show-toplevel'],
    ['symbolic-ref', '--quiet', '--short', 'HEAD'],
    ['rev-parse', '--verify', 'HEAD'],
    ['status', '--porcelain=v1', '-z', '--untracked-files=all', '--ignore-submodules=all'],
]


def git(cwd, *args, env):
    subprocess.run(['git', *args], cwd=cwd, env=env, check=True, capture_output=True)


def write_files(root, count, size):
    for i in range(count):
        path = root / f'd{i // 500:03}' / f'f{i:05}.txt'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((f'{i:08}\n' * (size // 9 + 1)).encode()[:size])


def fixtures(base, env):
    """Create the three scenarios and return their descriptions."""
    nongit = base / 'nongit'
    nongit.mkdir()
    (nongit / 'notes.txt').write_text('not a repository\n')

    small = base / 'small'
    small.mkdir()
    write_files(small, 50, 1024)
    git(small, 'init', '-q', '-b', 'main', env=env)
    git(small, 'add', '-A', env=env)
    git(small, 'commit', '-q', '-m', 'fixture', env=env)

    large = base / 'large'
    large.mkdir()
    write_files(large, 20_000, 512)
    git(large, 'init', '-q', '-b', 'main', env=env)
    git(large, 'add', '-A', env=env)
    git(large, 'commit', '-q', '-m', 'fixture', env=env)
    # Dirty state: 300 edited tracked files, 50 untracked, 5 large edited
    # files that dominate fingerprinting (each below the 8 MiB limit).
    for i in range(0, 20_000, 20_000 // 300):
        path = large / f'd{i // 500:03}' / f'f{i:05}.txt'
        path.write_bytes(path.read_bytes() + b'edited\n')
    write_files(large / 'untracked', 50, 512)
    for i in range(5):
        (large / f'd000/f{i:05}.txt').write_bytes(os.urandom(4 * 1024 * 1024))

    return [
        {'name': 'non-git', 'path': nongit, 'files': 1, 'changed': 0, 'bytes': 17},
        {'name': 'small-clean', 'path': small, 'files': 50, 'changed': 0, 'bytes': 50 * 1024},
        {'name': 'large-dirty', 'path': large, 'files': 20_050, 'changed': 355,
         'bytes': 20_050 * 512 + 5 * 4 * 1024 * 1024,
         'note': '300 small edits, 50 untracked, 5 edited 4 MiB files'},
    ]


def sample(fn, reps, warmup):
    for _ in range(warmup):
        fn()
    out = []
    for _ in range(reps):
        start = time.perf_counter_ns()
        fn()
        out.append((time.perf_counter_ns() - start) / 1e6)
    return out


def summary(values):
    ordered = sorted(values)
    # Nearest-rank percentiles: every reported number is an observed sample.
    def rank(p):
        return ordered[max(0, -(-len(ordered) * p // 100) - 1)]
    return {'median_ms': round(rank(50), 2), 'p95_ms': round(rank(95), 2),
            'min_ms': round(ordered[0], 2), 'max_ms': round(ordered[-1], 2)}


def measure(binary, scenario, env, reps, warmup):
    cwd = scenario['path']

    def startup():
        subprocess.run([binary, '--version'], cwd=cwd, env=env, check=True, capture_output=True)

    def git_only():
        for query in GIT_QUERIES:
            # capture stops after the first query when there is no repository.
            if subprocess.run(['git', *query], cwd=cwd, env=env, capture_output=True).returncode and query == GIT_QUERIES[0]:
                break

    ids = []

    def hook_start():
        result = subprocess.run([binary, 'hook', 'start', '--session', 'bench'], cwd=cwd, env=env,
                                input=b'true', check=True, capture_output=True)
        ids.append(result.stdout.decode().strip())

    def hook_end():
        subprocess.run([binary, 'hook', 'end', '--id', ids.pop(), '--exit-code', '0'], cwd=cwd,
                       env=env, check=True, capture_output=True)

    raw = {'startup': sample(startup, reps, warmup), 'git_only': sample(git_only, reps, warmup),
           'hook_start': sample(hook_start, reps, warmup)}
    # Each hook end consumes an event opened above, newest first.
    raw['hook_end'] = sample(hook_end, reps, min(warmup, len(ids) - reps))
    stats = {name: summary(values) for name, values in raw.items()}
    stats['nivra_own_estimate_ms'] = round(stats['hook_start']['median_ms'] - stats['git_only']['median_ms'], 2)
    return raw, stats


def environment(binary):
    def out(*cmd):
        try:
            return subprocess.run(cmd, capture_output=True, text=True, check=True).stdout.strip()
        except (OSError, subprocess.CalledProcessError):
            return 'unknown'
    cpu = out('sysctl', '-n', 'machdep.cpu.brand_string') if platform.system() == 'Darwin' else platform.processor()
    return {'date': datetime.date.today().isoformat(), 'os': f'{platform.system()} {platform.release()}',
            'machine': platform.machine(), 'cpu': cpu, 'rustc': out('rustc', '-V'), 'git': out('git', '--version'),
            'nivra': out(binary, '--version'), 'commit': out('git', '-C', str(ROOT), 'rev-parse', '--short', 'HEAD')}


def markdown(env_info, results, reps, warmup):
    lines = [f"# Capture benchmark · {env_info['date']}", '',
             'Synthetic fixtures only. Reproduce with `cargo build --release --locked && python3 scripts/bench.py`.', '',
             '| | |', '| --- | --- |']
    lines += [f'| {key} | {value} |' for key, value in env_info.items()]
    lines += ['', f'{reps} measured runs after {warmup} warmups per row. Times in milliseconds (median / p95).', '',
              '| Scenario | Fixture | startup | git only | hook start | hook end | Nivra own (est.) |',
              '| --- | --- | --- | --- | --- | --- | --- |']
    for scenario, stats in results:
        size = scenario['bytes']
        size = f'{size / 1048576:.1f} MiB' if size >= 1048576 else f'{size / 1024:.0f} KiB' if size >= 1024 else f'{size} B'
        fixture = f"{scenario['files']} files, {scenario['changed']} changed, {size}"
        cells = [f"{stats[k]['median_ms']} / {stats[k]['p95_ms']}" for k in ('startup', 'git_only', 'hook_start', 'hook_end')]
        lines.append(f"| {scenario['name']} | {fixture} | " + ' | '.join(cells) + f" | {stats['nivra_own_estimate_ms']} |")
    lines += ['', '- **startup**: `nivra --version`, the process floor.',
              '- **git only**: the four Git queries `git::capture` issues, run without Nivra.',
              '- **hook start / end**: what `preexec` / `precmd` wait for; each performs a full snapshot and one SQLite write.',
              '- **Nivra own (est.)**: median hook start minus median git only. It covers process start, fingerprint hashing and SQLite. '
              'A difference of medians is an estimate, not a measured component.',
              '', 'No threshold is enforced in CI; results depend on hardware, filesystem cache and Git version.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('--reps', type=int, default=30)
    parser.add_argument('--warmup', type=int, default=5)
    parser.add_argument('--out', type=Path, default=ROOT / 'docs' / 'benchmarks')
    args = parser.parse_args()
    binary = str(Path(os.environ.get('NIVRA_BIN', ROOT / 'target' / 'release' / 'nivra')).resolve())
    env_info = environment(binary)

    with tempfile.TemporaryDirectory(prefix='nivra-bench-') as folder:
        base = Path(folder)
        (base / 'home').mkdir()
        env = {'PATH': os.environ['PATH'], 'HOME': str(base / 'home'), 'NIVRA_DATA_DIR': str(base / 'data'),
               'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_NOSYSTEM': '1', 'LC_ALL': 'C',
               'GIT_AUTHOR_NAME': 'bench', 'GIT_AUTHOR_EMAIL': 'bench@example.invalid',
               'GIT_COMMITTER_NAME': 'bench', 'GIT_COMMITTER_EMAIL': 'bench@example.invalid'}
        scenarios = fixtures(base, env)
        results, raw_all = [], {}
        for scenario in scenarios:
            print(f"measuring {scenario['name']}…", flush=True)
            raw, stats = measure(binary, scenario, env, args.reps, args.warmup)
            results.append((scenario, stats))
            raw_all[scenario['name']] = {'fixture': {k: v for k, v in scenario.items() if k != 'path'},
                                         'summary': stats, 'samples_ms': raw}

    args.out.mkdir(parents=True, exist_ok=True)
    stem = f"{env_info['date']}-{platform.system().lower()}"
    (args.out / f'{stem}.json').write_text(json.dumps({'environment': env_info, 'reps': args.reps,
                                                       'warmup': args.warmup, 'scenarios': raw_all}, indent=2) + '\n')
    report = markdown(env_info, results, args.reps, args.warmup)
    (args.out / f'{stem}.md').write_text(report)
    print(report)


if __name__ == '__main__':
    main()
