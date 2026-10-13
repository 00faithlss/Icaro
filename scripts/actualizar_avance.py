#!/usr/bin/env python3
"""Actualiza la barra del README a partir de las casillas de su roadmap."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def main():
    readme = ROOT / "README.md"
    text = readme.read_text(encoding="utf-8")
    roadmap = text.split("\n## Roadmap\n", 1)[1].split("\n## ", 1)[0]
    tasks = re.findall(r"^\s*- \[([ xX])\] .+$", roadmap, re.MULTILINE)
    if not tasks:
        raise SystemExit("El roadmap no contiene casillas.")
    done = sum(task.lower() == "x" for task in tasks)
    total = len(tasks)
    percent = round(100 * done / total)
    fill = f"{480 * done / total:.2f}"
    label = f"{done} de {total} tareas completadas"
    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="520" height="96" viewBox="0 0 520 96" role="img" aria-labelledby="title desc">
  <title id="title">Avance del roadmap: {percent}%</title>
  <desc id="desc">{label}. Cada casilla tiene el mismo peso.</desc>
  <rect width="520" height="96" fill="#171717"/>
  <g font-family="Arial, sans-serif" fill="#eeeeea">
    <text x="20" y="29" font-size="13" font-weight="bold">AVANCE DEL ROADMAP</text>
    <text x="500" y="30" text-anchor="end" font-size="21" font-weight="bold">{percent}%</text>
    <rect x="20" y="43" width="480" height="12" fill="#3a3a38"/>
    <rect x="20" y="43" width="{fill}" height="12" fill="#eeeeea"/>
    <text x="20" y="78" font-size="12" fill="#bcbcb6">{label}</text>
  </g>
</svg>
'''
    block = f'''<!-- avance:inicio -->
[![Avance del roadmap: {percent}% — {label}](docs/assets/progreso.svg)](#roadmap)

**{percent}% del roadmap implementado · {done}/{total} tareas.** Cada casilla tiene el mismo peso.
<!-- avance:fin -->'''
    updated, count = re.subn(
        r"<!-- avance:inicio -->.*?<!-- avance:fin -->",
        lambda _: block, text, flags=re.DOTALL,
    )
    if count != 1:
        raise SystemExit("Se esperaba un único bloque de avance en el README.")
    target = ROOT / "docs" / "assets" / "progreso.svg"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(svg, encoding="utf-8")
    readme.write_text(updated, encoding="utf-8")
    print(f"Avance: {done}/{total} tareas ({percent}%).")


if __name__ == "__main__":
    main()
