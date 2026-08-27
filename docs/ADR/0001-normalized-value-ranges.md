# ADR 0001: Numeric Value Ranges And Invalid Input

- Status: Proposed
- Date: 2026-08-27
- Decision owners: PersonaFlux maintainers
- Related documents: `docs/BEHAVIOR_SPEC.md`, `docs/C_ABI_AND_FFI.md`

## Context

PersonaFlux 需要统一表示 PAD、亲和度、人格特质、行为影响、攻击性和置信度。这些值会跨越 Rust API、C ABI、移动端绑定、配置、快照和调试工具。

数值单位会直接影响公式可读性、编辑体验、ABI 文档和未来兼容性，因此应在完整领域类型和 C ABI v1 之前决定。

## Decision Drivers

- 公式清晰且不需要重复换算。
- 跨语言和序列化表达简单。
- 能清楚区分有符号值和概率值。
- 非法配置不能被静默掩盖。
- 内部累加可以安全饱和。
- UI 可以按需显示百分比或其他单位。

## Options

### Option A: Normalized Ranges

```text
PAD, affinity, traits, impact, aggression: [-1.0, 1.0]
confidence: [0.0, 1.0]
```

优点：

- 适合乘法权重和向量计算。
- 与概率和归一化数学表达一致。
- 公式不需要反复除以 100。

缺点：

- 游戏策划和 Inspector 可能更习惯百分制。
- 文档必须强调 `0.5` 代表 50%。

### Option B: Percentage-Like Ranges

```text
PAD, affinity, traits, impact, aggression: [-100.0, 100.0]
confidence: [0.0, 100.0]
```

优点：

- 对编辑器和日志更直观。
- 与部分既有游戏系统习惯一致。

缺点：

- 公式需要频繁归一化。
- 更容易漏掉单位换算。
- 概率和权重语义不够自然。

### Option C: Strong Domain Types With Internal Normalization

公开构造器接受指定展示单位，内部统一存储归一化强类型值。

优点：

- 内部公式清晰。
- 可为 UI 提供百分制转换。

缺点：

- API 和 FFI 需要更明确的单位命名。
- 如果同时接受多种单位，误用风险会上升。

## Additional Input Policies

还需决定：

- 外部有限值超出范围时返回错误还是自动钳制。
- 内部状态增量超出范围时是否自动钳制。
- 所有 NaN 和 Infinity 是否统一返回 `InvalidArgument`。
- 核心使用 `f32` 还是 `f64`。
- 快照中是否存储原始浮点或量化整数。

## Questions To Resolve

1. 内部和 C ABI 是否统一采用 `f32` 归一化值？
2. 公共构造输入超界是错误还是钳制？
3. 状态增量是否采用饱和钳制？
4. UI 百分制转换放在语言绑定还是工具层？
5. 是否需要为值创建 `Affinity`、`Confidence` 等强类型？

## Decision

尚未决定。

## Rationale

待决定后填写。

## Consequences

待决定后填写，并同步更新行为规格、C ABI 文档、领域类型和边界测试。
