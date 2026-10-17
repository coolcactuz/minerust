#!/usr/bin/env python3
"""
MineRust Procedural Audio Synthesizer
Generates ~10 essential, ultra-lightweight, 100% royalty-free 16-bit 44.1kHz WAV
sound effects for voxel gameplay, UI interactions, containers, and digging.
"""

import math
import os
import random
import struct
import wave

AUDIO_DIR = "assets/audio"
os.makedirs(AUDIO_DIR, exist_ok=True)
SAMPLE_RATE = 44100


def save_wav(name, samples):
    path = os.path.join(AUDIO_DIR, name)
    with wave.open(path, "w") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SAMPLE_RATE)
        buf = bytearray()
        for s in samples:
            clamped = max(-1.0, min(1.0, s))
            buf.extend(struct.pack("<h", int(clamped * 32767)))
        w.writeframes(buf)
    size_kb = os.path.getsize(path) / 1024.0
    ms = len(samples) / SAMPLE_RATE * 1000.0
    print(f"Generated {name:20s}: {ms:5.1f} ms | {size_kb:4.1f} KB")


def simple_lowpass(samples, alpha):
    out = []
    prev = 0.0
    for s in samples:
        prev = prev + alpha * (s - prev)
        out.append(prev)
    return out


def simple_highpass(samples, alpha):
    out = []
    prev_s = 0.0
    prev_y = 0.0
    for s in samples:
        y = alpha * (prev_y + s - prev_s)
        out.append(y)
        prev_s = s
        prev_y = y
    return out


def generate_click():
    # 1. Clean UI Click (18 ms)
    n = int(SAMPLE_RATE * 0.018)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.003))
        freq = 2400.0 - 800.0 * (i / n)
        noise = (random.random() * 2.0 - 1.0) * 0.25
        s = (math.sin(2.0 * math.pi * freq * t) * 0.75 + noise) * env
        samples.append(s)
    save_wav("click.wav", samples)


def generate_pop():
    # 2. Item Pickup / Crafting Pop (65 ms upward sweep)
    n = int(SAMPLE_RATE * 0.065)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        attack = 1.0 - math.exp(-i / (SAMPLE_RATE * 0.004))
        decay = math.exp(-i / (SAMPLE_RATE * 0.022))
        freq = 380.0 + 520.0 * (i / n)
        s = math.sin(2.0 * math.pi * freq * t) * attack * decay * 0.85
        samples.append(s)
    save_wav("pop.wav", samples)


def normalize(samples, target_peak=0.92):
    max_val = max((abs(s) for s in samples), default=0.0)
    if max_val > 0.001:
        factor = target_peak / max_val
        return [s * factor for s in samples]
    return samples


def generate_dig_grass():
    # 3. Punchy foliage / grass crunch (95 ms)
    n = int(SAMPLE_RATE * 0.095)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.028))
        thump = math.sin(2.0 * math.pi * 210.0 * t) * 0.45
        rustle = (random.random() * 2.0 - 1.0) * 0.65
        snap = math.sin(2.0 * math.pi * 1250.0 * t) * 0.35 * math.exp(-i / (SAMPLE_RATE * 0.008))
        samples.append((thump + rustle + snap) * env)
    filtered = simple_lowpass(samples, 0.42)
    save_wav("dig_grass.wav", normalize(filtered, 0.92))


def generate_dig_stone():
    # 4. Solid stone clack & chip (85 ms)
    n = int(SAMPLE_RATE * 0.085)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.022))
        knock = math.sin(2.0 * math.pi * 420.0 * t) * 0.50 + math.sin(2.0 * math.pi * 780.0 * t) * 0.30
        ping = math.sin(2.0 * math.pi * 1680.0 * t) * 0.35 + math.sin(2.0 * math.pi * 2450.0 * t) * 0.20
        chip_noise = (random.random() * 2.0 - 1.0) * 0.55 * math.exp(-i / (SAMPLE_RATE * 0.012))
        samples.append((knock + ping + chip_noise) * env)
    save_wav("dig_stone.wav", normalize(samples, 0.94))


def generate_dig_wood():
    # 5. Resonant wood thud & knock (105 ms)
    n = int(SAMPLE_RATE * 0.105)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.028))
        f1 = math.sin(2.0 * math.pi * 190.0 * t) * 0.60
        f2 = math.sin(2.0 * math.pi * 380.0 * t) * 0.35
        f3 = math.sin(2.0 * math.pi * 760.0 * t) * 0.20
        snap = (random.random() * 2.0 - 1.0) * 0.50 * math.exp(-i / (SAMPLE_RATE * 0.008))
        samples.append((f1 + f2 + f3 + snap) * env)
    filtered = simple_lowpass(samples, 0.55)
    save_wav("dig_wood.wav", normalize(filtered, 0.95))


def generate_dig_sand():
    # 6. Granular sand / dirt crunch (90 ms)
    n = int(SAMPLE_RATE * 0.090)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.026))
        thud = math.sin(2.0 * math.pi * 180.0 * t) * 0.40
        grit = (random.random() * 2.0 - 1.0) * 0.70
        samples.append((thud + grit) * env)
    lp = simple_lowpass(samples, 0.50)
    save_wav("dig_sand.wav", normalize(lp, 0.92))


