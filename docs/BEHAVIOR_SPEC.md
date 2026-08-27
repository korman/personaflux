# 行为规格草案

## 状态

本文定义 PersonaFlux 需要独立确定和测试的行为，不复制 Love/Hate 公式。所有“待决定”项目必须在实现和 ABI 冻结前形成 ADR 与测试向量。

## 数值约定

待决定内部使用 `[-1, 1]` 还是 `[-100, 100]`。

无论采用哪种范围：

- PAD 和亲和度必须限制在合法区间。
- NaN 和 Infinity 必须被拒绝。
- 边界、饱和、舍入和精度必须有测试。

## PAD 与情绪区域

`Pad` 包含 pleasure、arousal、dominance。长期幸福感如需要，应使用独立 `well_being`。

- 修改后限制各轴。
- 情绪区域匹配当前 PAD。
- 重叠区域按显式优先级和稳定顺序选择。
- 可选稳定化按逻辑时间向目标回归。

## 关系

关系有方向：`affinity(A, B) != affinity(B, A)`。

待决定：

- 阵营、成员和成员到阵营关系的支持范围。
- 成员关系如何覆盖或叠加阵营关系。
- 父阵营继承采用平均、加权还是显式策略。
- 缺失关系使用中立值还是返回错误。

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

- 支持短期和长期记忆。
- 容量可配置，使用逻辑时间过期。
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
