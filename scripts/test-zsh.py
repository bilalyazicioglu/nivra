#!/usr/bin/env python3
"""Exercise actual interactive zsh hooks through a PTY.

Every scenario runs in its own temporary HOME, ZDOTDIR and NIVRA_DATA_DIR. The
invariant under test: Nivra must never make the shell less reliable. Commands
keep their exit status and the prompt returns promptly, whatever state
Nivra's storage, Git or binary is in.
"""
import json
import os
from pathlib import Path
import pty
import select
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time

binary = str(Path(os.environ.get('NIVRA_BIN', 'target/debug/nivra')).resolve())
PROMPT = b'NIVRA_TEST> '
# A capture failure may cost a bounded delay, never a hang.
PROMPT_BUDGET_S = 3.0


class Shell:
    def __init__(self, folder):
        self.folder = Path(folder)
        for name in ('home', 'zdotdir', 'data'):
            (self.folder / name).mkdir()
        self.env = {
            'PATH': os.environ['PATH'], 'HOME': str(self.folder / 'home'),
            'ZDOTDIR': str(self.folder / 'zdotdir'), 'NIVRA_DATA_DIR': str(self.folder / 'data'),
            'TERM': 'dumb', 'PS1': PROMPT.decode(), 'RPS1': '', 'LC_ALL': 'C',
            'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_NOSYSTEM': '1',
            'GIT_AUTHOR_NAME': 't', 'GIT_AUTHOR_EMAIL': 't@example.invalid',
            'GIT_COMMITTER_NAME': 't', 'GIT_COMMITTER_EMAIL': 't@example.invalid',
        }
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.chdir(self.folder)
            os.execvpe('zsh', ['zsh', '-f', '-i', '-o', 'NO_ZLE', '-o', 'NO_PROMPT_SP'], self.env)
        self.until_prompt(10)

    def until_prompt(self, timeout=PROMPT_BUDGET_S):
        output = b''
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if select.select([self.fd], [], [], 0.05)[0]:
                output += os.read(self.fd, 65536)
                if output.endswith(PROMPT):
                    return output
        raise AssertionError(f'prompt did not return within {timeout}s: {output!r}')

    def send(self, text, timeout=PROMPT_BUDGET_S):
        os.write(self.fd, text.encode() + b'\n')
        return self.until_prompt(timeout)

    def status_of(self, command):
        """Run a command and return the exit status zsh observed afterwards."""
        self.send(command)
        out = self.send('print -r -- STATUS=$?')
        for line in out.split(b'\r\n'):
            if line.startswith(b'STATUS='):
                return int(line[7:])
        raise AssertionError(out)

    def alive(self):
        assert b'ALIVE\r\n' in self.send('print -r -- ALIVE'), 'shell stopped accepting commands'

    def init(self):
        self.send(f'eval "$(\'{binary}\' init zsh)"', timeout=10)

    def events(self):
        result = subprocess.run([binary, 'events', '--json', '--limit', '1000'], env=self.env,
                                check=True, capture_output=True)
        return json.loads(result.stdout)

    def close(self):
        try:
            os.write(self.fd, b'exit\n')
        except OSError:
            pass
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if os.waitpid(self.pid, os.WNOHANG)[0]:
                break
            time.sleep(0.05)
        else:
            os.kill(self.pid, signal.SIGKILL)
            os.waitpid(self.pid, 0)
        os.close(self.fd)


def repo(shell, name):
    path = shell.folder / name
    path.mkdir()
    for args in (['init', '-q', '-b', 'main'], ['commit', '-q', '--allow-empty', '-m', 'init']):
        subprocess.run(['git', *args], cwd=path, env=shell.env, check=True, capture_output=True)
    return path


def db_path(shell):
    return shell.folder / 'data' / 'nivra.db'


def commands(events):
    return [e['command'] for e in events]


# --- scenarios -------------------------------------------------------------

def failure_ctrl_c_and_missing_binary(sh):
    sh.init()
    assert sh.status_of('false') == 1
    os.write(sh.fd, b'sleep 30\n')
    time.sleep(0.3)
    os.write(sh.fd, b'\x03')
    sh.until_prompt()
    events = sh.events()
    assert any(e['command'] == 'false' and e['exit_code'] == 1 for e in events), events
    assert any(e['command'] == 'sleep 30' and e['exit_code'] == 130 for e in events), events
    sh.send('_NIVRA_BIN=/not/installed/nivra')
    assert sh.status_of('false') == 1
    sh.alive()


def leading_space_is_not_recorded(sh):
    sh.init()
    sh.send(' print -r -- secret-marker')
    sh.send('print -r -- visible-marker')
    recorded = commands(sh.events())
    assert 'print -r -- visible-marker' in recorded, recorded
    assert not any('secret-marker' in c for c in recorded), recorded


