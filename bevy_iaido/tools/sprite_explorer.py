#!/usr/bin/env python3
"""
Quick sprite sheet explorer for labeling tiles and exporting by index.
Requires: Pillow (pip install pillow)
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import tkinter as tk
from dataclasses import dataclass
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

try:
    from PIL import Image, ImageTk
except ImportError:  # pragma: no cover - runtime guard only
    print("Missing Pillow. Install with: pip install pillow", file=sys.stderr)
    raise


@dataclass
class SheetConfig:
    tile_w: int
    tile_h: int
    margin: int
    spacing: int
    scale: float

    @property
    def tile_w_total(self) -> int:
        return self.tile_w + self.spacing

    @property
    def tile_h_total(self) -> int:
        return self.tile_h + self.spacing


class SpriteExplorer(tk.Tk):
    def __init__(self, image_paths: list[Path], cfg: SheetConfig, out_dir: Path) -> None:
        super().__init__()
        self.title("Sprite Explorer")
        self.image_paths = image_paths
        self.current_image_idx = 0
        self.cfg = cfg
        self.out_dir = out_dir
        
        # Per-image labels: path -> {index: label}
        self.all_labels: dict[Path, dict[int, str]] = {}

        self.canvas = tk.Canvas(self, bg="#202020", highlightthickness=0)
        self.side = ttk.Frame(self)

        self._selected_index: int | None = None
        self._status_var = tk.StringVar(value="Click a tile to select it.")
        self._index_var = tk.StringVar(value="Index: -")
        self._label_var = tk.StringVar(value="")
        self._only_labeled_var = tk.BooleanVar(value=False)
        
        # Grid Config Vars
        self._tile_w_var = tk.IntVar(value=cfg.tile_w)
        self._tile_h_var = tk.IntVar(value=cfg.tile_h)
        self._margin_var = tk.IntVar(value=cfg.margin)
        self._spacing_var = tk.IntVar(value=cfg.spacing)

        self._build_ui()
        
        if self.image_paths:
            self._load_current_image()
        else:
            self._status_var.set("No images loaded.")

    @property
    def current_labels(self) -> dict[int, str]:
        path = self.image_paths[self.current_image_idx]
        if path not in self.all_labels:
            self.all_labels[path] = {}
            self._try_load_labels_for(path)
        return self.all_labels[path]

    def _try_load_labels_for(self, image_path: Path) -> None:
        # Look for a .json file with the same name
        json_path = image_path.with_suffix(".json")
        if json_path.exists():
            try:
                data = json.loads(json_path.read_text())
                labels = data.get("labels", [])
                for entry in labels:
                    self.current_labels[int(entry["index"])] = str(entry["label"])
            except Exception as e:
                print(f"Failed to load labels from {json_path}: {e}")

    def _load_current_image(self) -> None:
        self._selected_index = None
        path = self.image_paths[self.current_image_idx]
        print(f"DEBUG: Loading image {path}")
        self.title(f"Sprite Explorer - {path.name} ({self.current_image_idx + 1}/{len(self.image_paths)})")
        
        try:
            self.image = Image.open(path).convert("RGBA")
        except Exception as e:
            messagebox.showerror("Error", f"Failed to open image {path}: {e}")
            return

        self._update_grid_from_ui()
        self._refresh_label_list()
        self._draw_sheet()

    def _update_grid_from_ui(self, *args) -> None:
        self.cfg.tile_w = self._tile_w_var.get()
        self.cfg.tile_h = self._tile_h_var.get()
        self.cfg.margin = self._margin_var.get()
        self.cfg.spacing = self._spacing_var.get()
        self.columns, self.rows = self._compute_grid()
        self._draw_sheet()

    def _compute_grid(self) -> tuple[int, int]:
        if self.cfg.tile_w_total <= 0 or self.cfg.tile_h_total <= 0:
            return 0, 0
        w, h = self.image.size
        cols = (w - 2 * self.cfg.margin + self.cfg.spacing) // self.cfg.tile_w_total
        rows = (h - 2 * self.cfg.margin + self.cfg.spacing) // self.cfg.tile_h_total
        return max(cols, 0), max(rows, 0)

    def _build_ui(self) -> None:
        print("DEBUG: Building UI...")
        self.geometry("1200x800")
        self.minsize(900, 600)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        self.canvas.grid(row=0, column=0, sticky="nsew")
        
        # Scrollbars for canvas
        h_scroll = ttk.Scrollbar(self, orient="horizontal", command=self.canvas.xview)
        h_scroll.grid(row=1, column=0, sticky="ew")
        v_scroll = ttk.Scrollbar(self, orient="vertical", command=self.canvas.yview)
        v_scroll.grid(row=0, column=1, sticky="ns")
        self.canvas.configure(xscrollcommand=h_scroll.set, yscrollcommand=v_scroll.set)

        self.side.grid(row=0, column=2, rowspan=2, sticky="ns")
        self.side.columnconfigure(0, weight=1)

        # Navigation
        nav_frame = ttk.LabelFrame(self.side, text="Navigation")
        nav_frame.grid(row=0, column=0, padx=10, pady=5, sticky="ew")
        ttk.Button(nav_frame, text="<< Prev", command=self._prev_image).grid(row=0, column=0, sticky="ew")
        ttk.Button(nav_frame, text="Next >>", command=self._next_image).grid(row=0, column=1, sticky="ew")
        nav_frame.columnconfigure(0, weight=1)
        nav_frame.columnconfigure(1, weight=1)

        # Grid Config
        grid_frame = ttk.LabelFrame(self.side, text="Grid Configuration")
        grid_frame.grid(row=1, column=0, padx=10, pady=5, sticky="ew")
        
        ttk.Label(grid_frame, text="Tile W:").grid(row=0, column=0, sticky="e")
        ttk.Entry(grid_frame, textvariable=self._tile_w_var, width=5).grid(row=0, column=1, sticky="w")
        ttk.Label(grid_frame, text="Tile H:").grid(row=0, column=2, sticky="e")
        ttk.Entry(grid_frame, textvariable=self._tile_h_var, width=5).grid(row=0, column=3, sticky="w")
        
        ttk.Label(grid_frame, text="Margin:").grid(row=1, column=0, sticky="e")
        ttk.Entry(grid_frame, textvariable=self._margin_var, width=5).grid(row=1, column=1, sticky="w")
        ttk.Label(grid_frame, text="Space:").grid(row=1, column=2, sticky="e")
        ttk.Entry(grid_frame, textvariable=self._spacing_var, width=5).grid(row=1, column=3, sticky="w")
        
        ttk.Label(grid_frame, text="UI Scale:").grid(row=2, column=0, sticky="e")
        self._scale_slider = ttk.Scale(grid_frame, from_=0.1, to=2.0, value=self.cfg.scale, orient="horizontal", command=self._on_scale_change)
        self._scale_slider.grid(row=2, column=1, columnspan=3, sticky="ew", padx=5)

        ttk.Button(grid_frame, text="Update Grid", command=self._update_grid_from_ui).grid(row=3, column=0, columnspan=4, sticky="ew", pady=5)

        # Selection
        info = ttk.LabelFrame(self.side, text="Selection")
        info.grid(row=2, column=0, padx=10, pady=5, sticky="ew")
        ttk.Label(info, textvariable=self._index_var).grid(row=0, column=0, sticky="w")
        ttk.Label(info, text="Label").grid(row=1, column=0, sticky="w", pady=(5, 0))
        entry = ttk.Entry(info, textvariable=self._label_var)
        entry.grid(row=2, column=0, sticky="ew", pady=(2, 0))
        entry.bind("<Return>", lambda _evt: self._set_label())

        btns = ttk.Frame(info)
        btns.grid(row=3, column=0, sticky="ew", pady=(5, 0))
        ttk.Button(btns, text="Set Label", command=self._set_label).grid(
            row=0, column=0, sticky="ew"
        )
        ttk.Button(btns, text="Remove", command=self._remove_label).grid(
            row=0, column=1, sticky="ew", padx=(5, 0)
        )
        btns.columnconfigure(0, weight=1)
        btns.columnconfigure(1, weight=1)

        # Labels List
        labels_frame = ttk.LabelFrame(self.side, text="Labels")
        labels_frame.grid(row=3, column=0, padx=10, pady=5, sticky="nsew")
        labels_frame.rowconfigure(0, weight=1)
        labels_frame.columnconfigure(0, weight=1)

        self.labels_list = tk.Listbox(labels_frame, height=10)
        self.labels_list.grid(row=0, column=0, sticky="nsew")
        self.labels_list.bind("<<ListboxSelect>>", self._on_label_select)
        scrollbar = ttk.Scrollbar(labels_frame, orient="vertical", command=self.labels_list.yview)
        scrollbar.grid(row=0, column=1, sticky="ns")
        self.labels_list.configure(yscrollcommand=scrollbar.set)

        # Export
        export_frame = ttk.LabelFrame(self.side, text="Export")
        export_frame.grid(row=4, column=0, padx=10, pady=5, sticky="ew")
        ttk.Checkbutton(
            export_frame, text="Only labeled", variable=self._only_labeled_var
        ).grid(row=0, column=0, sticky="w")
        ttk.Button(export_frame, text="Export Selected", command=self._export_selected).grid(
            row=1, column=0, sticky="ew", pady=(2, 0)
        )
        ttk.Button(export_frame, text="Export All", command=self._export_all).grid(
            row=2, column=0, sticky="ew", pady=(2, 0)
        )
        ttk.Button(export_frame, text="Save Labels JSON", command=self._save_labels).grid(
            row=3, column=0, sticky="ew", pady=(2, 0)
        )

        status = ttk.Label(self.side, textvariable=self._status_var, wraplength=240)
        status.grid(row=5, column=0, padx=10, pady=(0, 10), sticky="ew")

        self.canvas.bind("<Button-1>", self._on_click)

    def _on_scale_change(self, val: str) -> None:
        self.cfg.scale = float(val)
        self._draw_sheet()


    def _next_image(self) -> None:
        if self.current_image_idx < len(self.image_paths) - 1:
            self.current_image_idx += 1
            self._load_current_image()

    def _prev_image(self) -> None:
        if self.current_image_idx > 0:
            self.current_image_idx -= 1
            self._load_current_image()

    def _draw_sheet(self) -> None:
        scale = self.cfg.scale
        w, h = self.image.size
        print(f"DEBUG: Drawing sheet. Image size: {w}x{h}, Scale: {scale}, Grid: {self.columns}x{self.rows}")
        scaled = self.image.resize((int(w * scale), int(h * scale)), Image.NEAREST)
        self.tk_img = ImageTk.PhotoImage(scaled)
        
        self.canvas.configure(scrollregion=(0, 0, scaled.size[0], scaled.size[1]))
        self.canvas.delete("all")
        self.canvas.create_image(0, 0, anchor="nw", image=self.tk_img)

        for row in range(self.rows):
            for col in range(self.columns):
                x0 = (self.cfg.margin + col * self.cfg.tile_w_total) * scale
                y0 = (self.cfg.margin + row * self.cfg.tile_h_total) * scale
                x1 = x0 + self.cfg.tile_w * scale
                y1 = y0 + self.cfg.tile_h * scale
                self.canvas.create_rectangle(
                    x0, y0, x1, y1, outline="#4a4a4a", width=1
                )

    def _on_click(self, event: tk.Event) -> None:
        # Account for scrolling
        canvas_x = self.canvas.canvasx(event.x)
        canvas_y = self.canvas.canvasy(event.y)
        index = self._index_from_canvas(canvas_x, canvas_y)
        if index is None:
            self._status_var.set("Clicked outside tile bounds.")
            return
        self._select_index(index)

    def _index_from_canvas(self, x: float, y: float) -> int | None:
        scale = self.cfg.scale
        sx = x / scale
        sy = y / scale
        if sx < self.cfg.margin or sy < self.cfg.margin:
            return None
        sx -= self.cfg.margin
        sy -= self.cfg.margin
        col = int(sx // self.cfg.tile_w_total)
        row = int(sy // self.cfg.tile_h_total)
        if col < 0 or row < 0 or col >= self.columns or row >= self.rows:
            return None
        local_x = sx % self.cfg.tile_w_total
        local_y = sy % self.cfg.tile_h_total
        if local_x >= self.cfg.tile_w or local_y >= self.cfg.tile_h:
            return None
        return row * self.columns + col

    def _select_index(self, index: int) -> None:
        self._selected_index = index
        label = self.current_labels.get(index, "")
        self._index_var.set(f"Index: {index}  (r{index // self.columns}, c{index % self.columns})")
        self._label_var.set(label)
        self._status_var.set("Ready.")

    def _set_label(self) -> None:
        if self._selected_index is None:
            self._status_var.set("Select a tile before labeling.")
            return
        label = self._label_var.get().strip()
        if not label:
            self._status_var.set("Label is empty.")
            return
        self.current_labels[self._selected_index] = label
        self._refresh_label_list()
        self._status_var.set(f"Labeled index {self._selected_index}.")

    def _remove_label(self) -> None:
        if self._selected_index is None:
            self._status_var.set("Select a tile first.")
            return
        if self._selected_index in self.current_labels:
            del self.current_labels[self._selected_index]
            self._refresh_label_list()
            self._status_var.set(f"Removed label from index {self._selected_index}.")
        else:
            self._status_var.set("No label to remove.")

    def _refresh_label_list(self) -> None:
        self.labels_list.delete(0, tk.END)
        for idx in sorted(self.current_labels.keys()):
            self.labels_list.insert(tk.END, f"{idx}: {self.current_labels[idx]}")

    def _on_label_select(self, _evt: tk.Event) -> None:
        selection = self.labels_list.curselection()
        if not selection:
            return
        item = self.labels_list.get(selection[0])
        try:
            index = int(item.split(":", 1)[0])
        except ValueError:
            return
        self._select_index(index)

    def _save_labels(self) -> None:
        # Default to same name as image but .json
        default_name = self.image_paths[self.current_image_idx].with_suffix(".json").name
        path_str = filedialog.asksaveasfilename(
            title="Save labels JSON",
            initialfile=default_name,
            defaultextension=".json",
            filetypes=[("JSON", "*.json")],
        )
        if not path_str:
            return
        path = Path(path_str)
        
        payload = {
            "image": str(self.image_paths[self.current_image_idx]),
            "tile_w": self.cfg.tile_w,
            "tile_h": self.cfg.tile_h,
            "margin": self.cfg.margin,
            "spacing": self.cfg.spacing,
            "columns": self.columns,
            "rows": self.rows,
            "labels": [
                {"index": idx, "label": self.current_labels[idx]} for idx in sorted(self.current_labels.keys())
            ],
        }
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload, indent=2))
        self._status_var.set(f"Saved labels to {path}")

    def _export_selected(self) -> None:
        if self._selected_index is None:
            self._status_var.set("Select a tile to export.")
            return
        self._export_indexes([self._selected_index])

    def _export_all(self) -> None:
        indexes = list(range(self.columns * self.rows))
        if self._only_labeled_var.get():
            indexes = [idx for idx in indexes if idx in self.current_labels]
        self._export_indexes(indexes)

    def _export_indexes(self, indexes: list[int]) -> None:
        if not indexes:
            self._status_var.set("No tiles to export.")
            return
        
        # Create subfolder for this image
        base_name = self.image_paths[self.current_image_idx].stem
        target_dir = self.out_dir / base_name
        target_dir.mkdir(parents=True, exist_ok=True)
        
        for idx in indexes:
            row = idx // self.columns
            col = idx % self.columns
            left = self.cfg.margin + col * self.cfg.tile_w_total
            upper = self.cfg.margin + row * self.cfg.tile_h_total
            right = left + self.cfg.tile_w
            lower = upper + self.cfg.tile_h
            tile = self.image.crop((left, upper, right, lower))
            label = self.current_labels.get(idx, "")
            safe_label = "".join(ch for ch in label if ch.isalnum() or ch in ("-", "_")).strip("_-")
            name = f"{idx:03d}"
            if safe_label:
                name = f"{name}_{safe_label}"
            tile.save(target_dir / f"{name}.png")
        self._status_var.set(f"Exported {len(indexes)} tiles to {target_dir}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Sprite sheet explorer and labeler.")
    parser.add_argument("path", type=Path, help="Path to image file or directory of images.")
    parser.add_argument("--tile-w", type=int, default=512, help="Initial tile width in pixels.")
    parser.add_argument("--tile-h", type=int, default=512, help="Initial tile height in pixels.")
    parser.add_argument("--margin", type=int, default=0, help="Outer margin in pixels.")
    parser.add_argument("--spacing", type=int, default=0, help="Spacing between tiles in pixels.")
    parser.add_argument(
        "--scale", type=float, default=0.4, help="UI scale factor for the sheet."
    )
    parser.add_argument(
        "--out-dir", type=Path, default=Path("sprite_exports"), help="Output dir."
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    
    images = []
    if args.path.is_file():
        images = [args.path]
    elif args.path.is_dir():
        images = sorted([
            p for p in args.path.iterdir() 
            if p.is_file() and p.suffix.lower() in ('.png', '.jpg', '.jpeg', '.bmp')
        ])
    
    if not images:
        print(f"No images found at {args.path}", file=sys.stderr)
        return

    cfg = SheetConfig(
        tile_w=args.tile_w,
        tile_h=args.tile_h,
        margin=args.margin,
        spacing=args.spacing,
        scale=args.scale,
    )
    app = SpriteExplorer(images, cfg, out_dir=args.out_dir)
    app.mainloop()


if __name__ == "__main__":
    main()
