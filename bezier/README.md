# Bezier

一个研究贝塞尔曲线的小工具。

## WASM 编译和部署

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen ./target/wasm32-unknown-unknown/release/bezier.wasm --target web --no-typescript --out-dir .
```