def generate_break_block():
    # 7. Block shatter / destruction impact (280 ms multi-burst crunch & debris)
    n = int(SAMPLE_RATE * 0.280)
    samples = [0.0] * n

    # Phase 1: High-energy initial crack impact (0 to 90 ms)
    crack_len = int(SAMPLE_RATE * 0.090)
    for i in range(crack_len):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.024))
        # High-mid presence snap (1850Hz + 820Hz)
        crack = math.sin(2.0 * math.pi * 1850.0 * t) * 0.45 + math.sin(2.0 * math.pi * 820.0 * t) * 0.35
        noise = (random.random() * 2.0 - 1.0) * 0.70
        punch = math.sin(2.0 * math.pi * (290.0 - 160.0 * (i / crack_len)) * t) * 0.50
        samples[i] += (crack + noise + punch) * env

    # Phase 2: Textured debris / gravel crumble bursts
    burst_offsets = [0.035, 0.075, 0.125, 0.180]
    burst_gains = [0.65, 0.52, 0.38, 0.26]
    for offset, gain in zip(burst_offsets, burst_gains):
        start_idx = int(offset * SAMPLE_RATE)
        burst_len = int(SAMPLE_RATE * 0.055)
        for b in range(burst_len):
            idx = start_idx + b
            if idx >= n:
                break
            t = b / SAMPLE_RATE
            env = math.exp(-b / (SAMPLE_RATE * 0.016))
            freq = 950.0 + random.random() * 850.0
            grain = (random.random() * 2.0 - 1.0) * 0.60 + math.sin(2.0 * math.pi * freq * t) * 0.40
            samples[idx] += grain * env * gain

    save_wav("break_block.wav", normalize(samples, 0.96))


def generate_step():
    # 8. Subtle footstep tap (40 ms)
    n = int(SAMPLE_RATE * 0.040)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.010))
        thud = math.sin(2.0 * math.pi * 110.0 * t) * 0.5
        scuff = (random.random() * 2.0 - 1.0) * 0.35
        samples.append((thud + scuff) * env * 0.5)
    filtered = simple_lowpass(samples, 0.35)
    save_wav("step.wav", filtered)


def generate_chest_open():
    # 9. Chest Open Creak (230 ms ascending glide with wood harmonic)
    n = int(SAMPLE_RATE * 0.230)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = (1.0 - math.exp(-i / (SAMPLE_RATE * 0.02))) * math.exp(-i / (SAMPLE_RATE * 0.12))
        freq = 150.0 + 170.0 * (i / n)
        # Add slight pitch wobble (creaking friction)
        wobble = math.sin(2.0 * math.pi * 45.0 * t) * 15.0
        s = (
            math.sin(2.0 * math.pi * (freq + wobble) * t) * 0.6
            + math.sin(2.0 * math.pi * (freq * 2.0) * t) * 0.25
            + (random.random() * 2.0 - 1.0) * 0.12
        ) * env
        samples.append(s * 0.8)
    save_wav("chest_open.wav", samples)


def generate_chest_close():
    # 10. Chest Close Heavy Latch (150 ms thud + snap)
    n = int(SAMPLE_RATE * 0.150)
    samples = []
    for i in range(n):
        t = i / SAMPLE_RATE
        env = math.exp(-i / (SAMPLE_RATE * 0.035))
        thud = math.sin(2.0 * math.pi * 135.0 * t) * 0.7
        # Latch click at 35 ms
        latch_idx = int(SAMPLE_RATE * 0.035)
        latch = 0.0
        if i >= latch_idx:
            latch_env = math.exp(-(i - latch_idx) / (SAMPLE_RATE * 0.004))
            latch = math.sin(2.0 * math.pi * 2200.0 * (t - 0.035)) * 0.6 * latch_env
        samples.append((thud * env + latch) * 0.85)
    save_wav("chest_close.wav", samples)


def generate_furnace_crackle():
    # 11. Furnace Ember Crackle (320 ms)
    n = int(SAMPLE_RATE * 0.320)
    samples = []
    pops = [int(SAMPLE_RATE * d) for d in [0.04, 0.11, 0.19, 0.26]]
    for i in range(n):
        t = i / SAMPLE_RATE
        base_hiss = (random.random() * 2.0 - 1.0) * 0.08
        pop_sum = 0.0
        for p in pops:
            if p <= i < p + 400:
                dt = (i - p) / SAMPLE_RATE
                pop_env = math.exp(-dt / 0.003)
                pop_sum += math.sin(2.0 * math.pi * 950.0 * dt) * 0.55 * pop_env
        samples.append(base_hiss + pop_sum)
    filtered = simple_lowpass(samples, 0.55)
    save_wav("furnace_crackle.wav", filtered)


def main():
    print("=" * 60)
    print("MINERUST PROCEDURAL AUDIO SYNTHESIZER")
    print("=" * 60)
    generate_click()
    generate_pop()
    generate_dig_grass()
    generate_dig_stone()
    generate_dig_wood()
    generate_dig_sand()
    generate_break_block()
    generate_step()
    generate_chest_open()
    generate_chest_close()
    generate_furnace_crackle()
    print("=" * 60)
    total_files = len([f for f in os.listdir(AUDIO_DIR) if f.endswith(".wav")])
    total_size = sum(os.path.getsize(os.path.join(AUDIO_DIR, f)) for f in os.listdir(AUDIO_DIR))
    print(f"Total: {total_files} procedural WAV sounds | {total_size / 1024:.1f} KB total")
    print("=" * 60)


if __name__ == "__main__":
    main()
