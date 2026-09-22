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

    baseline_static_fps = data_list[0].get("static_fps", data_list[0].get("avg_fps", 1.0))
    baseline_static_verts = data_list[0].get("static_vertices", data_list[0].get("peak_vertices", 1))
    baseline_flight_fps = data_list[0].get("flight_avg_fps", data_list[0].get("avg_fps", 1.0))

    headers = [
        "Scenario / Preset",
        "Static FPS (Pure Render)",
        "Static Geometry",
        "Static VRAM",
        "Static Speedup",
        "Flight Avg FPS",
        "Flight 1% Low",
        "Flight Frametime",
        "Peak RAM (RSS)",
        "Peak GPU VRAM",
        "Flight Speedup",
    ]

    rows = []
    for d in data_list:
        name = d.get("preset", d.get("_file", "Unknown"))
        
        # Static baseline metrics
        s_fps = d.get("static_fps", d.get("avg_fps", 0.0))
        s_verts = d.get("static_vertices", d.get("peak_vertices", 0))
        s_vram = d.get("static_vram_mb", 0.0)
        s_speedup = (s_fps / baseline_static_fps) if baseline_static_fps > 0 else 1.0
        s_verts_str = f"{s_verts / 1_000_000:.2f}M" if s_verts >= 1_000_000 else f"{s_verts:,}"
        s_vram_str = f"{s_vram:.1f} MB" if s_vram > 0.0 else "N/A"

        # Dynamic flight metrics
        f_fps = d.get("flight_avg_fps", d.get("avg_fps", 0.0))
        f_p99 = d.get("flight_one_percent_low_fps", d.get("one_percent_low_fps", 0.0))
        f_ft = d.get("flight_avg_frametime_ms", d.get("avg_frametime_ms", 0.0))
        rss = d.get("flight_peak_rss_mb", d.get("peak_rss_mb", 0.0))
        f_vram = d.get("flight_peak_vram_mb", 0.0)
        f_vram_str = f"{f_vram:.1f} MB" if f_vram > 0.0 else "N/A"
        f_speedup = (f_fps / baseline_flight_fps) if baseline_flight_fps > 0 else 1.0

        rows.append([
            name,
            f"{s_fps:.1f} FPS",
            f"{s_verts_str} verts",
            s_vram_str,
            f"{s_speedup:.2f}x" if s_speedup != 1.0 else "Baseline (1.0x)",
            f"{f_fps:.1f} FPS",
            f"{f_p99:.1f} FPS",
            f"{f_ft:.2f} ms",
            f"{rss:.1f} MB",
            f_vram_str,
            f"{f_speedup:.2f}x" if f_speedup != 1.0 else "Baseline (1.0x)",
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
