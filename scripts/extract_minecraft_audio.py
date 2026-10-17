#!/usr/bin/env python3
"""
MineRust Official Minecraft Audio Extractor
Selects only strictly necessary core Minecraft audio files from local Minecraft assets
and converts them into 16-bit 48kHz WAV files for zero-overhead, high-fidelity playback.
"""

import json
import os
import subprocess
import sys

INDEX_PATH = "/home/cactuz/.minecraft/assets/indexes/34.json"
OBJECTS_DIR = "/home/cactuz/.minecraft/assets/objects"
OUTPUT_DIR = "assets/audio"

SOUND_MAPPING = {
    "click.wav": "minecraft/sounds/random/click.ogg",
    "pop.wav": "minecraft/sounds/random/pop.ogg",
    "step.wav": "minecraft/sounds/step/grass1.ogg",
    "dig_grass.wav": "minecraft/sounds/dig/grass1.ogg",
    "dig_stone.wav": "minecraft/sounds/dig/stone1.ogg",
    "dig_wood.wav": "minecraft/sounds/dig/wood1.ogg",
    "dig_sand.wav": "minecraft/sounds/dig/sand1.ogg",
    "break_block.wav": "minecraft/sounds/dig/stone1.ogg",
    "chest_open.wav": "minecraft/sounds/random/chestopen.ogg",
    "chest_close.wav": "minecraft/sounds/random/chestclosed.ogg",
    "furnace_crackle.wav": "minecraft/sounds/block/furnace/fire_crackle1.ogg",
}


def main():
    if not os.path.exists(INDEX_PATH):
        print(f"Error: Index file not found at {INDEX_PATH}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(OUTPUT_DIR, exist_ok=True)

    with open(INDEX_PATH, "r", encoding="utf-8") as f:
        data = json.load(f)

    objects = data.get("objects", {})
    extracted_count = 0

    for out_name, mc_key in SOUND_MAPPING.items():
        if mc_key not in objects:
            print(f"Warning: Key {mc_key} not found in index!", file=sys.stderr)
            continue

        obj_hash = objects[mc_key]["hash"]
        src_path = os.path.join(OBJECTS_DIR, obj_hash[:2], obj_hash)

        if not os.path.exists(src_path):
            print(f"Warning: Object file not found at {src_path}", file=sys.stderr)
            continue

        dst_path = os.path.join(OUTPUT_DIR, out_name)

        # Convert OGG to 16-bit 48kHz mono WAV using ffmpeg with high quality
        cmd = [
            "ffmpeg",
            "-y",
            "-i",
            src_path,
            "-ar",
            "48000",
            "-ac",
            "1",
            "-sample_fmt",
            "s16",
            dst_path,
        ]

        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error converting {mc_key} -> {dst_path}: {result.stderr}", file=sys.stderr)
            continue

        size_kb = os.path.getsize(dst_path) / 1024.0
        print(f"Extracted & Converted: {mc_key} -> {dst_path} ({size_kb:.1f} KB)")
        extracted_count += 1

    print(f"\nSuccessfully extracted and converted {extracted_count}/{len(SOUND_MAPPING)} audio files.")


if __name__ == "__main__":
    main()
