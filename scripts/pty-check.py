#!/usr/bin/env python3
"""A pty harness for the picker: items on a pipe, keys from a real terminal,
value on a pipe. That is the shape dmenu callers use, so it is the shape the
acceptance check drives.

usage: pty-check.py <binary> <spec> <keys-literal> <items> [flags...]

keys-literal understands \x1b (escape), \r (enter), \t (tab), \x7f (backspace).
PTY07_ITEMS_FILE=<path> takes items from a file instead of the argument — that
is how the 60 000-line find run was made.

It answers the cursor-position query ratatui makes for an inline viewport,
because a pty made by forkpty has no terminal emulator behind it to reply.
"""
import os, struct, sys, time, fcntl, termios, select

def main():
    binary, spec, keys, items = sys.argv[1:5]
    flags = sys.argv[5:]
    keys = keys.encode().decode("unicode_escape").encode("latin-1")
    items = items.encode().decode("unicode_escape").encode()

    work = "/tmp/pty07"
    os.makedirs(work, exist_ok=True)
    source = os.environ.get("PTY07_ITEMS_FILE")
    if source:
        with open(source, "rb") as f:
            items = f.read()
    with open(f"{work}/items", "wb") as f:
        f.write(items)
    value_path = f"{work}/value"
    if os.path.exists(value_path):
        os.unlink(value_path)

    pid, master = os.forkpty()
    if pid == 0:
        inp = os.open(f"{work}/items", os.O_RDONLY)
        out = os.open(value_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
        os.dup2(inp, 0)
        os.dup2(out, 1)
        os.execv(binary, [binary, "--spec", spec, *flags])
        os._exit(127)

    fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack("HHHH", 16, 60, 0, 0))
    screen = b""
    started = time.time()
    sent = False
    exited = False
    status = 0
    while time.time() - started < 10:
        if not sent and time.time() - started > 0.6:
            os.write(master, keys)
            sent = True
        ready, _, _ = select.select([master], [], [], 0.05)
        if ready:
            try:
                chunk = os.read(master, 65536)
            except OSError:
                chunk = b""
            if not chunk:
                break
            screen += chunk
            # Answer the cursor-position query ratatui makes for an inline
            # viewport; a real terminal answers this without being asked.
            if b"\x1b[6n" in chunk:
                os.write(master, b"\x1b[1;1R")
        done, status = os.waitpid(pid, os.WNOHANG)
        if done:
            exited = True
            break
    if not exited:
        os.kill(pid, 9)
        _, status = os.waitpid(pid, 0)

    code = os.WEXITSTATUS(status) if os.WIFEXITED(status) else -1
    value = b""
    if os.path.exists(value_path):
        with open(value_path, "rb") as f:
            value = f.read()
    with open(f"{work}/screen", "wb") as f:
        f.write(screen)
    print(f"exit={code} value={value!r} screen_bytes={len(screen)}")

main()
