# Architecture Decision Records

本目录记录 PersonaFlux 中会长期影响领域行为、公共 API、数据格式或工程约束的重要决策。

## 状态

- `Proposed`：正在讨论，尚不能作为实现依据。
- `Accepted`：已经决定，代码和行为规格必须遵守。
- `Rejected`：讨论后不采用，保留原因供后续参考。
- `Deprecated`：仍可能存在于旧版本，但不再推荐。
- `Superseded by ADR-NNNN`：已被新决策取代。

## 工作流程

1. 从 [TEMPLATE.md](TEMPLATE.md) 复制一份文档并分配连续编号。
2. 在 `Context` 中描述问题和约束，不先假定答案。
3. 在 `Options` 中列出实际可选方案和取舍。
4. 讨论阶段保持 `Proposed`，可持续补充问题和证据。
5. 决定后填写 `Decision`、`Rationale` 和 `Consequences`，将状态改为 `Accepted` 或 `Rejected`。
6. 同步更新 `docs/BEHAVIOR_SPEC.md`、`docs/ARCHITECTURE.md` 或 `docs/C_ABI_AND_FFI.md`。
7. 用 Rust 测试固定所有可执行行为。

已接受的 ADR 不应被改写成新的结论。如果决定发生变化，创建新 ADR，并将旧 ADR 标记为被取代。

## 当前决策队列

| ADR | 主题 | 状态 |
|---|---|---|
| [0001](0001-normalized-value-ranges.md) | 数值范围与非法输入 | Proposed |
| [0002](0002-relationship-levels.md) | 关系层级与组合语义 | Proposed |
| [0003](0003-logical-time.md) | 逻辑时间表示与推进 | Proposed |
| [0004](0004-evaluation-v0.md) | MVP 行为评价模型 | Proposed |

## 哪些内容需要 ADR

需要 ADR：

- PAD、亲和度和置信度的数值范围。
- 阵营关系与成员关系如何组合。
- 时间、随机性和确定性策略。
- 行为评价公式和事件排序。
- C ABI 版本兼容与内存所有权。
- 快照格式和迁移策略。

通常不需要 ADR：

- 局部重命名或格式调整。
- 不改变行为的内部重构。
- 短期任务顺序。
- 容易撤销且没有公共影响的实现细节。
