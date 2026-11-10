"""Collects the findings of the test bots into a Markdown list.
usage: python collect.py <bot folder> <output.md>"""
import re
import sys
from collections import defaultdict
from pathlib import Path

bot = Path(sys.argv[1])
out = Path(sys.argv[2])
names = {}
appdata = Path.home() / "AppData" / "Roaming" / "pagan"
for f in appdata.glob("save_E_*.name"):
    names[f.stem.split("_")[-1]] = f.read_text(encoding="latin1").strip()

bugs = defaultdict(list)      # (kind, map, area) -> [(run, line)]
errors = defaultdict(list)    # message -> [run]
deaths = defaultdict(int)     # (map, area, fighting) -> count
runs = 0
ticks_note = []
for log in sorted(bot.glob("logs/*.log")):
    runs += 1
    run = log.stem
    text = log.read_text(encoding="utf-8", errors="replace")
    for line in text.splitlines():
        if line.startswith("BUG "):
            m = re.match(r"BUG (\S+) \| map (\d+) at (-?\d+),(-?\d+),(-?\d+)", line)
            if m:
                kind, mp, x, y = m.group(1), m.group(2), int(m.group(3)), int(m.group(4))
                bugs[(kind, mp, x // 1024, y // 1024)].append((run, line))
        elif line.startswith("EVENT avatar-died"):
            m = re.match(r"EVENT avatar-died \| map (\d+) at (-?\d+),(-?\d+),(-?\d+).*fighting near: (.*)", line)
            if m:
                fighting = "Kampf" if m.group(5) != "nobody" else "ohne Kampf"
                deaths[(m.group(1), int(m.group(2)) // 2048, int(m.group(3)) // 2048, fighting)] += 1
        elif line.startswith("usecode:") or "panicked" in line:
            msg = re.sub(r"process \d+", "process N", line)
            errors[msg].append(run)
        elif line.startswith("exit ") and line.strip() not in ("exit 0",):
            errors[f"Lauf endete mit {line.strip()} (124 = nach 300 s abgebrochen)"].append(run)


def start_of(run):
    slot = re.match(r"slot(\d+)_", run).group(1)
    return f"Spielstand {slot} \"{names.get(slot, '?')}\""


with out.open("w", encoding="utf-8") as f:
    f.write(f"# Fundliste der Testbots\n\n{runs} Läufe à bis zu 40000 Ticks.\n\n")
    f.write("## Regelverstöße\n\n")
    if not bugs:
        f.write("Keine.\n\n")
    by_kind = defaultdict(list)
    for key, hits in bugs.items():
        by_kind[key[0]].append((key, hits))
    for kind, groups in sorted(by_kind.items()):
        f.write(f"### {kind} ({len(groups)} Stellen)\n\n")
        for (k, mp, ax, ay), hits in sorted(groups, key=lambda g: -len(g[1])):
            run, line = hits[0]
            f.write(f"- Karte {mp}, Bereich {ax * 1024},{ay * 1024}: {len(hits)}× — Beispiel aus {start_of(run)} ({run}): `{line}`\n")
        f.write("\n")
    f.write("## Skriptfehler und Abstürze\n\n")
    if not errors:
        f.write("Keine.\n\n")
    for msg, rs in sorted(errors.items(), key=lambda e: -len(e[1])):
        f.write(f"- {len(rs)}× `{msg}` (z. B. {start_of(rs[0])}, {rs[0]})\n")
    f.write("\n## Tode des Avatars (zur Sichtung, meist kein Fehler)\n\n")
    for (mp, ax, ay, fighting), n in sorted(deaths.items(), key=lambda d: -d[1])[:25]:
        f.write(f"- Karte {mp}, Bereich {ax * 2048},{ay * 2048}, {fighting}: {n}×\n")
print(f"{runs} runs, {sum(len(h) for h in bugs.values())} findings in {len(bugs)} places, {len(errors)} error kinds")
