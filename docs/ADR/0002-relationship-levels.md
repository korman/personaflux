# ADR 0002: Relationship Levels And Composition

- Status: Proposed
- Date: 2026-08-27
- Decision owners: PersonaFlux maintainers
- Related documents: `docs/BEHAVIOR_SPEC.md`, `docs/ARCHITECTURE.md`

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

## Options

### Option A: Faction Relationships Only

所有成员通过所属阵营读取和修改关系。

优点：MVP 最小、存储简单。

缺点：无法表达同阵营角色之间的个人差异，个人行为可能错误地改变整个阵营。

### Option B: Faction Baseline Plus Member Override

支持 `Faction -> Faction` 与 `Member -> Member`。成员关系存在时覆盖阵营基线，否则回退到阵营关系。

优点：语义简单、查询可解释、容易逐步采用个人关系。

缺点：覆盖会造成关系跳变，不能自然表达“阵营偏见加个人经历”。

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

还需决定：

- 是否在 MVP 支持父阵营。
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

## Questions To Resolve

1. MVP 是否采用阵营基线加成员覆盖？
2. 是否需要 `Member -> Faction`？
3. 缺失关系是否返回 `Option`？
4. 父阵营继承是否推迟到 MVP 之后？
5. 行为默认修改成员关系还是阵营关系？

## Decision

尚未决定。

## Rationale

待决定后填写。

## Consequences

待决定后填写，并同步更新关系存储、查询 API、行为规格和测试矩阵。
