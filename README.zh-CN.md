# PersonaFlux

[English](README.md) | [简体中文](README.zh-CN.md)

PersonaFlux 是一个使用 Rust 编写、与游戏引擎无关的社会情感模拟内核。

当前实现已经提供确定性的 v1 核心，覆盖规范化数值、阵营、有向关系、
直接目击 Deed、PAD 状态、记忆、逻辑时间和事件处理。同时提供版本化的
C ABI v0，以及面向外部应用和小游戏原型的 C#、Swift、Kotlin 薄绑定。

## 项目目标

- 模拟连续 PAD 情感状态和可配置情绪区域。
- 表达阵营、成员、人格特质和有向关系。
- 按固定的 v1 关系和情感规则评价直接目击 Deed。
- 管理具有确定性过期规则的短期和长期记忆。
- 与渲染、物理、ECS 和具体游戏引擎解耦。
- 通过版本化 C ABI v0 服务桌面端、移动端、Unity 和其他语言。

后续模型版本可以加入人格、传闻、来源可信度和其他评价因子，同时保持
v1 合同的含义不变。

## Workspace

- `personaflux-core`：安全、与引擎无关的 Rust 领域逻辑。
- `personaflux-ffi`：版本化 C ABI，同时构建静态库和动态库。
- `bindings/csharp`：使用 `SafeHandle` 的 `netstandard2.0` P/Invoke 包装。
- `bindings/swift`：Swift Package 及 iOS XCFramework 构建脚本。
- `bindings/kotlin`：带 JNI 桥接和 AAR 构建脚本的 Kotlin/Android 包装。
- `docs`：项目章程、行为规格、架构、FFI 和路线图。

完整设计文档参见 [docs/README.md](docs/README.md)。

## 构建

```sh
cargo build --workspace
cargo test --workspace
cargo build -p personaflux-ffi --release
```

在 Windows 上，FFI crate 会生成 `personaflux.lib` 和 `personaflux.dll`；其他平台会生成对应的 `.a`、`.so` 或 `.dylib` 产物。

## 语言绑定

ABI v0 绑定位于 `bindings/csharp`、`bindings/swift` 和 `bindings/kotlin`。
平台构建要求参见 `docs/PLATFORMS_AND_BINDINGS.md`。C# 包装可使用
`dotnet build bindings/csharp/PersonaFlux.csproj` 构建；Swift/iOS 和
Android 打包需要对应的 Apple 或 Android 工具链。C 头文件和线协议参见
`docs/C_ABI_AND_FFI.md`。

## 当前状态

Rust v1 核心和 C ABI v0 已可供外部原型和小游戏调用。C# 已有本机集成
冒烟测试；Swift/iOS 和 Android/Kotlin 仍需要对应平台工具链或 CI 完成
完整验证。ABI v0 还不是长期兼容冻结版本，跨平台验证后需要由维护者单独
评审 ABI v1。快照、传闻、人格和具体引擎集成仍属于后续工作，ADR 状态仍
为 `Proposed`。

## 独立实现

PersonaFlux 基于公开的情感计算理论和原创行为规格独立实现，与 Pixel Crushers 无关联。项目不得复制或直接翻译 Love/Hate 的源码、文档、资源或品牌内容。

## 许可证

PersonaFlux 当前采用 MIT License。
