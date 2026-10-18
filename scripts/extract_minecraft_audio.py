#!/usr/bin/env python3
"""
MineRust Local Minecraft Audio Extractor
Selects strictly necessary core audio files from a local Minecraft Java Edition installation
and converts them into 16-bit 48kHz mono WAV files for local personal playback.

Usage:
  python3 scripts/extract_minecraft_audio.py                 # Extracts to ./assets/audio
  python3 scripts/extract_minecraft_audio.py --to-user-dir   # Extracts to user data dir (for AppImage / macOS DMG)
"""

import argparse
import glob
import json
import os
import platform
import subprocess
import sys

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


def default_minecraft_assets_dir() -> str:
    system = platform.system()
    home = os.path.expanduser("~")
    if system == "Windows":
        appdata = os.environ.get("APPDATA", os.path.join(home, "AppData", "Roaming"))
        return os.path.join(appdata, ".minecraft", "assets")
    if system == "Darwin":
        return os.path.join(home, "Library", "Application Support", "minecraft", "assets")
    return os.path.join(home, ".minecraft", "assets")


def default_user_data_audio_dir() -> str:
    env_dir = os.environ.get("MINERUST_DATA_DIR")
    if env_dir:
        return os.path.join(env_dir, "assets", "audio")
    system = platform.system()
    home = os.path.expanduser("~")
    if system == "Windows":
        appdata = os.environ.get("APPDATA", os.path.join(home, "AppData", "Roaming"))
        return os.path.join(appdata, "minerust", "assets", "audio")
    if system == "Darwin":
        return os.path.join(home, "Library", "Application Support", "minerust", "assets", "audio")
    xdg = os.environ.get("XDG_DATA_HOME", os.path.join(home, ".local", "share"))
    return os.path.join(xdg, "minerust", "assets", "audio")


def find_latest_index(indexes_dir: str) -> str:
    candidates = glob.glob(os.path.join(indexes_dir, "*.json"))
    if not candidates:
        return ""
    return max(candidates, key=os.path.getmtime)


def main():
    parser = argparse.ArgumentParser(description="Extract local Minecraft audio for MineRust")
    parser.add_argument(
        "--minecraft-assets",
        default=default_minecraft_assets_dir(),
        help="Path to .minecraft/assets directory",
    )
    parser.add_argument(
        "--output-dir",
        default="assets/audio",
        help="Destination directory for converted WAV files (default: assets/audio)",
    )
    parser.add_argument(
        "--to-user-dir",
        action="store_true",
        help="Extract into the OS user data directory (~/.local/share/minerust/assets/audio, etc.) for AppImage/DMG overrides",
    )
    args = parser.parse_args()

    output_dir = default_user_data_audio_dir() if args.to_user_dir else args.output_dir
    indexes_dir = os.path.join(args.minecraft_assets, "indexes")
    objects_dir = os.path.join(args.minecraft_assets, "objects")

    index_path = find_latest_index(indexes_dir)
    if not index_path or not os.path.exists(index_path):
        print(f"Error: No Minecraft asset index JSON found in {indexes_dir}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(output_dir, exist_ok=True)
    print(f"Using Minecraft index: {index_path}")
    print(f"Output directory:      {output_dir}")

    with open(index_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    objects = data.get("objects", {})
    extracted_count = 0

    for out_name, mc_key in SOUND_MAPPING.items():
        if mc_key not in objects:
            print(f"Warning: Key {mc_key} not found in index!", file=sys.stderr)
            continue

        obj_hash = objects[mc_key]["hash"]
        src_path = os.path.join(objects_dir, obj_hash[:2], obj_hash)

        if not os.path.exists(src_path):
            print(f"Warning: Object file not found at {src_path}", file=sys.stderr)
            continue

        dst_path = os.path.join(output_dir, out_name)

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
