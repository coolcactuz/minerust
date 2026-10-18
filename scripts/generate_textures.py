#!/usr/bin/env python3
"""
MineRust Texture Pipeline — Authentic Minecraft Vanilla Assets
Downloads official Minecraft client assets from Mojang's version manifest and
extracts pixel-perfect block textures, item icons, and 3D isometric inventory mini-cubes.
"""

import io
import os
import urllib.request
import zipfile
from PIL import Image

CACHE_JAR = "/tmp/minerust_mc_client.jar"
BLOCKS_DIR = "assets/textures/blocks"
ITEMS_DIR = "assets/textures/items"

os.makedirs(BLOCKS_DIR, exist_ok=True)
os.makedirs(ITEMS_DIR, exist_ok=True)


def ensure_client_jar():
    if os.path.exists(CACHE_JAR) and os.path.getsize(CACHE_JAR) > 10_000_000:
        return zf_from_file(CACHE_JAR)

    print("Fetching Mojang version manifest...")
    manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
    req = urllib.request.Request(manifest_url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req) as resp:
        import json
        manifest = json.loads(resp.read().decode())

    target_ver = "1.20.4"
    ver_info = next(v for v in manifest["versions"] if v["id"] == target_ver)
    ver_req = urllib.request.Request(ver_info["url"], headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(ver_req) as ver_resp:
        ver_data = json.loads(ver_resp.read().decode())

    client_url = ver_data["downloads"]["client"]["url"]
    print(f"Downloading vanilla Minecraft {target_ver} client.jar...")
    client_req = urllib.request.Request(client_url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(client_req) as client_resp:
        data = client_resp.read()
        with open(CACHE_JAR, "wb") as f:
            f.write(data)

    return zf_from_file(CACHE_JAR)


def zf_from_file(path):
    return zipfile.ZipFile(path)


def get_pixel_list(img):
    try:
        return list(img.get_flattened_data())
    except AttributeError:
        return list(img.getdata())


def main():
    zf = ensure_client_jar()

    def get_img(zip_path):
        data = zf.read(zip_path)
        return Image.open(io.BytesIO(data)).convert("RGBA")

    # 1. Block Textures
    block_mapping = {
        "dirt.png": "assets/minecraft/textures/block/dirt.png",
        "stone.png": "assets/minecraft/textures/block/stone.png",
        "cobblestone.png": "assets/minecraft/textures/block/cobblestone.png",
        "wood_side.png": "assets/minecraft/textures/block/oak_log.png",
        "wood_top.png": "assets/minecraft/textures/block/oak_log_top.png",
        "planks.png": "assets/minecraft/textures/block/oak_planks.png",
        "sand.png": "assets/minecraft/textures/block/sand.png",
        "sandstone_side.png": "assets/minecraft/textures/block/sandstone.png",
        "sandstone_top.png": "assets/minecraft/textures/block/sandstone_top.png",
        "snow.png": "assets/minecraft/textures/block/snow.png",
        "snow_side.png": "assets/minecraft/textures/block/grass_block_snow.png",
        "bedrock.png": "assets/minecraft/textures/block/bedrock.png",
        "coal_ore.png": "assets/minecraft/textures/block/coal_ore.png",
        "iron_ore.png": "assets/minecraft/textures/block/iron_ore.png",
        "gold_ore.png": "assets/minecraft/textures/block/gold_ore.png",
        "diamond_ore.png": "assets/minecraft/textures/block/diamond_ore.png",
        "cactus_side.png": "assets/minecraft/textures/block/cactus_side.png",
        "cactus_top.png": "assets/minecraft/textures/block/cactus_top.png",
        "gravel.png": "assets/minecraft/textures/block/gravel.png",
        "ice.png": "assets/minecraft/textures/block/ice.png",
        "glass.png": "assets/minecraft/textures/block/glass.png",
        "grass_side.png": "assets/minecraft/textures/block/grass_block_side.png",
        "crafting_table_top.png": "assets/minecraft/textures/block/crafting_table_top.png",
        "crafting_table_side.png": "assets/minecraft/textures/block/crafting_table_side.png",
        "crafting_table_front.png": "assets/minecraft/textures/block/crafting_table_front.png",
        "furnace_top.png": "assets/minecraft/textures/block/furnace_top.png",
        "furnace_side.png": "assets/minecraft/textures/block/furnace_side.png",
        "furnace_front.png": "assets/minecraft/textures/block/furnace_front.png",
        "furnace_lit.png": "assets/minecraft/textures/block/furnace_front_on.png",
    }

    for out_name, zip_path in block_mapping.items():
        img = get_img(zip_path)
        img.save(os.path.join(BLOCKS_DIR, out_name))

    # Tinted blocks using vanilla colormap coordinates (Plains biome: temp=0.8, rain=0.4)
    grass_map = get_img("assets/minecraft/textures/colormap/grass.png")
    foliage_map = get_img("assets/minecraft/textures/colormap/foliage.png")
    plains_grass = grass_map.getpixel((51, 173))
    plains_foliage = foliage_map.getpixel((51, 173))

    # Grass top
    gt_raw = get_img("assets/minecraft/textures/block/grass_block_top.png")
    gt_px = []
    for r, g, b, a in get_pixel_list(gt_raw):
        factor = r / 255.0
        gt_px.append((
            int(plains_grass[0] * factor),
            int(plains_grass[1] * factor),
            int(plains_grass[2] * factor),
            a,
        ))
    gt_img = Image.new("RGBA", (16, 16))
    gt_img.putdata(gt_px)
    gt_img.save(os.path.join(BLOCKS_DIR, "grass_top.png"))

    # Oak Leaves
    lv_raw = get_img("assets/minecraft/textures/block/oak_leaves.png")
    lv_px = []
    for r, g, b, a in get_pixel_list(lv_raw):
        if a == 0:
            lv_px.append((0, 0, 0, 0))
        else:
            factor = r / 255.0
            lv_px.append((
                int(plains_foliage[0] * factor),
                int(plains_foliage[1] * factor),
                int(plains_foliage[2] * factor),
                a,
            ))
    lv_img = Image.new("RGBA", (16, 16))
    lv_img.putdata(lv_px)
    lv_img.save(os.path.join(BLOCKS_DIR, "leaves.png"))

    # Water (still frame 0 tinted)
    water_anim = get_img("assets/minecraft/textures/block/water_still.png")
    water_frame0 = water_anim.crop((0, 0, 16, 16))
    wt_px = []
    for r, g, b, a in get_pixel_list(water_frame0):
        factor = r / 255.0
        wt_px.append((int(63 * factor), int(118 * factor), int(228 * factor), 200))
    wt_img = Image.new("RGBA", (16, 16))
    wt_img.putdata(wt_px)
    wt_img.save(os.path.join(BLOCKS_DIR, "water.png"))

    # Chest faces from entity/chest/normal.png
    chest = get_img("assets/minecraft/textures/entity/chest/normal.png")
    chest_rim = (40, 28, 18, 255)

    # Top
    lid_top = chest.crop((14, 0, 28, 14))
    chest_top = Image.new("RGBA", (16, 16), chest_rim)
    chest_top.paste(lid_top, (1, 1))
    chest_top.save(os.path.join(BLOCKS_DIR, "chest_top.png"))

    # Side
    lid_side = chest.crop((0, 14, 14, 19))
    base_side = chest.crop((0, 34, 14, 43))
    side_14 = Image.new("RGBA", (14, 14))
    side_14.paste(lid_side, (0, 0))
    side_14.paste(base_side, (0, 5))
    chest_side = Image.new("RGBA", (16, 16), chest_rim)
    chest_side.paste(side_14, (1, 1))
    chest_side.save(os.path.join(BLOCKS_DIR, "chest_side.png"))

    # Front
    lid_front = chest.crop((14, 14, 28, 19))
    base_front = chest.crop((14, 34, 28, 43))
    lock = chest.crop((1, 1, 3, 5))
    front_14 = Image.new("RGBA", (14, 14))
    front_14.paste(lid_front, (0, 0))
    front_14.paste(base_front, (0, 5))
    front_14.paste(lock, (6, 3))
    chest_front = Image.new("RGBA", (16, 16), chest_rim)
    chest_front.paste(front_14, (1, 1))
    chest_front.save(os.path.join(BLOCKS_DIR, "chest_front.png"))

    print(f"Generated 35 pixel-perfect Minecraft block textures in '{BLOCKS_DIR}'.")

    # 2. Item Textures (Materials & Tools)
    items_mapping = {
        "stick.png": "assets/minecraft/textures/item/stick.png",
        "coal.png": "assets/minecraft/textures/item/coal.png",
        "iron_ingot.png": "assets/minecraft/textures/item/iron_ingot.png",
        "gold_ingot.png": "assets/minecraft/textures/item/gold_ingot.png",
        "diamond.png": "assets/minecraft/textures/item/diamond.png",

        "wood_sword.png": "assets/minecraft/textures/item/wooden_sword.png",
        "stone_sword.png": "assets/minecraft/textures/item/stone_sword.png",
        "iron_sword.png": "assets/minecraft/textures/item/iron_sword.png",
        "gold_sword.png": "assets/minecraft/textures/item/golden_sword.png",
        "diamond_sword.png": "assets/minecraft/textures/item/diamond_sword.png",

        "wood_pickaxe.png": "assets/minecraft/textures/item/wooden_pickaxe.png",
        "stone_pickaxe.png": "assets/minecraft/textures/item/stone_pickaxe.png",
        "iron_pickaxe.png": "assets/minecraft/textures/item/iron_pickaxe.png",
        "gold_pickaxe.png": "assets/minecraft/textures/item/golden_pickaxe.png",
        "diamond_pickaxe.png": "assets/minecraft/textures/item/diamond_pickaxe.png",

        "wood_axe.png": "assets/minecraft/textures/item/wooden_axe.png",
        "stone_axe.png": "assets/minecraft/textures/item/stone_axe.png",
        "iron_axe.png": "assets/minecraft/textures/item/iron_axe.png",
        "gold_axe.png": "assets/minecraft/textures/item/golden_axe.png",
        "diamond_axe.png": "assets/minecraft/textures/item/diamond_axe.png",

        "wood_shovel.png": "assets/minecraft/textures/item/wooden_shovel.png",
        "stone_shovel.png": "assets/minecraft/textures/item/stone_shovel.png",
        "iron_shovel.png": "assets/minecraft/textures/item/iron_shovel.png",
        "gold_shovel.png": "assets/minecraft/textures/item/golden_shovel.png",
        "diamond_shovel.png": "assets/minecraft/textures/item/diamond_shovel.png",

        "wood_hoe.png": "assets/minecraft/textures/item/wooden_hoe.png",
        "stone_hoe.png": "assets/minecraft/textures/item/stone_hoe.png",
        "iron_hoe.png": "assets/minecraft/textures/item/iron_hoe.png",
        "gold_hoe.png": "assets/minecraft/textures/item/golden_hoe.png",
        "diamond_hoe.png": "assets/minecraft/textures/item/diamond_hoe.png",
    }

    for out_name, zip_path in items_mapping.items():
        img = get_img(zip_path)
        img.save(os.path.join(ITEMS_DIR, out_name))

    print(f"Generated 30 pixel-perfect Minecraft item icons in '{ITEMS_DIR}'.")

    # 3. 3D Isometric Mini-Cubes for Placeable Blocks
    def render_isometric_cube(top_path, left_path, right_path, out_path):
        top_px = get_pixel_list(Image.open(top_path).convert("RGBA"))
        left_px = get_pixel_list(Image.open(left_path).convert("RGBA"))
        right_px = get_pixel_list(Image.open(right_path).convert("RGBA"))

        def shade(color, factor):
            r, g, b, a = color
            if a == 0:
                return (0, 0, 0, 0)
            return (int(r * factor), int(g * factor), int(b * factor), a)

        out = [(0, 0, 0, 0)] * 256

        for y in range(16):
            for x in range(16):
                px = x + 0.5
                py = y + 0.5

                # Top face (illuminated, 1.0x brightness)
                if py < 8.0:
                    d_tl = (py - 1.0) - (7.0 / 12.0) * (8.0 - px)
                    d_tr = (py - 1.0) - (7.0 / 12.0) * (px - 8.0)
                    d_bl = (py - 8.0) + (7.0 / 12.0) * (8.0 - px)
                    d_br = (py - 8.0) + (7.0 / 12.0) * (px - 8.0)
                    if d_tl >= 0 and d_tr >= 0 and d_bl < 0 and d_br < 0:
                        u = int(min(15, max(0, (px - 8.0) * 1.25 + (py - 1.0) * (15.0 / 7.0))))
                        v = int(min(15, max(0, -(px - 8.0) * 1.25 + (py - 1.0) * (15.0 / 7.0))))
                        c = top_px[v * 16 + u]
                        out[y * 16 + x] = shade(c, 1.0)
                        continue

                # Left face (directional daylight, 0.82x brightness)
                if 2.0 <= px < 8.0:
                    d_bl = (py - 8.0) + (7.0 / 12.0) * (8.0 - px)
                    d_bot = (py - 15.0) + (7.0 / 12.0) * (8.0 - px)
                    if d_bl >= 0 and d_bot <= 0:
                        u = int(min(15, max(0, ((px - 2.0) / 6.0) * 15.0)))
                        y_top = 8.0 - (7.0 / 12.0) * (8.0 - px)
                        v = int(min(15, max(0, ((py - y_top) / 7.0) * 15.0)))
                        c = left_px[v * 16 + u]
                        out[y * 16 + x] = shade(c, 0.82)
                        continue

                # Right face (shadow side, 0.62x brightness)
                if 8.0 <= px <= 14.0:
                    d_br = (py - 8.0) + (7.0 / 12.0) * (px - 8.0)
                    d_bot = (py - 15.0) + (7.0 / 12.0) * (px - 8.0)
                    if d_br >= 0 and d_bot <= 0:
                        u = int(min(15, max(0, ((px - 8.0) / 6.0) * 15.0)))
                        y_top = 8.0 - (7.0 / 12.0) * (px - 8.0)
                        v = int(min(15, max(0, ((py - y_top) / 7.0) * 15.0)))
                        c = right_px[v * 16 + u]
                        out[y * 16 + x] = shade(c, 0.62)
                        continue

        img = Image.new("RGBA", (16, 16))
        img.putdata(out)
        img.save(out_path)

    cube_mappings = [
        ("grass.png", f"{BLOCKS_DIR}/grass_top.png", f"{BLOCKS_DIR}/grass_side.png", f"{BLOCKS_DIR}/grass_side.png"),
        ("dirt.png", f"{BLOCKS_DIR}/dirt.png", f"{BLOCKS_DIR}/dirt.png", f"{BLOCKS_DIR}/dirt.png"),
        ("stone.png", f"{BLOCKS_DIR}/stone.png", f"{BLOCKS_DIR}/stone.png", f"{BLOCKS_DIR}/stone.png"),
        ("cobblestone.png", f"{BLOCKS_DIR}/cobblestone.png", f"{BLOCKS_DIR}/cobblestone.png", f"{BLOCKS_DIR}/cobblestone.png"),
        ("wood.png", f"{BLOCKS_DIR}/wood_top.png", f"{BLOCKS_DIR}/wood_side.png", f"{BLOCKS_DIR}/wood_side.png"),
        ("planks.png", f"{BLOCKS_DIR}/planks.png", f"{BLOCKS_DIR}/planks.png", f"{BLOCKS_DIR}/planks.png"),
        ("leaves.png", f"{BLOCKS_DIR}/leaves.png", f"{BLOCKS_DIR}/leaves.png", f"{BLOCKS_DIR}/leaves.png"),
        ("glass.png", f"{BLOCKS_DIR}/glass.png", f"{BLOCKS_DIR}/glass.png", f"{BLOCKS_DIR}/glass.png"),
        ("sand.png", f"{BLOCKS_DIR}/sand.png", f"{BLOCKS_DIR}/sand.png", f"{BLOCKS_DIR}/sand.png"),
        ("sandstone.png", f"{BLOCKS_DIR}/sandstone_top.png", f"{BLOCKS_DIR}/sandstone_side.png", f"{BLOCKS_DIR}/sandstone_side.png"),
        ("gravel.png", f"{BLOCKS_DIR}/gravel.png", f"{BLOCKS_DIR}/gravel.png", f"{BLOCKS_DIR}/gravel.png"),
        ("water.png", f"{BLOCKS_DIR}/water.png", f"{BLOCKS_DIR}/water.png", f"{BLOCKS_DIR}/water.png"),
        ("ice.png", f"{BLOCKS_DIR}/ice.png", f"{BLOCKS_DIR}/ice.png", f"{BLOCKS_DIR}/ice.png"),
        ("snow.png", f"{BLOCKS_DIR}/snow.png", f"{BLOCKS_DIR}/snow_side.png", f"{BLOCKS_DIR}/snow_side.png"),
        ("cactus.png", f"{BLOCKS_DIR}/cactus_top.png", f"{BLOCKS_DIR}/cactus_side.png", f"{BLOCKS_DIR}/cactus_side.png"),
        ("coal_ore.png", f"{BLOCKS_DIR}/coal_ore.png", f"{BLOCKS_DIR}/coal_ore.png", f"{BLOCKS_DIR}/coal_ore.png"),
        ("iron_ore.png", f"{BLOCKS_DIR}/iron_ore.png", f"{BLOCKS_DIR}/iron_ore.png", f"{BLOCKS_DIR}/iron_ore.png"),
        ("gold_ore.png", f"{BLOCKS_DIR}/gold_ore.png", f"{BLOCKS_DIR}/gold_ore.png", f"{BLOCKS_DIR}/gold_ore.png"),
        ("diamond_ore.png", f"{BLOCKS_DIR}/diamond_ore.png", f"{BLOCKS_DIR}/diamond_ore.png", f"{BLOCKS_DIR}/diamond_ore.png"),
        ("bedrock.png", f"{BLOCKS_DIR}/bedrock.png", f"{BLOCKS_DIR}/bedrock.png", f"{BLOCKS_DIR}/bedrock.png"),
        ("crafting_table.png", f"{BLOCKS_DIR}/crafting_table_top.png", f"{BLOCKS_DIR}/crafting_table_side.png", f"{BLOCKS_DIR}/crafting_table_front.png"),
        ("chest.png", f"{BLOCKS_DIR}/chest_top.png", f"{BLOCKS_DIR}/chest_side.png", f"{BLOCKS_DIR}/chest_front.png"),
        ("furnace.png", f"{BLOCKS_DIR}/furnace_top.png", f"{BLOCKS_DIR}/furnace_side.png", f"{BLOCKS_DIR}/furnace_front.png"),
        ("furnace_lit.png", f"{BLOCKS_DIR}/furnace_top.png", f"{BLOCKS_DIR}/furnace_side.png", f"{BLOCKS_DIR}/furnace_lit.png"),
    ]

    for item_name, top_p, left_p, right_p in cube_mappings:
        render_isometric_cube(top_p, left_p, right_p, os.path.join(ITEMS_DIR, item_name))

    print(f"Generated 24 3D isometric mini-cube icons in '{ITEMS_DIR}'.")


if __name__ == "__main__":
    main()
