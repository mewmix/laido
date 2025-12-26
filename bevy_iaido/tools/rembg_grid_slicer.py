#!/usr/bin/env python3
"""
Slice sprite sheets into a grid, removing background via rembg.
"""

from __future__ import annotations

import argparse
import io
from pathlib import Path
from typing import Iterable, List, Tuple

from PIL import Image
from rembg import new_session, remove


def parse_grid(s: str) -> Tuple[int, int]:
    parts = s.lower().split("x")
    if len(parts) != 2:
        raise ValueError(f"Invalid grid '{s}'. Use format NxM, e.g. 2x2.")
    cols = int(parts[0].strip())
    rows = int(parts[1].strip())
    if cols <= 0 or rows <= 0:
        raise ValueError("Grid values must be positive.")
    return cols, rows


def parse_tile_size(s: str) -> Tuple[int, int]:
    if "x" in s.lower():
        parts = s.lower().split("x")
        if len(parts) != 2:
            raise ValueError(f"Invalid tile size '{s}'. Use format WxH, e.g. 64x64.")
        w = int(parts[0].strip())
        h = int(parts[1].strip())
    else:
        w = h = int(s.strip())
    if w <= 0 or h <= 0:
        raise ValueError("Tile size must be positive.")
    return w, h


def iter_input_paths(patterns: Iterable[str]) -> List[Path]:
    paths: List[Path] = []
    for pattern in patterns:
        paths.extend(sorted(Path().glob(pattern)))
    return [p for p in paths if p.is_file()]


def process_sheet(
    img_path: Path,
    output_dir: Path,
    cols: int,
    rows: int,
    session,
    tile_size: Tuple[int, int] | None,
) -> None:
    print(f"Processing {img_path.name}...")
    input_data = img_path.read_bytes()
    no_bg_data = remove(input_data, session=session)
    img = Image.open(io.BytesIO(no_bg_data)).convert("RGBA")
    width, height = img.size

    if tile_size:
        sprite_w, sprite_h = tile_size
        cell_w = width / cols
        cell_h = height / rows
    else:
        if width % cols != 0 or height % rows != 0:
            print(f"  Skipping {img_path.name}: size {width}x{height} not divisible by {cols}x{rows}")
            return
        sprite_w = width // cols
        sprite_h = height // rows
        cell_w = sprite_w
        cell_h = sprite_h

    for r in range(rows):
        for c in range(cols):
            center_x = (c + 0.5) * cell_w
            center_y = (r + 0.5) * cell_h
            left = int(round(center_x - sprite_w / 2))
            upper = int(round(center_y - sprite_h / 2))
            right = left + sprite_w
            lower = upper + sprite_h
            if left < 0:
                right -= left
                left = 0
            if upper < 0:
                lower -= upper
                upper = 0
            if right > width:
                left -= right - width
                right = width
            if lower > height:
                upper -= lower - height
                lower = height
            sprite = img.crop((left, upper, right, lower))
            sprite_filename = f"{img_path.stem}_frame_{r}_{c}.png"
            sprite.save(output_dir / sprite_filename)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Slice sprite sheets into a grid with background removal via rembg."
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
        "--tile-size",
        default=None,
        help="Optional fixed tile size (WxH or single int). Uses centered crop per cell.",
    )
    args = parser.parse_args()

    cols, rows = parse_grid(args.grid)
    tile_size = parse_tile_size(args.tile_size) if args.tile_size else None
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    input_paths = iter_input_paths(args.input)
    if not input_paths:
        print("No input sheets found.")
        return 1

    session = new_session("u2net", providers=["CPUExecutionProvider"])
    for img_path in input_paths:
        process_sheet(img_path, out_dir, cols, rows, session, tile_size)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
