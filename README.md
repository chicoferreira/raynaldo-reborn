# Raynaldo Reborn

## Prerequisites

### Installing Intel Embree

Embree is required for the high-performance ray tracing backend. Install it according to your operating system:

#### Windows
1. Download Embree from [Intel's official releases](https://github.com/embree/embree/releases)
2. Extract the archive and add the `embree/bin` directory to your PATH and `embree/lib` to your LIB environment variable

#### macOS
```bash
brew install embree
```

#### Linux (Ubuntu/Debian)
```bash
# Install development packages
sudo apt update
sudo apt install libembree-dev
```

If you get `stddef.h` errors, try instaling clang.

### Rust Requirements

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Minimum version**: Rust 2024 edition (latest stable recommended)

## Compilation

```bash
# Using a sample scene
cargo run --release -- assets/worlds/cornell_box.toml

# Specify tracer backend explicitly
cargo run --release -- assets/worlds/dragon8k.toml --tracer embree
cargo run --release -- assets/worlds/balls.toml --tracer naive # Very slow
```
