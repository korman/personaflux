# 项目章程

## 项目摘要

PersonaFlux 是使用 Rust 编写的开源、可嵌入社会情感模拟内核，面向游戏、交互角色、仿真系统和服务端代理。它通过稳定 C ABI 服务 C、C++、C#、Swift、Kotlin、Python、Unity 和移动应用，不绑定任何游戏引擎。

> PersonaFlux is a deterministic social and emotional simulation core for interactive characters.

## 产品目标

1. 提供纯 Rust 的社会情感领域模型和模拟器。
2. 支持 PAD 连续情绪和可配置情绪区域。
3. 支持阵营、成员、人格特质和有向关系。
4. 支持行为、目击、评价、记忆和传闻传播。
5. 相同输入、配置、顺序和种子产生可重复结果。
6. 提供快照、持久化、回放和调试能力。
7. 提供稳定、版本化、可长期维护的 C ABI。
8. 支持 Windows、Linux、macOS、iOS 和 Android。
9. 为主流语言提供薄且符合语言习惯的包装层。

## 非目标

核心不负责：

- 渲染、UI、动画、碰撞和射线。
- 寻路、角色移动和场景生命周期。
- Unity、Bevy、Godot 或其他 ECS 集成。
- 自动对话、通用人工智能或意识系统。
- Love/Hate API、源码或存档格式兼容。

宿主负责逻辑时间、行为事件、目击者、感知和空间查询。

## MVP 范围

初始领域对象：

```text
Simulation
Faction / FactionId
Member / MemberId
Relationship
PersonalityTraits
Pad / EmotionRegion
Deed / Rumor / Memory
Evaluation
SimulationEvent
SimulationConfig
```

初始能力：

- PAD 修改、限制和区域映射。
- 阵营与成员的有向亲和关系。
- 行为影响、攻击性和人格特质。
- 来源信任、目标亲和度和人格契合评价。
- 唤醒度、力量差和重复行为习惯化。
- 短期、长期记忆和逻辑过期。
- 传闻置信度、去重和传播。
- 命令、查询、事件和版本化快照。

## 工程原则

- 核心与 FFI 分离。
- 使用稳定 ID，不向外暴露 Rust 地址。
- 评价尽量为纯函数，状态由模拟器统一提交。
- 对外 API 使用粗粒度和批量操作。
- 时间与随机性显式注入。
- 公开行为必须有规格和测试。
- ABI 所有权、布局、错误和版本必须文档化。

## MVP 成功标准

- 完成 `Deed -> Evaluation -> PAD/Relationship -> Memory -> Events` 闭环。
- 固定测试向量在支持的平台稳定通过。
- C 程序可创建模拟、添加实体、提交行为并读取事件。
- C ABI 无 panic 越界、跨分配器释放和悬垂内部指针。
- Windows、Linux、macOS、iOS 和 Android 可构建。
- 至少提供 C# 和一个移动端包装示例。
