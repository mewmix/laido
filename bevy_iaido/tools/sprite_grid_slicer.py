#!/usr/bin/env python3
"""
Slice sprite sheets into individual PNGs, remove white background,
and generate an HTML labeling preview.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Tuple

from PIL import Image


@dataclass
class TileRecord:
    sheet: str
    tile_index: int
    row: int
    col: int
    file: str
    label: str


def parse_grid(s: str) -> Tuple[int, int]:
    m = re.match(r"^\s*(\d+)\s*x\s*(\d+)\s*$", s.lower())
    if not m:
        raise ValueError(f"Invalid grid '{s}'. Use format NxM, e.g. 2x2.")
    cols = int(m.group(1))
    rows = int(m.group(2))
    if cols <= 0 or rows <= 0:
        raise ValueError("Grid values must be positive.")
    return cols, rows


def sanitize_label(label: str) -> str:
    label = label.strip().lower()
    label = re.sub(r"[^a-z0-9]+", "_", label)
    label = re.sub(r"_+", "_", label)
    return label.strip("_")


def open_image(path: Path) -> None:
    system = platform.system().lower()
    try:
        if "darwin" in system:
            subprocess.run(["open", str(path)], check=False)
        elif "linux" in system:
            subprocess.run(["xdg-open", str(path)], check=False)
        elif "windows" in system:
            os.startfile(str(path))  # type: ignore[attr-defined]
    except Exception:
        pass


def remove_white_background(img: Image.Image, threshold: int, chroma_threshold: int, mode: str) -> Image.Image:
    if img.mode != "RGBA":
        img = img.convert("RGBA")
    pixels = img.load()
    width, height = img.size
    for y in range(height):
        for x in range(width):
            r, g, b, a = pixels[x, y]
            if a == 0:
                continue
            max_c = max(r, g, b)
            min_c = min(r, g, b)
            if mode == "white":
                if r >= threshold and g >= threshold and b >= threshold and (max_c - min_c) <= chroma_threshold:
                    pixels[x, y] = (r, g, b, 0)
            else:
                if r <= threshold and g <= threshold and b <= threshold and (max_c - min_c) <= chroma_threshold:
                    pixels[x, y] = (r, g, b, 0)
    return img


def iter_input_paths(patterns: Iterable[str]) -> List[Path]:
    paths: List[Path] = []
    for pattern in patterns:
        paths.extend(sorted(Path().glob(pattern)))
    return [p for p in paths if p.is_file()]

def write_html(records: List[TileRecord], out_dir: Path, html_path: Path) -> None:
    html_records = []
    for record in records:
        html_records.append({
            "sheet": record.sheet,
            "tile_index": record.tile_index,
            "row": record.row,
            "col": record.col,
            "file": record.file,
            "label": record.label,
        })

    html = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Sprite Labeler</title>
  <style>
    :root {{
      color-scheme: light;
      font-family: "Iosevka", "SF Mono", "Menlo", monospace;
      background: #f5f1ea;
      color: #2a2320;
    }}
    body {{
      margin: 24px;
    }}
    header {{
      display: flex;
      gap: 12px;
      align-items: center;
      flex-wrap: wrap;
      margin-bottom: 18px;
    }}
    button {{
      background: #2a2320;
      color: #f5f1ea;
      border: 0;
      padding: 8px 12px;
      cursor: pointer;
    }}
    input[type="text"] {{
      width: 100%;
      padding: 6px 8px;
      border: 1px solid #b9b1a7;
      border-radius: 4px;
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
      gap: 16px;
    }}
    .card {{
      background: #fffaf2;
      border: 1px solid #d8d0c6;
      padding: 10px;
      border-radius: 6px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.08);
    }}
    .card img {{
      width: 100%;
      height: auto;
      display: block;
      background: repeating-linear-gradient(
        45deg,
        #eee 0,
        #eee 10px,
        #f7f7f7 10px,
        #f7f7f7 20px
      );
      margin-bottom: 8px;
    }}
    .meta {{
      font-size: 12px;
      margin-bottom: 6px;
      color: #6b5f57;
    }}
  </style>
</head>
<body>
  <header>
    <button id="download">Download labels.json</button>
    <label>
      Load labels.json
      <input id="load" type="file" accept="application/json" />
    </label>
    <span>Open this file directly; images are relative to: {out_dir.as_posix()}</span>
  </header>
  <div class="grid" id="grid"></div>

  <script>
    const records = {json.dumps(html_records)};
    const grid = document.getElementById("grid");

    function render() {{
      grid.innerHTML = "";
      records.forEach((rec, idx) => {{
        const card = document.createElement("div");
        card.className = "card";

        const img = document.createElement("img");
        img.src = rec.file;
        img.alt = rec.sheet + " " + rec.tile_index;

        const meta = document.createElement("div");
        meta.className = "meta";
        meta.textContent = `${{rec.sheet}} | tile ${{rec.tile_index}} (r${{rec.row}} c${{rec.col}})`;

        const input = document.createElement("input");
        input.type = "text";
        input.placeholder = "label";
        input.value = rec.label || "";
        input.addEventListener("input", (e) => {{
          records[idx].label = e.target.value;
        }});

        card.appendChild(img);
        card.appendChild(meta);
        card.appendChild(input);
        grid.appendChild(card);
      }});
    }}

    document.getElementById("download").addEventListener("click", () => {{
      const blob = new Blob([JSON.stringify(records, null, 2)], {{type: "application/json"}});
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "labels.json";
      document.body.appendChild(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
    }});

    document.getElementById("load").addEventListener("change", (e) => {{
      const file = e.target.files[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {{
        try {{
          const loaded = JSON.parse(reader.result);
          const byFile = new Map(loaded.map((r) => [r.file, r]));
          records.forEach((rec) => {{
            const match = byFile.get(rec.file);
            if (match && match.label) {{
              rec.label = match.label;
            }}
          }});
          render();
        }} catch (err) {{
          alert("Failed to load labels.json");
        }}
      }};
      reader.readAsText(file);
    }});

    render();
  </script>
</body>
</html>
"""
    html_path.write_text(html)


