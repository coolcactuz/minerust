#!/usr/bin/env python3
import json
import glob
import os
import sys

def main():
    results_dir = sys.argv[1] if len(sys.argv) > 1 else "benchmark_results"
    pattern = os.path.join(results_dir, "*.json")
    files = sorted(glob.glob(pattern))

    if not files:
        print(f"[ERROR] No benchmark JSON files found in {results_dir}")
        sys.exit(1)

    data_list = []
    for f in files:
        with open(f, "r") as fp:
            try:
                data = json.load(fp)
                data["_file"] = os.path.basename(f)
                data_list.append(data)
            except Exception as e:
                print(f"Warning: could not parse {f}: {e}")

    if not data_list:
        print("No valid benchmark data.")
        sys.exit(1)

    baseline_fps = data_list[0].get("avg_fps", 1.0)
    baseline_verts = data_list[0].get("peak_vertices", 1)

    headers = [
        "Scenario / Preset",
        "Avg FPS",
        "1% Low FPS",
        "Avg Frametime",
        "Peak Vertices",
        "Peak Triangles",
        "Peak RAM (RSS)",
        "FPS Speedup",
        "Geometry Cut",
    ]

    rows = []
    for d in data_list:
        name = d.get("preset", d.get("_file", "Unknown"))
        fps = d.get("avg_fps", 0.0)
        p99 = d.get("one_percent_low_fps", 0.0)
        ft = d.get("avg_frametime_ms", 0.0)
        p_verts = d.get("peak_vertices", 0)
        p_tris = p_verts // 2
        rss = d.get("peak_rss_mb", 0.0)

        speedup = (fps / baseline_fps) if baseline_fps > 0 else 1.0
        geom_drop = (1.0 - (p_verts / baseline_verts)) * 100.0 if baseline_verts > 0 else 0.0

        p_verts_str = f"{p_verts / 1_000_000:.2f}M" if p_verts >= 1_000_000 else f"{p_verts:,}"
        p_tris_str = f"{p_tris / 1_000_000:.2f}M" if p_tris >= 1_000_000 else f"{p_tris:,}"

        rows.append([
            name,
            f"{fps:.1f} FPS",
            f"{p99:.1f} FPS",
            f"{ft:.2f} ms",
            p_verts_str,
            p_tris_str,
            f"{rss:.1f} MB",
            f"{speedup:.2f}x" if speedup != 1.0 else "Baseline (1.0x)",
            f"-{geom_drop:.1f}%" if geom_drop > 0 else "0.0%",
        ])

    col_widths = [len(h) for h in headers]
    for r in rows:
        for i, val in enumerate(r):
            col_widths[i] = max(col_widths[i], len(val))

    def fmt_row(items):
        return "| " + " | ".join(f"{item:<{col_widths[i]}}" for i, item in enumerate(items)) + " |"

    def fmt_sep():
        return "|-" + "-|-".join("-" * col_widths[i] for i in range(len(headers))) + "-|"

    output_lines = [
        "# MineRust: 1km Trajectory Optimization Benchmark Comparison",
        "",
        f"- **World Seed**: `{data_list[0].get('seed', 'N/A')}`",
        f"- **Distance Traveled**: `{data_list[0].get('distance_meters', 1000):.0f} meters (1.00 km)`",
        f"- **Render Distance**: `{data_list[0].get('view_distance', 16)} chunks`",
        f"- **VSync**: `AutoNoVsync (Uncapped)`",
        "",
        fmt_row(headers),
        fmt_sep(),
    ]

    for r in rows:
        output_lines.append(fmt_row(r))

    output_lines.append("")
    summary_text = "\n".join(output_lines)
    print("\n" + summary_text)

    summary_file = os.path.join(results_dir, "SUMMARY.md")
    with open(summary_file, "w") as fp:
        fp.write(summary_text)
    print(f"[BENCHMARK] Summary saved to: {summary_file}")

if __name__ == "__main__":
    main()
