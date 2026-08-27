# 命名规范

## 已选名称

项目品牌为 `PersonaFlux`：

- `Persona` 强调角色人格和社会身份。
- `Flux` 强调人格、情绪和关系持续变化。

README 副标题负责说明其社会情感模拟定位，避免名称被误解为用户画像或状态管理库。

## 技术命名

| 用途 | 名称 |
|---|---|
| 品牌 | `PersonaFlux` |
| Workspace / repository | `personaflux` |
| 核心包 | `personaflux-core` |
| FFI 包 | `personaflux-ffi` |
| Rust crate 引用 | `personaflux_core` |
| C 头文件 | `personaflux.h` |
| C 库 | `libpersonaflux` / `personaflux.dll` |
| C ABI 前缀 | `pf_` |

品牌使用 CamelCase 符合 Rust 生态；包名使用 kebab-case，代码标识符使用 snake_case，Rust 类型使用 PascalCase。

## 项目描述

> PersonaFlux is a deterministic social and emotional simulation core for interactive characters.

## 发布前检查

- GitHub 组织和仓库。
- crates.io、PyPI、NuGet、Maven Central 和 npm。
- 域名和社交账号。
- 主要目标市场商标。
- 搜索引擎中的同类软件和 SDK。

ABI v1 发布后，不更改 `pf_` 前缀、库名或基础包坐标。
