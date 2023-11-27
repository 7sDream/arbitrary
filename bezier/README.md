# Bezier

一个研究贝塞尔曲线的小工具。

## WASM 编译和部署

```bash
cargo build --target wasm32-unknown-unknown --example editor --release
wasm-bindgen ./target/wasm32-unknown-unknown/release/examples/editor.wasm --target web --no-typescript --out-dir .
```

## TODO

- [ ] 角点变平滑点时、新增控制点时尽量保证曲线不变
- [ ] 不显示控制点
- [ ] 颜色等配置
- [ ] 导入导出
- [ ] 扫描线算法填充
- [ ] 最近点算法优化
- [ ] 文档
