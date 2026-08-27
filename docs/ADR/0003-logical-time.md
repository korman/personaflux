# ADR 0003: Logical Time Representation And Advancement

- Status: Proposed
- Date: 2026-08-27
- Decision owners: PersonaFlux maintainers
- Related documents: `docs/ARCHITECTURE.md`, `docs/BEHAVIOR_SPEC.md`, `docs/C_ABI_AND_FFI.md`

## Context

记忆过期、习惯化、传闻传播和状态稳定化都依赖时间。PersonaFlux 必须适用于游戏帧、服务端 tick、暂停、加速、回放和移动端恢复，因此不能读取系统墙钟。

时间表示一旦进入 C ABI 和快照格式，改变成本很高。

## Decision Drivers

- 确定性和可回放。
- C ABI 与多语言表达稳定。
- 不依赖平台时钟精度。
- 可以暂停、加速和离线推进。
- 算术溢出和时间倒退有明确行为。

## Options

### Option A: Integer Milliseconds

使用单调递增 `u64` 毫秒。

优点：跨语言简单、精度足够、快照稳定。

缺点：不适合亚毫秒模拟；需要定义溢出行为。

### Option B: Integer Microseconds

使用 `u64` 微秒。

优点：精度高，仍为整数。

缺点：对社会情感模拟通常没有必要，数值增长更快。

### Option C: Simulation Ticks

使用 `u64` tick，tick 时长由配置定义。

优点：适合固定步长和确定性游戏循环。

缺点：不同模拟配置的时间值不能直接比较，绑定层需要转换。

### Option D: Floating-Point Seconds

使用 `f64` 秒。

优点：API 直观，可表达小数秒。

缺点：累积误差和跨平台一致性较难说明，NaN/Infinity 增加验证负担。

## Advancement API

采用两种推进形式：

- `step(delta_ticks)`：按增量推进 tick。
- `advance_to(absolute_tick)`：推进到绝对逻辑 tick。
- 两者同时提供，以绝对 tick 作为模拟状态中的唯一时间事实来源。

`step(delta_ticks)` 必须先以 checked addition 计算目标 tick，再使用与
`advance_to` 相同的推进路径。`step(0)` 和 `advance_to(current_tick)` 都是
严格 no-op：不推进时间、不触发队列处理、不生成事件，也不改变状态。

常规模拟实例拒绝时间倒退并返回错误。快照恢复通过替换完整实例状态实现，
而不是让活动实例回退。允许从当前 tick 直接推进到更大的目标 tick；不要求
一次大步推进与多个小步推进得到相同结果，推进调用序列属于回放输入的一部分。

同一 tick 内的多个命令按照宿主提交顺序处理；批量命令先完整校验，再原子提交。
命令内部的事件顺序遵循 `BEHAVIOR_SPEC.md` 规定的固定顺序，不能依赖
`HashMap` 迭代顺序。

## Resolved Questions

1. 使用单调递增的 `u64` tick，不使用毫秒、微秒或浮点秒作为核心时间类型。
2. C ABI 时间字段和参数固定为 `uint64_t`，Rust 对应 `u64`。
3. 同时提供 `step(delta_ticks)` 和 `advance_to(absolute_tick)`。
4. `step(0)` 和推进到当前 tick 都是严格 no-op。
5. 时间加法溢出返回错误；不饱和、不回绕，失败时状态和事件队列保持不变。
6. 过期判断使用 `now >= expiration`，过期区间为 `[created, expiration)`。

## Decision

PersonaFlux 的规范逻辑时间是从模拟实例创建时开始的单调递增 `u64` tick，
初始值为零，不代表 Unix epoch 或任何系统墙钟时间。核心行为使用 tick 数
表达推进、过期、习惯化、传闻传播和状态稳定化。

C ABI 中所有公开的时间值使用固定宽度 `uint64_t`；不得使用 `time_t`、
`size_t`、平台相关的 `unsigned long` 或浮点数表示逻辑时间。C#、Swift
和其他绑定应分别映射到其固定宽度的无符号整数类型；Kotlin/JNI 包装层
必须明确处理 C 无符号值的表示。

实例可以配置一个固定的 tick duration（例如一个 tick 代表一秒或一分钟），
用于把人类可读的时长转换为 tick。该配置是实例和回放语义的一部分，创建后
不可改变，并随快照保存。模拟速度由宿主每次推进多少 tick 表示，不通过修改
tick duration 实现。

时间推进使用 `step(delta_ticks)` 或 `advance_to(absolute_tick)`。目标 tick
小于当前 tick 时返回错误；checked addition 溢出时返回错误。所有失败的推进
调用都是原子的，不改变模拟状态和事件队列。大跨度推进是允许的，但具体的
时间相关工作必须保持确定的处理顺序并满足资源预算。

## Rationale

无量纲 tick 能同时表达回合制、固定步长游戏、服务端批处理、暂停、加速、
回放和移动端恢复，而不把核心绑定到现实时间或平台时钟。tick duration
作为不可变配置保留了把 tick 映射为秒、分钟等业务时长的能力，同时避免在
运行中重新解释既有过期时间和速率。

`u64` 为高频 tick、毫秒级使用和长期运行提供充足范围，且在 Rust、C ABI、
iOS、Android 和 C# 之间有稳定的固定宽度表达。整数时间避免了浮点累计误差、
NaN/Infinity 验证和跨语言浮点序列化差异。

同时提供增量和绝对推进形式，分别适合回合/动作驱动和回放/恢复场景；统一
推进路径可以避免两套状态变更规则。严格 no-op、拒绝倒退、checked overflow
和半开过期区间使边界行为可测试且不会静默修复宿主错误。

## Consequences

核心和配置中的持续时间必须明确使用 tick；秒、分钟等外部时长必须通过
明确且确定的转换得到 tick，不能隐式混用单位。不同 tick duration 的模拟
实例，其 tick 数值不能直接比较；快照和回放必须保存 tick duration、当前 tick、
schema/model 版本以及未读取的事件队列。若要求恢复后完全连续，还必须保存
随机源状态和稳定的命令/事件序号。

暂停通过不推进 tick 实现，加速通过一次推进更多 tick 实现，离线恢复可以
直接跳到目标 tick。由于不要求分块推进等价，回放必须保留实际的推进调用
边界和命令顺序。

实现落地时还需同步更新 `SimTime`、记忆规则、C ABI 头文件、绑定文档和快照
schema，并为倒退、溢出、零推进、过期边界、同 tick 顺序和大跨度推进补充
跨平台测试向量。
