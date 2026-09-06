"""Where did the time go in this session's Ghost calls?

Pairs every tool_use with its tool_result in the session transcript, pulls the
server-side "ms" stamp Ghost puts on every response, and separates:
  - Ghost's own execution time per tool (median / p90 / max, count)
  - explicit ghost_wait sleeps the agent asked for inside ghost_run
  - the "Process not found" (window title miss) failure path and its cost
"""
import json, re, sys, statistics
from collections import defaultdict

path = sys.argv[1]
uses = {}          # id -> (name, input)
results = []       # (name, input, text)
for line in open(path, encoding="utf-8", errors="ignore"):
    try:
        rec = json.loads(line)
    except Exception:
        continue
    msg = rec.get("message") or {}
    content = msg.get("content")
    if not isinstance(content, list):
        continue
    for block in content:
        if not isinstance(block, dict):
            continue
        if block.get("type") == "tool_use":
            uses[block.get("id")] = (block.get("name", ""), block.get("input") or {})
        elif block.get("type") == "tool_result":
            tid = block.get("tool_use_id")
            if tid not in uses:
                continue
            c = block.get("content")
            if isinstance(c, list):
                text = " ".join(x.get("text", "") for x in c if isinstance(x, dict))
            else:
                text = str(c or "")
            results.append((uses[tid][0], uses[tid][1], text))

ghost = [(n, i, t) for (n, i, t) in results if n.startswith("mcp__ghost__")]
print(f"ghost calls with results: {len(ghost)}")

per_tool = defaultdict(list)
notfound = []
waits_in_runs = 0
run_total_ms = 0
run_count = 0
for name, inp, text in ghost:
    m = re.search(r'"ms":\s*(\d+)', text)
    ms = int(m.group(1)) if m else None
    short = name.replace("mcp__ghost__", "")
    if ms is not None:
        per_tool[short].append(ms)
    if "Process not found" in text or "no visible window matching" in text:
        notfound.append((short, ms))
    if short == "ghost_run":
        run_count += 1
        if ms is not None:
            run_total_ms += ms
        for step in inp.get("steps", []) or []:
            if isinstance(step, dict) and step.get("op") == "ghost_wait":
                waits_in_runs += int(step.get("ms") or 0)

def pct(v, p):
    v = sorted(v)
    return v[min(len(v) - 1, int(round(p * (len(v) - 1))))]

print("\nper tool (server-side ms): count median p90 max")
for tool, v in sorted(per_tool.items(), key=lambda kv: -sum(kv[1])):
    print(f"  {tool:28s} n={len(v):3d}  med={statistics.median(v):7.0f}  p90={pct(v,0.9):7.0f}  max={max(v):7.0f}  total={sum(v)/1000:7.1f}s")

print(f"\nghost_run: {run_count} calls, {run_total_ms/1000:.1f}s total, of which explicit ghost_wait sleeps = {waits_in_runs/1000:.1f}s "
      f"({(100*waits_in_runs/run_total_ms) if run_total_ms else 0:.0f}%)")
print(f"\nwindow-title misses ('Process not found'): {len(notfound)}; ms each: {[ms for _, ms in notfound]}")
if notfound:
    v = [ms for _, ms in notfound if ms is not None]
    if v:
        print(f"  cost of misses: total={sum(v)/1000:.1f}s median={statistics.median(v):.0f}ms max={max(v)}ms")

# shell tool: duration_ms inside payload vs outer ms
sh = [(i, t) for n, i, t in ghost if n.endswith("ghost_shell")]
d = [int(m.group(1)) for _, t in sh for m in [re.search(r'"duration_ms":\s*(\d+)', t)] if m]
if d:
    print(f"\nghost_shell: n={len(sh)} command durations med={statistics.median(d):.0f}ms p90={pct(d,0.9)}ms max={max(d)}ms (npx/cargo/docker dominate the tail)")
