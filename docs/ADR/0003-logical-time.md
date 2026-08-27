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

候选方式：

- `step(delta)`：按增量推进。
- `advance_to(absolute_time)`：推进到绝对逻辑时间。
- 同时提供两者，以绝对时间作为内部事实来源。

需要决定时间倒退时返回错误、忽略还是允许恢复快照后回退。常规模拟实例建议拒绝倒退；快照恢复通过替换整个实例状态实现。

## Questions To Resolve

1. 使用毫秒、微秒还是 tick？
2. C ABI 时间类型是否固定为 `uint64_t`？
3. 同时提供 `step` 和 `advance_to` 吗？
4. `step(0)` 是否触发队列处理？
5. 时间加法溢出返回错误还是饱和？
6. 过期判断使用 `now >= expiration` 还是 `now > expiration`？

## Decision

尚未决定。

## Rationale

待决定后填写。

## Consequences

待决定后填写，并同步更新 `SimTime`、记忆规则、C ABI 和快照 schema。
