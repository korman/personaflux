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
- 内部计算不会以静默钳制掩盖错误。
- UI 可以按需显示百分比或其他单位。

## Options

以下选项是讨论阶段的候选方案；最终约束以本 ADR 的 `Decision` 部分为准。

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

公开构造器接受归一化值，内部统一存储归一化强类型值；展示单位转换由绑定或工具层显式完成。

优点：

- 内部公式清晰。
- 可为 UI 提供百分制转换。

缺点：

- API 和 FFI 需要更明确的单位命名。
- 如果同时接受多种单位，误用风险会上升。

## Additional Input Policies

本 ADR 对输入和存储策略作如下约定：

- 外部有限值超出范围时返回错误，不自动钳制。
- 有限的内部中间计算结果可以暂时超出范围。
- 内部计算或最终状态出现 NaN、Infinity，或最终状态超出范围时，命令失败，不改变状态或事件队列。
- 外部 NaN、Infinity 和越界值统一返回 `PF_INVALID_ARGUMENT`。
- 核心和 C ABI 统一使用 IEEE-754 binary32（Rust `f32`，C ABI `float`）。
- 快照存储明确格式的 binary32，采用小端字节序和版本化 schema。
- 输入和快照中的 `-0.0` 规范化为 `+0.0`。

## Questions To Resolve

1. 已决定使用统一的 f32/binary32 归一化值。
2. 已决定公共构造输入超界返回错误，不自动钳制。
3. 已决定不采用内部饱和钳制；最终状态非法时命令失败并保持原子性。
4. 已决定 UI 百分制转换放在语言绑定或工具层，并且必须显式表示单位。
5. 已决定 Rust 核心为 `Affinity`、`Confidence`、`PadValue` 等值使用语义明确的强类型；C ABI 使用命名明确的 `float` 字段。
6. 已决定批量命令先完整校验，失败时返回从零开始的失败项索引；非元素级错误使用 `UINT64_MAX`。
7. 已决定只承诺行为级确定性，不承诺跨 CPU 的浮点逐位一致。

## Decision

### Numeric representation

- `PAD`、`affinity`、`traits`、`impact` 使用 `[-1.0, 1.0]`。
- `confidence`、`aggression` 使用 `[0.0, 1.0]`。
- 所有范围端点都是合法值。
- Rust 核心使用 IEEE-754 binary32，并通过语义明确的强类型区分不同数值含义。
- C ABI 使用语义明确、底层类型为 `float` 的字段；不暴露 Rust 类型布局。

### Validation and failure behavior

- 外部输入必须是有限值并位于对应范围内；否则返回 `PF_INVALID_ARGUMENT`。
- 有限的内部中间值可以暂时超出范围。
- 内部出现 NaN 或 Infinity，或最终待提交状态超出合法范围时，返回 `PF_INTERNAL_ERROR`。
- 内部不进行饱和钳制，不用静默修正掩盖计算错误。
- 失败命令不得改变模拟状态或事件队列。
- 批量命令先完整校验，任一元素失败则整体不提交，并返回从零开始的失败项索引；无法归因到元素的错误返回 `UINT64_MAX`。

### Serialization and determinism

- 快照保存明确格式的 IEEE-754 binary32，采用小端字节序和版本化 schema，不直接序列化 Rust 内部布局。
- `-0.0` 在输入和快照边界规范化为 `+0.0`。
- 项目只承诺行为级确定性：相同输入、配置、顺序和种子应产生一致的事件顺序、状态变化方向、记忆决策和其他规定行为；数值比较使用明确容差，不要求所有 CPU 逐位一致。

## Rationale

归一化范围使评价公式、权重和跨语言数据表达保持一致，并明确区分有符号领域值与概率/强度值。binary32 能直接对应 C、C# 和移动平台常用的单精度类型，降低 ABI 和传输复杂度。强类型防止 Rust 核心混用不同语义的裸浮点；百分制只在绑定或工具层显式转换。

外部输入错误直接返回，避免静默钳制掩盖调用方问题。内部允许有限中间值暂时超界，以免正常的加权和中间计算被不必要地拒绝；只有非有限结果或最终状态越界才使命令失败。批量原子提交保证失败不会留下部分状态或事件。版本化 little-endian binary32 快照避免依赖 Rust 内部布局，并为后续迁移保留空间。

## Consequences

领域类型、公式、C ABI 字段和快照 schema 必须遵守上述单位和精度。实现必须覆盖端点、端点外一 ulp、NaN、Infinity、负零、内部最终越界、批量失败索引、原子回滚、快照往返和跨平台行为级回放测试。若未来需要逐位跨平台一致性或更高精度，必须通过新的 ADR 演进 ABI 和快照策略；本 ADR 仍为 `Proposed`，直到项目维护者完成接受流程。
