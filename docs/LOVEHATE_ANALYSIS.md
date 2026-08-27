# Love/Hate 前期分析

## 核心功能

Love/Hate 是 Unity NPC 社会关系和情绪模拟插件。它让角色根据看到的行为、听到的传闻、所属阵营、人格倾向和已有关系，动态改变情绪、记忆和对其他角色的态度。

核心闭环：

```text
定义阵营、人格和关系
    -> 角色成为阵营成员
    -> 宿主报告行为
    -> 管理器确定目击者
    -> 目击者评价行为或传闻
    -> 更新关系、PAD 和人格
    -> 重要事件进入记忆
    -> NPC 传播传闻
    -> 接收者基于信任再次评价
```

评价因素主要包括来源信任、目标亲和度、行为影响、攻击性、人格契合、当前唤醒度、力量差和重复次数。

## 原项目模块

- `FactionDatabase`：阵营、特质、关系和继承。
- `FactionMember`：PAD、记忆、目击、传闻、人格和关系变化。
- `FactionManager`：注册、查询、目击队列和序列化。
- `Scripts/Interaction`：行为模板、八卦、问候、光环、动画和事件桥接。
- `Scripts/Editor`：Unity Inspector 和数据编辑工具。

## 代码规模

前期工作区统计：

| 范围 | C# 文件 | 物理行 | 非空行 | 近似有效代码行 |
|---|---:|---:|---:|---:|
| `Scripts/Core` | 28 | 4,946 | 4,302 | 2,837 |
| `Scripts/Interaction` | 25 | 1,341 | 1,094 | 830 |
| 核心与交互 | 53 | 6,287 | 5,396 | 3,667 |
| 全部 C# | 104 | 10,695 | 9,184 | 未统计 |

最大三个核心文件合计 2,743 行：`FactionMember.cs` 1,189 行、`FactionDatabase.cs` 878 行、`FactionManager.cs` 676 行。

## 架构结论

原架构是典型 Unity 组件式设计：

```text
ScriptableObject database
    -> MonoBehaviour manager
    -> GameObject member components
    -> Trigger / Physics / EventSystem
```

它适合 Unity，但不适合作为跨语言库直接蓝本。PersonaFlux 应保留阵营、关系、PAD、行为、记忆和传闻等领域概念，重做对象所有权、时间、感知、事件和 API 架构。
