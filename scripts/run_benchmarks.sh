#!/usr/bin/env bash
set -e

# ==============================================================================
# MineRust 1km Trajectory Benchmark Suite
# Executes reproducible, deterministic flight benchmarks across optimization presets
# ==============================================================================

SEED=133742
DISTANCE=1000      # 1 km standard flight trajectory
SPEED=50           # 50 m/s (180 km/h) = 20s recording time per run
WARMUP=2.5         # 2.5s warmup to initialize initial chunk geometry
VIEW_DIST=16       # 16 chunks render distance
RESULTS_DIR="benchmark_results"

mkdir -p "$RESULTS_DIR"
rm -f "$RESULTS_DIR"/*.json

echo "======================================================================"
echo "          MINERUST: 1KM OPTIMIZATION BENCHMARK SUITE                  "
echo "======================================================================"
echo " Building release binary with optimizations..."
cargo build --release
ln -sfn "$(pwd)/assets" target/release/assets

echo ""
echo "Configuration:"
echo " - World Seed       : $SEED"
echo " - Flight Distance  : $DISTANCE meters (1.00 km)"
echo " - Flight Speed     : $SPEED m/s (Altitude: 92.0)"
echo " - Render Distance  : $VIEW_DIST chunks"
echo " - Framerate Mode   : Uncapped (AutoNoVsync)"
echo " - Results Output   : $RESULTS_DIR/"
echo "======================================================================"
echo ""

run_scenario() {
    local PRESET="$1"
    local OUTPUT_FILE="$2"
    local LABEL="$3"

    echo "----------------------------------------------------------------------"
    echo ">> Running Scenario: $LABEL (Preset: $PRESET)"
    echo "----------------------------------------------------------------------"

    target/release/minerust \
        --benchmark \
        --benchmark-preset "$PRESET" \
        --benchmark-distance "$DISTANCE" \
        --benchmark-speed "$SPEED" \
        --benchmark-warmup "$WARMUP" \
        --benchmark-output "$OUTPUT_FILE" \
        --seed "$SEED" \
        --view-distance "$VIEW_DIST"

    echo ""
}

# 1. Baseline: Raw naive meshing, no LOD, no culling, no max-y skip
run_scenario "baseline" "$RESULTS_DIR/1_baseline.json" "1. Baseline (Raw Naive Meshing, No Culling)"

# 2. Culling: Backface culling + Max-Y skip ON (Naive meshing)
run_scenario "culling" "$RESULTS_DIR/2_culling.json" "2. Culling (Backface Culling & Max-Y Skip)"

# 3. Greedy: Greedy Meshing ON, no LOD
run_scenario "greedy" "$RESULTS_DIR/3_greedy.json" "3. Greedy Meshing (Coplanar Quad Merging)"

# 4. Sloped LOD: Greedy Meshing + 3D Sloped Heightfield LOD
run_scenario "sloped_lod" "$RESULTS_DIR/4_sloped_lod.json" "4. Sloped LOD (Greedy + 3D Heightfield LOD)"

# 5. Full Production: All optimizations ON
run_scenario "production" "$RESULTS_DIR/5_production.json" "5. Full Production (All Optimizations + Fog)"

# Run comparative analysis
python3 scripts/compare_benchmarks.py "$RESULTS_DIR"
