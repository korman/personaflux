# ADR 0002: Relationship Levels And Composition

- Status: Proposed
- Date: 2026-08-27
- Decision owners: PersonaFlux maintainers
- Related documents: `docs/BEHAVIOR_SPEC.md`, `docs/ARCHITECTURE.md`, `docs/C_ABI_AND_FFI.md`

## Context

PersonaFlux 需要表达阵营之间的长期关系，也可能需要表达成员之间的个人关系。必须明确支持哪些主体组合，以及多层关系同时存在时如何计算有效亲和度。

如果语义不清，多个成员共享阵营时可能意外共享个人态度，或者同一查询在不同绑定中产生不一致结果。

## Decision Drivers

- 关系始终有方向。
- 查询结果容易解释和调试。
- MVP 实现规模可控。
- 不阻塞未来增加更丰富的社会层级。
- 存储与 Rust 借用模型简单。
- FFI 不暴露模糊主体类型。

## Subject Combinations

候选关系包括：

```text
Faction -> Faction
Member -> Member
Member -> Faction
Faction -> Member
```

MVP 一等支持 `Faction -> Faction`、`Faction -> Member` 和 `Member -> Member`；
`Member -> Faction` 推迟到后续 ADR。

## Options

### Option A: Faction Relationships Only

所有成员通过所属阵营读取和修改关系。

优点：MVP 最小、存储简单。

缺点：无法表达同阵营角色之间的个人差异，个人行为可能错误地改变整个阵营。

### Option B: Institutional Faction Baseline Plus Member Override

支持 `Faction -> Faction`、`Faction -> Member` 与 `Member -> Member`。
`Member -> Member` 是个人关系，存在时优先使用；不存在时先回退到阵营对具体成员的制度立场，
再回退到 `Faction -> Faction` 基线。`Faction -> Member` 不表示成员的私人情感，
可以由显式阵营政策或领导影响规则产生。

优点：个人关系优先、查询可解释，也能表达阵营对具体成员的制度立场。

缺点：个人关系建立或删除时可能发生关系跳变；阵营基线与个人经历不是自动加权组合。

### Option C: Weighted Composition

有效关系由阵营和成员关系加权组合：

```text
effective = faction_weight * faction_affinity
          + member_weight * member_affinity
```

优点：能同时表达群体偏见和个人经历。

缺点：权重、缺失值和钳制规则增加复杂度，调试结果较难解释。

### Option D: Explicit Policy Selected By Configuration

存储多个层级，由配置选择覆盖、加权或自定义数据策略。

优点：灵活。

缺点：MVP 和 ABI 复杂度显著增加，测试矩阵扩大。

## Parent Faction Inheritance

MVP 不支持父阵营继承或多重父阵营。未来重新讨论时，必须明确：

- 是否支持父阵营。
- 多个父阵营采用平均、权重还是最近祖先。
- 继承值是实时计算还是创建时复制。
- 显式关系是否总是优先于继承关系。
- 关系图出现循环时是拒绝配置还是限制深度。

## Missing Relationship

候选语义：

- 返回中立值 `0`，并标记来源为默认值。
- 返回 `None`，由调用者决定默认值。
- 将缺失视为领域错误。

查询 API 最好能区分“显式中立关系”和“关系不存在”。

## Leadership Influence

领导者对阵营的影响力是独立的权力或控制力维度，不复用亲和度字段。
显式评价策略可以根据领导者对目标的个人关系和其影响力，产生或更新阵营对该目标的制度立场。
当影响力满足策略定义的强势条件时，阵营立场可以跟随领导者；这不会自动覆盖成员已有的私人关系。
具体阈值、投影公式和多领导者规则不在本 ADR 中固定。

## Questions To Resolve

1. MVP 是否采用阵营基线加成员覆盖？是，采用个人关系优先的变体，并加入阵营对具体成员的制度立场。
2. 是否需要 `Member -> Faction`？MVP 不要求；保留主体类型和 ABI 扩展空间，待具体用例证明后再加入。
3. 缺失关系是否返回 `Option`？Rust 查询必须区分缺失与显式中立；有效关系查询还必须返回来源。行为评价可由策略将缺失解释为默认中立。
4. 父阵营继承是否推迟到 MVP 之后？是。MVP 不支持父阵营继承或多重父阵营。
5. 行为默认修改成员关系还是阵营关系？对具体行为者和目标，默认修改 `Member -> Member`；阵营关系只能由明确的阵营级命令或策略修改。

## Decision

1. `Member -> Member` 是 PersonaFlux 的主要关系层，用于表达角色之间有方向的个人历史、好感和敌意。
2. `Faction -> Faction` 是阵营之间的制度基线，不把阵营假设为拥有统一的私人情感。
3. MVP 支持 `Faction -> Member` 作为阵营对具体成员的制度立场。对于成员 `A` 查询成员 `B`，
   其中 `A` 属于阵营 `FA`、`B` 属于阵营 `FB` 时，有效关系按以下优先级解析：
   - 显式 `Member -> Member`；
   - `FA -> B` 的 `Faction -> Member` 制度立场；
   - 当 `FB` 存在时，使用 `FA -> FB` 的 `Faction -> Faction` 阵营基线；
   - 缺失。
4. 领导者影响力作为独立维度，由显式策略把领导者的个人关系投影为阵营制度立场。
   该投影可以使强势领导者的立场影响整个阵营，但不得隐式覆盖成员的私人关系。
5. MVP 不支持 `Member -> Faction`、父阵营继承或多重父阵营。相关主体类型和存储格式必须保留未来 ABI 扩展空间。
6. 关系查询必须区分显式中立值 `0` 与关系不存在，并暴露关系层级或来源。Rust 可以使用语义明确的可选/带来源类型；C ABI 使用显式存在标志和稳定的来源枚举，不暴露 Rust `Option`。
7. 对具体成员行为的默认写入目标是 `Member -> Member`。阵营层变化必须通过明确的阵营级命令或领导影响策略发生。

## Rationale

galgame 等场景需要表达同一阵营内角色对主角的不同态度，因此仅保存阵营关系不能满足需求。
个人关系优先可以保留角色差异，同时让没有个人历史的关系从制度立场和阵营基线获得可解释的默认值。
将领导影响力与亲和度分离，可以表达“女王钟爱主角并凭借权力使阵营对主角友善”，
而不把这种剧情效果错误地复制成每个成员的私人情感。

该方案避免在 MVP 引入加权组合、父阵营图和可配置策略矩阵，从而控制 Rust 存储、
批量 API、跨语言绑定、快照迁移和测试矩阵的复杂度。

## Consequences

关系存储必须记录关系层级和方向，至少能区分个人关系、阵营对成员的制度立场和阵营基线。
有效关系查询需要返回解析来源，以便调试、审计、事件回放和跨语言绑定保持一致。

评价和行为管线必须明确：个人关系优先；阵营制度立场不自动改写个人关系；阵营级变化由显式命令或
领导影响策略提交，并遵循现有的原子提交和有序事件规则。

C ABI 需要使用显式主体类型、关系层级/来源枚举和存在标志；不得把 Rust `Option` 或内部对象布局暴露给
Swift、Kotlin/JNI、C# 或其他宿主。快照必须保存原始关系层和模型版本，而不能只保存计算后的有效值。

实现前必须补充关系解析、领导影响、成员迁移、显式中立与缺失、循环输入（若未来启用继承）以及
跨平台 ABI/快照兼容测试，并同步更新 `docs/BEHAVIOR_SPEC.md`、关系 API 设计和测试矩阵。
