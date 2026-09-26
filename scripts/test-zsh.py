#!/usr/bin/env python3
"""Exercise actual interactive zsh hooks through a PTY, including Ctrl-C."""
import json
import os
from pathlib import Path
import pty
import select
import signal
import subprocess
import tempfile
import time

binary = str(Path(os.environ.get('NIVRA_BIN', 'target/debug/nivra')).resolve())
with tempfile.TemporaryDirectory(prefix='nivra-zsh-') as folder:
    env = dict(os.environ, NIVRA_DATA_DIR=folder + '/data', TERM='dumb', PS1='NIVRA_TEST> ', RPS1='')
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(folder)
        os.execvpe('zsh', ['zsh', '-f', '-i', '-o', 'NO_ZLE', '-o', 'NO_PROMPT_SP'], env)
    def until_prompt():
        output = b''
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if select.select([fd], [], [], 0.1)[0]:
                output += os.read(fd, 65536)
                if output.endswith(b'NIVRA_TEST> '):
                    return output
        raise AssertionError('zsh prompt timeout: ' + repr(output))
    def send(text):
        os.write(fd, text.encode() + b'\n')
        return until_prompt()
    try:
        until_prompt()
        send(f'eval "$(\'{binary}\' init zsh)"')
        send('false')
        status = send('print -r -- EXIT_STATUS=$?')
        assert b'EXIT_STATUS=1\r\n' in status, status
        os.write(fd, b'sleep 30\n')
        time.sleep(0.3)
        os.write(fd, b'\x03')
        until_prompt()
        send("_NIVRA_BIN=/not/installed/nivra")
        survived = send('print -r -- STILL_ALIVE')
        assert b'STILL_ALIVE\r\n' in survived, survived
        result = subprocess.run([binary, 'events', '--json'], env=env, check=True, capture_output=True)
        events = json.loads(result.stdout)
        assert any(e['command'] == 'false' and e['exit_code'] == 1 for e in events), events
        assert any(e['command'] == 'sleep 30' and e['exit_code'] == 130 for e in events), events
        print('PASS: interactive zsh captures failure, preserves status, handles Ctrl-C, and survives a missing binary')
    finally:
        os.write(fd, b'exit\n')
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if os.waitpid(pid, os.WNOHANG)[0]:
                break
            time.sleep(0.05)
        else:
            os.kill(pid, signal.SIGKILL)
            os.waitpid(pid, 0)
        os.close(fd)
