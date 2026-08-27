# PersonaFlux 设计文档

本目录固化 PersonaFlux 的项目目标、理论基础、独立实现边界、Rust 架构、行为规格、C ABI 和跨平台计划。

## 文档导航

1. [PROJECT_CHARTER.md](PROJECT_CHARTER.md)：项目定位、目标、非目标和成功标准。
2. [LOVEHATE_ANALYSIS.md](LOVEHATE_ANALYSIS.md)：前期参考项目的功能、规模和架构观察。
3. [AFFECT_THEORY.md](AFFECT_THEORY.md)：PAD 理论、气质映射和认知评价式设计。
4. [CLEAN_ROOM_AND_LEGAL.md](CLEAN_ROOM_AND_LEGAL.md)：独立实现、版权和品牌边界。
5. [ARCHITECTURE.md](ARCHITECTURE.md)：Rust 核心、数据模型和处理流程。
6. [BEHAVIOR_SPEC.md](BEHAVIOR_SPEC.md)：待实现的行为规则、公式约束和测试基线。
7. [C_ABI_AND_FFI.md](C_ABI_AND_FFI.md)：C ABI、内存所有权、错误和版本策略。
8. [PLATFORMS_AND_BINDINGS.md](PLATFORMS_AND_BINDINGS.md)：桌面、移动端和语言绑定。
9. [NAMING.md](NAMING.md)：项目、crate、库文件和公开符号命名。
10. [ROADMAP.md](ROADMAP.md)：里程碑、工作量和主要风险。

## 当前共识

- 复刻领域行为，不照搬 Unity 技术架构。
- `personaflux-core` 是无引擎、无 FFI 的安全 Rust 库。
- `personaflux-ffi` 提供稳定 C ABI，同时输出静态库与动态库。
- 宿主提供时间、目击者、视线和空间查询结果。
- 核心采用命令、查询和事件队列，第一版不使用跨语言回调。
- 第一版覆盖 PAD、阵营、有向关系、人格、行为、记忆、传闻和信任传播。
- PersonaFlux 是独立实现，不复制或直接翻译 Love/Hate 源码。

## 开发前提

行为实现前必须补齐 [BEHAVIOR_SPEC.md](BEHAVIOR_SPEC.md) 中的待决策项。ABI v1 冻结前必须以 C、C#、Swift 和 Kotlin 的实际调用验证接口形态。
