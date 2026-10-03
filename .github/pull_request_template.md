## 📋 Summary
<!-- Provide a concise description of the purpose of this Pull Request and the problem it solves. -->

## 🏷️ Type of Change
<!-- Please check the relevant options: -->
- [ ] 🚀 **Feature** (non-breaking change which adds new functionality)
- [ ] 🐛 **Bug Fix** (non-breaking change which fixes an issue or regression)
- [ ] ⚡ **Performance Optimization** (rendering, meshing, streaming, memory, or ECS tuning)
- [ ] ♻️ **Refactoring** (code structure improvement with no functional changes)
- [ ] 🛠️ **CI/CD & Build System** (GitHub Actions, build profiles, dependencies)
- [ ] 📝 **Documentation** (README, architecture, comments)

## 🧩 Architectural / Implementation Details
<!-- Highlight key technical changes, design decisions, or non-obvious implementations. -->
- 

## 📊 Performance & Graphics Telemetry (if applicable)
<!-- Detail any performance changes, framerate benchmarks, memory footprints, or vertex reduction metrics. -->
- **Tested Render Distance**: [e.g. 16 / 32 / 64 chunks]
- **Average FPS / 1% Low**: [e.g. 140 FPS / 85 FPS]
- **Geometry & Memory**: [e.g. vertices, Peak RAM, Peak VRAM]
- **Hardware In-Game Benchmark Verdict**: [e.g. PERFECT / GREAT / N/A]

## ✅ Quality Checklist
<!-- Ensure all project standards are met before requesting review: -->
- [ ] Code is formatted cleanly with `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings` reports **0 warnings**
- [ ] `cargo test` passes with **100% success rate** (all unit & property tests pass)
- [ ] 100% Safe Rust preserved (`#![forbid(unsafe_code)]`)
- [ ] All modified source files remain strictly **under 1,000 LOC**
- [ ] No regression on water transparency, seabed textures, or player physics
