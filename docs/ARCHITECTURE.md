# Rust 核心架构

## 总体结构

PersonaFlux 保留领域行为，重新设计运行时：

```text
Host application
    | Commands / Queries / Events
    v
C ABI or native Rust API
    v
Simulation
    |-- FactionStore
    |-- MemberStore
    |-- RelationshipStore
    |-- MemoryStore
    |-- EvaluationPolicy
    |-- LogicalClock
    `-- EventQueue
```

## Crate 边界

```text
personaflux-core
    safe Rust domain model and simulation

personaflux-ffi
    C ABI, pointers, handles, validation, errors, panic containment
```

核心禁止依赖引擎、平台或 FFI。FFI 不包含业务规则。

## 集中式模拟上下文

```rust
pub struct Simulation {
    factions: FactionStore,
    members: MemberStore,
    relationships: RelationshipStore,
    memories: MemoryStore,
    config: SimulationConfig,
    clock: SimTime,
    events: VecDeque<SimulationEvent>,
}
```

外部只持有稳定 `FactionId`、`MemberId` 等 ID，不使用对象地址作为身份，也不使用大量 `Rc<RefCell<T>>` 模仿 C# 对象图。

## 关系存储

关系是有向图，建议独立存储：

```text
A -> B: +0.70
B -> A: -0.20
```

必须明确阵营关系、成员关系和成员到阵营关系如何叠加，不能用模糊“个人亲和度”隐藏不同层级。

## 命令、查询和事件

```text
Command
    -> validation
    -> immutable context
    -> pure Evaluation
    -> atomic apply
    -> ordered events
```

评价应尽量是纯函数，返回亲和度、PAD、人格、记忆和置信度变化，再由 `Simulation` 统一提交。这有利于测试、审计、回放和 Rust 借用管理。

数值使用 IEEE-754 binary32。PAD、亲和度、人格特质和行为影响为 `[-1, 1]`，置信度和攻击性为 `[0, 1]`。核心使用语义明确的强类型值，不把不同含义混用为裸 `f32`。有限的内部中间值可以暂时超界，但最终提交值必须合法；内部出现 NaN、Infinity 或最终越界时命令失败，不进行饱和钳制，也不改变状态或事件队列。

## 感知和时间边界

- 核心不做物理查询；宿主提交目击者 ID。
- 核心不读取系统时钟；宿主调用 `step` 或 `advance_to`。
- 内部时间使用整数 tick、毫秒或微秒。
- 时间倒退必须产生明确错误。

## 策略扩展

第一版使用数据化 `EvaluationPolicy`，不从 Rust 深层回调 Swift、JVM 或 C#。未来如需自定义评价，可采用“导出上下文、宿主计算覆盖、核心验证提交”的两阶段 API。

## 确定性

- 随机种子显式提供。
- 事件处理和提交顺序固定。
- 不依赖 `HashMap` 迭代顺序。
- 统一浮点精度。
- 快照包含格式和模型版本。
- 并行计算仍按固定顺序提交。

快照浮点使用 little-endian binary32，`-0.0` 规范化为 `+0.0`。

项目只承诺行为级确定性，不承诺所有 CPU 的逐位浮点一致。

## 性能

- 提供批量命令和查询。
- 避免字段级 FFI。
- 记忆容量和过期处理有预算。
- 先建立基准和确定性规格，再考虑并行化。
