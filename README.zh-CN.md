# PersonaFlux

[English](README.md) | [简体中文](README.zh-CN.md)

PersonaFlux 是一个使用 Rust 编写、与游戏引擎无关的社会情感模拟内核。

项目目前处于规格设计和基础框架阶段，计划覆盖 PAD 情感状态、阵营、有向关系、人格特质、行为、记忆、传闻以及确定性事件处理。

## 项目目标

- 模拟连续 PAD 情感状态和可配置情绪区域。
- 表达阵营、成员、人格特质和有向关系。
- 根据信任、关系、人格和上下文评价行为及传闻。
- 管理具有确定性过期规则的短期和长期记忆。
- 与渲染、物理、ECS 和具体游戏引擎解耦。
- 通过稳定 C ABI 服务桌面端、移动端、Unity 和其他语言。

## Workspace

- `personaflux-core`：安全、与引擎无关的 Rust 领域逻辑。
- `personaflux-ffi`：版本化 C ABI，同时构建静态库和动态库。
- `docs`：项目章程、行为规格、架构、FFI 和路线图。

完整设计文档参见 [docs/README.md](docs/README.md)。

## 构建

```sh
cargo build --workspace
cargo test --workspace
cargo build -p personaflux-ffi --release
```

在 Windows 上，FFI crate 会生成 `personaflux.lib` 和 `personaflux.dll`；其他平台会生成对应的 `.a`、`.so` 或 `.dylib` 产物。

## 当前状态

公开行为和 C ABI 尚未稳定。在实现完整模拟模型或冻结 ABI v1 前，需要先解决 `docs/BEHAVIOR_SPEC.md` 中的待决策项。

## 独立实现

PersonaFlux 基于公开的情感计算理论和原创行为规格独立实现，与 Pixel Crushers 无关联。项目不得复制或直接翻译 Love/Hate 的源码、文档、资源或品牌内容。

## 许可证

PersonaFlux 当前采用 MIT License。
