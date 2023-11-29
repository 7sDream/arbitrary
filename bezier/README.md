# Bezier

一个研究贝塞尔曲线的小工具。

## WASM 编译和部署

```bash
cargo build --target wasm32-unknown-unknown --example editor --release
wasm-bindgen ./target/wasm32-unknown-unknown/release/examples/editor.wasm --target web --no-typescript --out-dir .
```

## TODO

- [ ] 配置重置
- [ ] 配置保存
- [ ] 窗口菜单
- [ ] 保持方向，只移动长度的模式
- [ ] 保持长度，只移动方向的模式
- [ ] 角点变平滑点时，新增控制点时尽量保证曲线不变
- [ ] 曲线导入导出（SVG Path）
- [ ] 最近点算法优化
- [ ] 扫描线算法填充
- [ ] 文档
- [ ] 重命名 alley
