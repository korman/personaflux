# 行为规格草案

## 状态

本文定义 PersonaFlux 需要独立确定和测试的行为，不复制 Love/Hate 公式。所有“待决定”项目必须在实现和 ABI 冻结前形成 ADR 与测试向量。

## 数值约定

PersonaFlux 使用 IEEE-754 binary32（Rust `f32`）。

- PAD、亲和度、人格特质和行为影响使用 `[-1, 1]`。
- 置信度和攻击性使用 `[0, 1]`。
- 端点是合法值；外部越界值、NaN 和 Infinity 必须拒绝。
- Rust 核心使用语义明确的强类型，C ABI 使用命名明确的 `float` 字段。
- 快照保存小端 binary32，并由版本化 schema 管理。
- `-0.0` 在输入和快照边界规范化为 `+0.0`。

输入和计算规则：

- PAD 和亲和度必须限制在合法区间。
- 外部非法值返回 `PF_INVALID_ARGUMENT`，不自动钳制。
- 有限的内部中间值可以暂时超出范围；内部不采用饱和钳制。
- 内部出现 NaN 或 Infinity，或最终待提交状态超出范围，返回 `PF_INTERNAL_ERROR`，状态和事件队列保持不变。
- 边界、非法值、舍入、精度和失败回滚必须有测试。

## PAD 与情绪区域

`Pad` 包含 pleasure、arousal、dominance。长期幸福感如需要，应使用独立 `well_being`。

- 最终提交时各轴必须位于合法区间；有限的中间计算可以暂时超界，不得用饱和钳制掩盖错误。
- 情绪区域匹配当前 PAD。
- 重叠区域按显式优先级和稳定顺序选择。
- 可选稳定化按逻辑时间向目标回归。

## 关系

关系有方向：`affinity(A, B) != affinity(B, A)`。v1 支持三层关系：

- `Member -> Member`：成员之间的个人关系。
- `Faction -> Member`：阵营对具体成员的制度立场。
- `Faction -> Faction`：阵营之间的制度基线。

关系使用 `Affinity`，显式设置的 `0.0` 是一条真实关系记录，不能与缺失混淆。设置会覆盖同方向已有值；清除会删除记录，使查询恢复为缺失或继续使用 fallback。关系允许自指，且 `A -> B` 与 `B -> A` 独立保存。

对成员 `observer` 查询目标成员 `target` 的有效关系时，按以下固定优先级命中第一条记录：

1. `observer -> target` 的 `Member -> Member`。
2. `observer` 所属阵营 `-> target` 的 `Faction -> Member`。
3. `observer` 所属阵营 `-> target` 所属阵营的 `Faction -> Faction`。
4. 没有记录时返回 `Missing`。

有效查询返回实际命中的关系层级，不进行加权、平均或自动叠加。成员和阵营必须预先存在；未知 ID 是错误，查询不会隐式创建关系。

关系值实际发生变化时，核心按宿主调用顺序加入 `RelationshipChanged` 事件，记录关系层、主体、前值和后值。新增记录的前值为空，覆盖记录的前后值均存在，清除记录的后值为空；同值设置和清除缺失关系是幂等 no-op，不产生事件。

v1 不支持 `Member -> Faction`、父阵营继承或多重父阵营。

## Deed

行为至少包含：

```text
deed_id
actor identity
target identity
kind or tag
impact
aggression
trait vector
logical timestamp
```

宿主提交目击者。核心验证 ID、有限浮点和 trait 维度，并按稳定顺序处理。

批量命令必须先完整校验；任一元素非法时整体不提交，并返回从零开始的失败项索引。无法归因到具体元素的错误使用 `UINT64_MAX`。

### 单个直接目击提交（v1 Rust 核心）

Rust 核心当前通过 `Simulation::submit_direct_witness` 支持单个直接目击行为。输入包含 `deed_id`、`observer`、`actor`、可选 `target`、`impact`、`aggression` 和 `threatens_observer`。允许 `observer == actor`，拒绝 `actor == target`。无目标行为使用 `target_affinity = 0`，并忽略观察者威胁标志。

命令先验证所有 ID，有目标时解析有效关系，然后完成评价和全部有界状态计算，最后一次性提交状态。观察者对行为者的动态亲和度与三层关系配置独立存储，并从 `0.0` 开始；关系值只作为本次评价的输入。PAD 只更新观察者。任何失败都不改变状态或事件队列。

成功提交先产生 `DeedEvaluated`，对应状态实际变化时再依次产生 `AffinityChanged` 和 `PadChanged`，随后按记忆分类产生 `MemoryRemembered` 或 `MemoryUpgraded`。逻辑时间仍推迟到后续阶段。

### 批量直接目击与幂等