def apply_labels(labels_path: Path) -> int:
    if not labels_path.exists():
        print(f"labels.json not found: {labels_path}")
        return 1
    data = json.loads(labels_path.read_text())
    base_dir = labels_path.parent
    for rec in data:
        label = sanitize_label(rec.get("label", "")) or f"tile_{rec.get('tile_index', 0)}"
        file_path = base_dir / rec["file"]
        if not file_path.exists():
            print(f"Missing file: {file_path}")
            continue
        stem = Path(rec["file"]).stem.split("__")[0]
        new_name = f"{stem}__{label}.png"
        new_path = file_path.with_name(new_name)
        if new_path != file_path:
            file_path.replace(new_path)
            rec["file"] = str(Path(rec["file"]).with_name(new_name))
    labels_path.write_text(json.dumps(data, indent=2))
    print(f"Renamed files and updated labels: {labels_path}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Slice sprite sheets, remove white background, and generate an HTML labeling preview."
    )
    parser.add_argument(
        "--input",
        nargs="+",
        default=["assets/atlas/*.png"],
        help="Glob pattern(s) for input sheets. Default: assets/atlas/*.png",
    )
    parser.add_argument(
        "--out-dir",
        default="assets/atlas/slices",
        help="Output directory for sliced tiles.",
    )
    parser.add_argument(
        "--grid",
        default="2x2",
        help="Grid size (NxM). Default: 2x2",
    )
    parser.add_argument(
        "--white-threshold",
        type=int,
        default=245,
        help="RGB threshold for white background removal. Default: 245",
    )
    parser.add_argument(
        "--black-threshold",
        type=int,
        default=12,
        help="RGB threshold for black background removal. Default: 12",
    )
    parser.add_argument(
        "--chroma-threshold",
        type=int,
        default=10,
        help="Max RGB spread to treat as neutral white. Default: 10",
    )
    parser.add_argument(
        "--bg",
        choices=["white", "black"],
        default="white",
        help="Background mode to remove. Default: white",
    )
    parser.add_argument(
        "--open",
        action="store_true",
        help="Open each tile image for preview as they are generated.",
    )
    parser.add_argument(
        "--apply-labels",
        default=None,
        help="Apply labels from labels.json and rename files.",
    )
    parser.add_argument(
        "--labels-json",
        default=None,
        help="Where to write labels JSON. Default: <out-dir>/labels.json",
    )
    parser.add_argument(
        "--html",
        default=None,
        help="Where to write HTML preview. Default: <out-dir>/index.html",
    )
    args = parser.parse_args()

    if args.apply_labels:
        return apply_labels(Path(args.apply_labels))

    cols, rows = parse_grid(args.grid)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    labels_json = Path(args.labels_json) if args.labels_json else out_dir / "labels.json"
    html_path = Path(args.html) if args.html else out_dir / "index.html"

    input_paths = iter_input_paths(args.input)
    if not input_paths:
        print("No input sheets found.")
        return 1

    records: List[TileRecord] = []
    for sheet_path in input_paths:
        sheet = sheet_path.name
        print(f"Processing: {sheet}")
        img = Image.open(sheet_path)
        width, height = img.size
        if width % cols != 0 or height % rows != 0:
            print(f"  Skipping {sheet}: size {width}x{height} not divisible by {cols}x{rows}")
            continue

        tile_w = width // cols
        tile_h = height // rows

        base = sheet_path.stem
        for row in range(rows):
            for col in range(cols):
                idx = row * cols + col
                box = (col * tile_w, row * tile_h, (col + 1) * tile_w, (row + 1) * tile_h)
                tile = img.crop(box)
                threshold = args.white_threshold if args.bg == "white" else args.black_threshold
                tile = remove_white_background(tile, threshold, args.chroma_threshold, args.bg)
                final_label = f"tile_{idx}"
                final_name = f"{base}__{final_label}.png"
                final_path = out_dir / final_name
                tile.save(final_path)
                if args.open:
                    open_image(final_path)

                records.append(
                    TileRecord(
                        sheet=sheet,
                        tile_index=idx,
                        row=row,
                        col=col,
                        file=str(final_path.relative_to(out_dir)),
                        label="",
                    )
                )

    payload = [record.__dict__ for record in records]
    labels_json.write_text(json.dumps(payload, indent=2))
    write_html(records, out_dir, html_path)
    print(f"Wrote labels: {labels_json}")
    print(f"Wrote preview: {html_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
