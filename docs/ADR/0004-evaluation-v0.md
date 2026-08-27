# ADR 0004: MVP Deed And Rumor Evaluation Model

- Status: Proposed
- Date: 2026-08-27
- Decision owners: PersonaFlux maintainers
- Related documents: `docs/AFFECT_THEORY.md`, `docs/BEHAVIOR_SPEC.md`

## Context

PersonaFlux 的核心价值在于角色如何根据行为、关系和信息来源形成不同评价。MVP 需要一个原创、可解释、确定性且有界的评价模型，同时避免在第一版引入过多参数。

评价模型应先定义行为性质和测试向量，再固定精确公式。

## Decision Drivers

- 同一事件可因观察者立场不同产生不同结果。
- 每个因子可解释和单独测试。
- 所有修正有界。
- 中立关系的行为有明确语义。
- 结果能映射到亲和度和 PAD。
- 未来可增加人格、力量和习惯化，而不破坏基本管线。

## Proposed Inputs

MVP 最小输入候选：

```text
deed impact
deed aggression
confidence in information
trust in source
affinity toward target
current PAD
```

后续候选：人格契合、力量差、重复次数和文化规范。

## Semantic Cases To Decide First

1. 观察者喜欢目标，行为者帮助目标。
2. 观察者喜欢目标，行为者伤害目标。
3. 观察者讨厌目标，行为者伤害目标。
4. 观察者对目标中立。
5. 来源完全可信、不可信或不存在。
6. 行为没有明确目标。
7. 行为影响为零但攻击性非零。

每个案例先决定结果方向，再决定精确幅度。

## Formula Options

### Option A: Direct Multiplicative Model

```text
effective_confidence = rumor_confidence * source_trust
affinity_delta = effective_confidence * impact * target_affinity
```

优点：简单，天然支持“敌人的敌人”。

缺点：目标中立时所有事件影响为零；零信任的直接目击需要特殊语义。

### Option B: Baseline Concern Model

```text
target_concern = neutral_concern + relationship_factor
affinity_delta = effective_confidence * impact * target_concern
```

优点：中立目标仍能产生道德评价。

缺点：需要定义 baseline；若不允许 concern 为负，则不能自然反转敌对目标评价。

### Option C: Signed Concern With Neutral Baseline

将道德基线与关系立场分开：

```text
affinity_delta = effective_confidence
               * impact
               * (moral_baseline + relationship_weight * target_affinity)
```

优点：既支持中立评价，也可支持关系反转。

缺点：参数更多，需钳制组合因子并解释极值。

### Option D: Staged Appraisal

先计算可解释因子，再组合为评价：

```text
Appraisal {
    credibility,
    desirability,
    norm_alignment,
    threat,
    control,
}
```

优点：扩展性和解释性最好。

缺点：MVP 领域模型和测试工作量更大。

## PAD Mapping Questions

- Pleasure 是否跟随对行为者的评价方向？
- Arousal 是否取事件主观变化的绝对幅度？
- Dominance 是否主要由 aggression 和力量差决定？
- 低于记忆阈值的事件是否仍短暂改变 PAD？
- PAD delta 是否与关系 delta 使用相同尺度？

## Direct Witness And Rumor

需要区分：

- 直接目击通常具有高置信度，来源可视为自己。
- 传闻置信度受来源信任影响。
- 同一 deed ID 不能重复施加完整影响。
- 传播链是否导致置信度逐跳衰减。

## Questions To Resolve

1. 中立目标是否仍触发基础道德评价？
2. 讨厌目标是否可以反转对伤害者的评价？
3. MVP 采用直接公式还是分阶段 Appraisal？
4. source trust 的范围和负值语义是什么？
5. 直接目击是否绕过 source trust？
6. 第一版是否先排除人格、力量和习惯化？

## Decision

尚未决定。

## Rationale

待决定后填写。

## Consequences

待决定后填写。决定后必须建立方向测试、固定数值测试、边界测试和解释因子快照。
