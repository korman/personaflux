# ADR 0004: MVP Deed And Rumor Evaluation Model

- Status: Proposed
- Date: 2026-08-27
- Decision owners: PersonaFlux maintainers
- Related documents: `docs/AFFECT_THEORY.md`, `docs/BEHAVIOR_SPEC.md`, `docs/C_ABI_AND_FFI.md`, `docs/ARCHITECTURE.md`

## Context

PersonaFlux 的核心价值在于角色如何根据行为、关系和信息来源形成不同评价。MVP 需要一个原创、可解释、确定性且有界的评价模型，同时避免在第一版引入过多参数。

评价模型先定义行为性质和测试方向，再固定精确公式。模型运行在 Rust 核心中，跨语言宿主通过版本化 C ABI 提交命令并读取结果。评价函数应尽量是纯函数；状态、记忆和事件由 `Simulation` 原子提交。

## Decision Drivers

- 同一事件可因观察者立场不同产生不同结果。
- 每个因子可解释和单独测试。
- 所有提交后的状态值有界，非法输入不会被静默钳制。
- 中立关系的行为有明确语义。
- 结果能映射到亲和度和 PAD。
- 直接目击、批量处理、去重和事件顺序可确定。
- v1 的 C ABI 和快照可以长期兼容。
- 未来可增加人格、力量和习惯化，而不改变 v1 的基本管线。

## Scope Of Model Version 1

v1 的模型版本固定为：

```text
model_version       = 1
neutral_target_policy = MoralBaseline
moral_baseline      = 0.30
relationship_weight = 0.70
memory_threshold    = 0.40
long_term_threshold = 0.75
```

`moral_baseline` 和 `relationship_weight` 的合法范围均为 `[0, 1]`，且两者之和不得超过 `1`；记忆阈值使用 `[0, 1]`，并要求 `long_term_threshold >= memory_threshold`。v1 固定值满足这些约束。

策略和上述参数在创建 `Simulation` 时确定，运行期间不可修改，并写入快照。若小游戏或后续产品测试需要改变参数，应发布新的模型版本，不得改变 v1 存档或回放的含义。

v1 只评价直接目击的行为：

```text
effective_confidence = 1.0
```

谣言传播、`source_trust`、人格契合、力量差、习惯化和文化规范不属于 v1 评价路径。相关字段如已存在于数据结构，可以校验、保存和序列化，但不参与 v1 公式。谣言应在后续版本以独立的证据类型和 API 加入。

## Inputs And Semantic Rules

v1 真正参与评价的输入为：

```text
observer identity
deed_id
actor identity
optional target identity
impact
aggression
current affinity toward target
current PAD
logical timestamp
```

字段语义：

- `impact` 使用 `[-1, 1]`。正数表示行为使目标受益，负数表示行为使目标受害，零表示没有实际结果。
- `aggression` 使用 `[0, 1]`，表示攻击性或威胁性，不改变 `impact` 的符号。
- 无明确目标必须通过 ABI 的显式 `has_target = 0` 表示，不能用未约定的特殊 ID。此时 `target_affinity = 0`，仍应用基础道德评价。
- `actor == target` 的行为在 v1 中拒绝，返回 `PF_INVALID_ARGUMENT`。
- `current PAD` 只作为评价上下文和有界状态更新的当前值，不决定亲和度评价方向。
- `trait vector` 在 v1 中保留为预留数据，不参与评价。

## Evaluation Formula

v1 采用带中立基线的有符号关切模型（Option C）：

```text
concern = moral_baseline + relationship_weight * target_affinity
raw_affinity_delta = effective_confidence * impact * concern
```

`moral_baseline = 0.30` 使中立目标也能产生基础道德评价。`relationship_weight = 0.70` 允许对强烈厌恶目标的评价发生反转，同时满足：

```text
moral_baseline + relationship_weight = 1.0
```

因此 `concern` 在 `[-1, 1]` 内。关系项只改变评价的幅度和可能的方向；它不改变行为原始的 `impact`。

### Bounded State Update

亲和度和 PAD 的每个轴都使用剩余空间缩放，将原始增量转换为提交增量：

```text
if raw_delta >= 0:
    applied_delta = raw_delta * (1 - current_value)
else:
    applied_delta = raw_delta * (1 + current_value)

new_value = current_value + applied_delta
```

这是一项显式模型规则，不是错误发生后的隐式饱和钳制。最终值仍必须通过 `[-1, 1]` 校验；出现 NaN、Infinity 或越界时，命令失败且状态、记忆和事件队列保持不变。

## PAD Mapping

PAD 的即时变化使用以下 v1 初始系数：

```text
event_intensity = effective_confidence * max(abs(impact), aggression)

raw_pleasure_delta = 0.5 * raw_affinity_delta
raw_arousal_delta  = 0.5 * event_intensity

raw_dominance_delta =
    -0.4 * aggression  // 行为直接针对观察者
     0.0                // 其他情况，包括攻击第三方和无目标行为
```

- Pleasure 与对行为者的评价同向。
- Arousal 由事件的主观强度驱动，正面和负面事件都可以提高 Arousal。`impact = 0` 但 `aggression > 0` 时，Pleasure 不变而 Arousal 可以上升。
- Dominance v1 不建模力量差；只有行为直接以观察者为目标时，攻击性才降低 Dominance。
- 无目标行为不改变 Dominance。
- PAD 各轴的恢复只在 `step` 或 `advance_to` 中按逻辑时间进行，不在评价函数中隐式发生。初始恢复速度为：

```text
pleasure_rate  = 0.05 / logical_second
arousal_rate   = 0.20 / logical_second
dominance_rate = 0.03 / logical_second
```

