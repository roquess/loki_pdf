# Loki PDF

![Logo](https://github.com/roquess/loki_pdf/raw/main/logo.png)

A lightweight and performant PDF compression library for WebAssembly, built with Rust.

## Compression Levels

### Light
- Basic stream compression
- **Image Quality:** 90%
- **Metadata:** Preserved
- **Best for:** Documents where quality is critical

### Medium (Default)
- Stream compression
- Metadata removal
- Image compression
- Duplicate removal
- **Image Quality:** 75%
- **Best for:** General use, balanced compression

### High
- All Medium optimizations
- Font optimization
- Aggressive compression
- **Image Quality:** 60%
- **Best for:** Maximum file size reduction

## Installation

### Prerequisites

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or with cargo
cargo install wasm-pack
```

### Build

```bash
# Build for web (ES modules)
wasm-pack build --target web

# Build for Node.js
wasm-pack build --target nodejs

# Build for bundler (webpack, rollup, etc.)
wasm-pack build --target bundler

# Optimized production build
wasm-pack build --release --target web
```

### Running

Serve the files with any HTTP server:

```bash
python3 -m http.server 8080
```

Then open `http://localhost:8080` in your browser.

The `index.html` automatically loads the WASM module.

## License

MIT

## Useful Links

- [wasm-bindgen documentation](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [lopdf documentation](https://docs.rs/lopdf/)