def repeated_init_records_once(sh):
    sh.init()
    sh.init()
    sh.send('print -r -- once-marker')
    assert commands(sh.events()).count('print -r -- once-marker') == 1


def explicit_run_is_not_double_recorded(sh):
    sh.init()
    assert sh.status_of(f"'{binary}' run -- false") == 1
    matching = [e for e in sh.events() if e['command'] == 'false']
    assert len(matching) == 1, matching
    assert matching[0]['session'].startswith('run-'), matching


def cd_across_repositories(sh):
    first, second = repo(sh, 'first'), repo(sh, 'second')
    sh.init()
    sh.send(f'cd {first}')
    sh.send('print -r -- in-first')
    sh.send(f'cd {second}')
    sh.send('print -r -- in-second')
    by_command = {e['command']: e for e in sh.events()}
    assert Path(by_command['print -r -- in-first']['before']['repo']).resolve() == first.resolve()
    assert Path(by_command['print -r -- in-second']['before']['repo']).resolve() == second.resolve()


def nested_shell_has_separate_session(sh):
    sh.init()
    sh.send('print -r -- outer-before')
    sh.send("zsh -f -i -o NO_ZLE -o NO_PROMPT_SP")
    sh.init()
    assert sh.status_of('false') == 1
    sh.send('print -r -- inner-marker')
    sh.send('exit')
    sh.send('print -r -- outer-after')
    by_command = {e['command']: e for e in sh.events()}
    outer = by_command['print -r -- outer-before']['session']
    assert by_command['print -r -- outer-after']['session'] == outer
    assert by_command['print -r -- inner-marker']['session'] != outer
    sh.alive()


def locked_database_fails_open(sh):
    sh.init()
    sh.send('print -r -- create-db')
    lock = sqlite3.connect(db_path(sh), timeout=0)
    lock.execute('BEGIN EXCLUSIVE')
    try:
        start = time.monotonic()
        assert sh.status_of('false') == 1
        elapsed = time.monotonic() - start
        assert elapsed < 2, f'locked database stalled the prompt for {elapsed:.2f}s'
        sh.alive()
    finally:
        lock.rollback()
        lock.close()
    sh.send('print -r -- after-unlock')
    recorded = commands(sh.events())
    # Proves the lock was effective: the locked command was dropped, not queued.
    assert 'false' not in recorded, recorded
    assert 'print -r -- after-unlock' in recorded, recorded


def read_only_storage_fails_open(sh):
    sh.init()
    sh.send('print -r -- create-db')
    data = sh.folder / 'data'
    os.chmod(db_path(sh), 0o400)
    os.chmod(data, 0o500)
    try:
        assert sh.status_of('false') == 1
        sh.alive()
    finally:
        os.chmod(data, 0o700)
        os.chmod(db_path(sh), 0o600)


def corrupt_database_fails_open(sh):
    sh.init()
    sh.send('print -r -- create-db')
    db_path(sh).write_bytes(b'this is not a sqlite database' * 100)
    assert sh.status_of('false') == 1
    assert sh.status_of('true') == 0
    sh.alive()


def missing_git_fails_open(sh):
    sh.init()
    # zsh builtins keep working; the absolute nivra path does not need PATH.
    sh.send('PATH=/nonexistent')
    assert sh.status_of('false') == 1
    sh.alive()
    by_command = {e['command']: e for e in sh.events()}
    assert by_command['false']['before']['repo'] is None, by_command['false']


def broken_repository_fails_open(sh):
    broken = sh.folder / 'broken'
    broken.mkdir()
    (broken / '.git').write_text('gitdir: /nonexistent/elsewhere\n')
    sh.init()
    sh.send(f'cd {broken}')
    assert sh.status_of('false') == 1
    sh.alive()


SCENARIOS = [
    failure_ctrl_c_and_missing_binary, leading_space_is_not_recorded, repeated_init_records_once,
    explicit_run_is_not_double_recorded, cd_across_repositories, nested_shell_has_separate_session,
    locked_database_fails_open, read_only_storage_fails_open, corrupt_database_fails_open,
    missing_git_fails_open, broken_repository_fails_open,
]


def main():
    selected = [s for s in SCENARIOS if len(sys.argv) < 2 or s.__name__ in sys.argv[1:]]
    failures = 0
    for scenario in selected:
        with tempfile.TemporaryDirectory(prefix='nivra-zsh-') as folder:
            shell = Shell(folder)
            try:
                scenario(shell)
                print(f'PASS  {scenario.__name__}')
            except AssertionError as error:
                failures += 1
                print(f'FAIL  {scenario.__name__}: {error}')
            finally:
                shell.close()
    print(f'{len(selected) - failures}/{len(selected)} interactive zsh scenarios passed')
    sys.exit(1 if failures else 0)


if __name__ == '__main__':
    main()