`Simulation::submit_direct_witness_batch` 按输入数组顺序处理一组 Deed，并返回与输入一一对应的 `Applied` 或 `Duplicate` 结果；单个 `submit_direct_witness` 也使用同一结果枚举。`(observer, deed_id)` 是单提交和批量提交共享的去重键；同一观察者重复提交时返回 `Duplicate`，不重新评价、不改变状态且不产生事件，不同观察者可以使用相同 `deed_id`。

批量命令先完整验证所有成员 ID 和 `actor == target` 约束，再在工作状态中顺序执行。后续输入可以看到前面已应用项的亲和度和 PAD 更新；任一项失败时，整个批次的状态、事件和去重记录全部回滚，并返回失败项的零基索引。空批量是无副作用成功操作。

## v1 Memory State

The Rust core stores remembered direct-witness deeds under the deterministic key
`(observer, deed_id)`. A record contains the observer, deed, actor, optional target,
the original impact and aggression, salience, and `ShortTerm` or `LongTerm` kind.

Memory salience is `max(abs(impact), aggression)`. Values below `0.40` are not
remembered; `0.40` is the inclusive short-term boundary and `0.75` is the inclusive
long-term boundary. A short-term record may be upgraded to long-term, but records
are never downgraded and the first deed payload remains authoritative.

`Simulation::memory` and `Simulation::memories_for` are read-only queries. Unknown
observers are errors and missing deed keys return `None`; queries never create state.
Memory records are part of the direct-witness transaction. Duplicate deed keys do
not evaluate, update, or emit memory events. Batch failure rolls back memory state,
deduplication, other state, and all pending events together.

For an applied deed, memory events follow the existing state events:

```text
DeedEvaluated
AffinityChanged (if changed)
PadChanged (if changed)
MemoryRemembered or MemoryUpgraded (if changed)
```

This version does not assign logical creation or expiration ticks. The `60` and
`3600` logical-second TTLs, expiry cleanup, and permanent deduplication retention
are implemented with the logical-time state in the next development step.

## Rumor

传闻至少包含原始 deed 身份、行为者、目标、置信度、影响、攻击性、traits、重复次数和过期时间。

同一 deed ID 重复到达不能无限叠加影响；同类但不同 ID 的行为可以应用习惯化。

## 评价输入和输出

输入可考虑：

- 来源信任。
- 对目标的亲和度。
- 行为影响和攻击性。
- 人格契合。
- 当前唤醒度。
- 感知力量差。
- 重复次数和消息置信度。

输出：

```text
affinity_delta toward actor
pad_delta
personality_delta
resulting_confidence
memory decision
explanation factors
```

具体公式待独立设计，但必须满足：

- 不可信来源显著降低传闻影响。
- 目标关系可改变评价方向或幅度。
- 人格、唤醒和力量修正均有界。
- 力量差主要影响 dominance。
- 习惯化不随重复次数增大影响。
- 每个因子可单独测试和解释。

## 记忆

- v1 根据 `max(abs(impact), aggression)` 将直接目击行为分类为短期或长期记忆；低于 `0.40` 不记录，`0.40` 和 `0.75` 分别是包含边界。
- 同一 `(observer, deed_id)` 只保留一条记录，短期可以升级为长期，不降级；原始行为字段保持首次记录内容。
- v1 已提供确定性查询和事务回滚，但逻辑时间、TTL、过期清理和容量淘汰留到逻辑时间阶段。
- 相同 deed ID 去重，同类行为更新重复计数。
- 分享通常来自长期记忆。
- MVP 建议先清理过期项，再淘汰最低重要性；相同时按最早时间和稳定 ID。

## 传闻传播

宿主触发传播，核心不负责相遇判断。待决定单次数量、选择方式、置信度衰减、传播链和循环规则。任何随机选择都使用模拟器受控随机源。

## 事件顺序

建议固定为：

```text
DeedEvaluated
RelationshipChanged
PadChanged
EmotionChanged
PersonalityChanged
RumorRemembered or RumorUpdated
```

事件包含逻辑时间、相关 ID、变化前后值或 delta，以及命令关联 ID。

相同输入、配置、顺序和种子必须保持事件顺序、状态变化方向、记忆决策和其他规定行为稳定。项目不要求不同 CPU 上的浮点结果逐位一致，数值比较使用明确容差。

## 初始测试向量

1. 正面行为作用于喜欢的目标。
2. 负面行为作用于喜欢的目标。
3. 负面行为作用于讨厌的目标。
4. 同一传闻来自可信和不可信来源。
5. 人格完全契合与完全冲突。
6. 高唤醒和低唤醒评价同一事件。
7. 强势和弱势角色面对攻击行为。
8. 同类行为重复 1、2、10、100 次。
9. 低于和高于记忆阈值。
10. 短期和长期记忆过期。
11. 传闻循环与 deed ID 去重。
12. 极值、非法 ID、NaN、Infinity 和空 trait。

每个样例必须给出初始状态、命令、评价因子、状态变化和预期事件。