恢复方向为向 `0.0` 线性回归；足够长的时间直接回到 `0.0`。恢复速度属于模型版本配置，不能在运行期间修改。

## Direct Witness, Batch, And Idempotency

v1 的直接目击不需要来源字段，且有效置信度固定为 `1.0`。同一行为对不同观察者独立评价：

```text
deduplication key = (observer_id, deed_id)
```

批量目击命令必须先完整校验，再按输入数组顺序处理。任一元素非法或任一最终状态越界时，整个批次原子回滚，并且不产生部分事件。相同观察者重复提交同一 `deed_id` 是幂等空操作，不重复施加亲和度、PAD 或记忆影响。

## Memory

直接目击的记忆显著性为：

```text
memory_salience = effective_confidence * max(abs(impact), aggression)
```

v1 的分类规则为：

```text
salience < 0.40:
    不记录

0.40 <= salience < 0.75:
    短期记忆

salience >= 0.75:
    长期记忆
```

达到阈值（包括等于阈值）即记录。低于阈值的事件仍可改变亲和度和 PAD，但不进入记忆。`impact = 0` 的高攻击性行为可以作为威胁记忆；`impact = 0` 且 `aggression = 0` 不记录。

v1 的初始过期配置为：

```text
short_term_ttl = 60 logical seconds
long_term_ttl  = 3600 logical seconds
```

同一 `(observer_id, deed_id)` 只保留一条记忆：

- 重复到达不能再次施加完整评价。
- 记忆显著性提高时，可以从短期升级为长期；升级后不降级。
- 同类但不同 `deed_id` 的行为可以增加重复计数，但 v1 不因重复次数增加评价或 PAD 影响。
- 去重记录不能因为活动记忆过期而删除，否则同一 deed 可能再次生效。

## Rejected Alternatives

### Option A: Direct Multiplicative Model

```text
affinity_delta = effective_confidence * impact * target_affinity
```

该模型参数少且天然支持关系反转，但中立目标的所有行为影响为零，不能表达基础道德评价。

### Option B: Baseline Concern Model

```text
target_concern = neutral_concern + relationship_factor
```

该模型允许中立目标产生评价，但没有明确分离道德基线与关系立场，反转条件和极值语义不够清楚。

### Option D: Staged Appraisal

```text
Appraisal {
    credibility,
    desirability,
    norm_alignment,
    threat,
    control,
}
```

该模型扩展性和解释性最好，但会在 MVP 引入更多参数、测试和跨语言兼容负担。v1 保留未来分阶段评价的管线位置，不冻结这些字段为 C ABI 合同。

## Decision

PersonaFlux v1 采用 Option C 的有符号关切模型，默认开启基础道德评价，使用本 ADR 规定的固定参数、PAD 映射、边界更新、直接目击、批量原子性和记忆去重规则。

`model_version` 在 C ABI 查询和快照中返回 `1`。策略和参数创建后不可修改。v1 不实现谣言传播 API；后续版本应通过新增版本化 API 和模型版本加入谣言及其他评价因子。

## Rationale

Option C 在参数量可控的前提下同时表达三种必要语义：中立目标的基础道德评价、关系对评价幅度的影响，以及对强烈厌恶目标的可能反转。将直接目击限定为固定置信度、将物理影响预先归一化为 `impact`、并把攻击性主要放入 PAD，可以让 Rust 核心保持纯函数和确定性，也避免把游戏引擎、平台或宿主语言的规则泄漏到核心。

固定模型版本、快照参数和显式 ABI 语义，使 iOS、Android、C# 及其他宿主可以长期回放同一行为；未来新增人格、力量、习惯化或谣言时，可以通过新策略和新版本演进，而不改变 v1 的既有结果。

## Consequences

正面影响：

- 评价方向和每个因子都有可解释定义，可建立固定测试向量。
- 中立目标不会被静默视为“无意义”，关系反转也有明确阈值。
- 直接目击路径简单，批量提交、去重、事件顺序和失败回滚可以稳定测试。
- PAD、亲和度和记忆均有明确边界和逻辑时间语义。
- C ABI 只需承诺模型版本和行为，不暴露 Rust 内部布局或未成熟的解释字段。

代价和限制：

- `moral_baseline` 与 `relationship_weight` 的组合会产生固定的关系反转阈值；这属于 v1 世界观语义。
- v1 不表达来源可信度、谣言衰减、人格、力量差和习惯化。
- 去重记录不随记忆过期删除，需要容量或持久化策略在后续版本单独设计。
- PAD 恢复速度、记忆阈值和 TTL 是初始模型参数，可能需要小游戏验证后在新模型版本中调整。

## Evidence Required Before Accepting

在把状态改为 `Accepted` 前，需要完成以下证据：

1. 由维护者确认的方向测试向量，覆盖喜欢、中立、讨厌目标、无目标行为、自我目标拒绝以及 `impact = 0` 的高攻击性行为。
2. 固定数值和边界测试，覆盖剩余空间缩放、极值输入、NaN/Infinity、失败回滚和最终状态有界性。
3. 多人目击测试，验证每个观察者独立评价、输入顺序稳定和 `(observer_id, deed_id)` 幂等去重。
4. 记忆测试，验证阈值包含关系、短期/长期分类、升级、不降级、过期和去重记录保留。
5. Rust 核心与 C ABI 测试，验证 `model_version = 1`、结构体版本、事件顺序、快照导入导出和跨平台数值容差。
6. 使用 Bevy、Godot 或其他宿主建立最小场景，确认 PAD 恢复速度、记忆 TTL 和阈值符合实际体验；体验调整应产生新的模型版本。
