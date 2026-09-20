# MineRust ⛏️🦀

Un clone di Minecraft scritto in Rust, creato per fini didattici, diletto e per esplorare le potenzialità della grafica 3D e della programmazione concorrente e ad alte prestazioni con Rust.

---

## 🎯 Obiettivi dell'MVP

Un MVP (Minimum Viable Product) per un voxel game stile Minecraft include tipicamente:

1. **Gestione del Mondo & Chunk**:
   - Struttura dati per blocchi (`Air`, `Dirt`, `Grass`, `Stone`, `Wood`, ecc.).
   - Struttura a Chunk (es. $16 \times 16 \times 16$ o $16 \times 256 \times 16$).
   - Generazione procedurale di base (es. terreno semplice o rumore Perlin/Simplex).

2. **Meshing & Rendering**:
   - **Culling delle facce nascoste (Hidden Face Removal)**: non generare triangoli per le facce a contatto tra due blocchi solidi.
   - **Shading base**: illuminazione direzionale o colori/texture per faccia.
   - **Greedy Meshing** (opzionale/fase successiva) per comprimere i vertici.

3. **Camera & Movimento First-Person**:
   - Camera FPS con controlli WASD + Spazio/Shift.
   - Mouse lock per la visuale (Pitch & Yaw).

4. **Interazione**:
   - Raycasting / Voxel traversal (algoritmo di Amanatides & Woo) per selezionare il blocco mirato.
   - Rimozione (tasto sinistro) e posizionamento (tasto destro) di blocchi.

---

## 🏗️ Architettura Consigliata

```text
minerust/
├── assets/             # Shaders, texture e font
├── src/
│   ├── main.rs         # Entry point e loop dell'applicazione
│   ├── camera.rs       # Gestione visuale, proiezioni e controlli FPS
│   ├── world/          # Logica del mondo voxel
│   │   ├── mod.rs
│   │   ├── block.rs    # Tipi di blocco e proprietà
│   │   ├── chunk.rs    # Dati dei voxel per chunk (es. array 16^3)
│   │   └── terrain.rs  # Generatore di terreno (noise)
│   ├── mesh/           # Algoritmi di meshing per la GPU
│   │   ├── mod.rs
│   │   └── chunk_mesh.rs
│   ├── renderer/       # Pipeline grafica (WGPU o Engine)
│   │   └── mod.rs
│   └── input.rs        # Gestione input tastiera e mouse
└── Cargo.toml
```

---

## 🚀 Esecuzione

```bash
# Esecuzione in modalità debug
cargo run

# Esecuzione ottimizzata (consigliata per voxel generation/meshing)
cargo run --release
```
