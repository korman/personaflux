# C ABI 与 FFI

## 产物

`personaflux-core` 使用普通 Rust `rlib`。`personaflux-ffi` 同时输出：

```toml
[lib]
name = "personaflux"
crate-type = ["staticlib", "cdylib"]
```

Rust ABI 不稳定，所有跨语言公共边界使用版本化 C ABI。

## 不透明句柄和 ID

```c
typedef struct pf_simulation pf_simulation_t;
typedef uint64_t pf_faction_id_t;
typedef uint64_t pf_member_id_t;
```

外部不能读取 Rust 对象布局，Rust 地址不能作为公开实体 ID。

## 推荐 API

```text
pf_simulation_create / destroy / step
pf_faction_add
pf_member_add
pf_relationship_set
pf_deeds_commit_batch
pf_rumors_share_batch
pf_member_state_get
pf_events_count / read / clear
pf_snapshot_export / import
```

不要导出每个 Rust 方法；优先完整业务动作和批量操作。

## ABI 类型

允许：

- `#[repr(C)]` 结构体。
- 固定宽度整数和明确精度浮点。
- 指针与长度。
- 不透明句柄。
- 明确底层类型的枚举或常量。

禁止直接暴露 Rust `String`、`Vec`、引用、slice、默认 enum、`bool`、`Option`、trait object、闭包和泛型。

字符串使用 UTF-8 字节指针和长度。数组同样传指针与元素数量。必须验证空指针、长度溢出、对齐和有限浮点。

## 内存所有权

- 谁分配，谁释放。
- Rust 内存只能由 Rust API 释放。
- 输入指针仅在当前调用有效。
- 不允许宿主长期保存 Rust 内部数组指针。
- 优先使用调用方输出缓冲区。
- 销毁函数对空句柄必须安全。

## 错误和 panic

所有可失败函数返回固定错误码，例如：

```text
PF_OK
PF_INVALID_ARGUMENT
PF_NOT_FOUND
PF_INVALID_STATE
PF_BUFFER_TOO_SMALL
PF_SERIALIZATION_ERROR
PF_VERSION_MISMATCH
PF_INTERNAL_ERROR
```

- panic 不得跨 C ABI。
- 导出入口统一捕获 panic。
- 详细文本通过独立 last-error API 获取。
- 业务逻辑不能依赖错误字符串。

## ABI 演进

- 公开结构可包含 `struct_size` 和 `api_version`。
- 新字段只添加在尾部。
- 不改变已发布枚举值和基础类型宽度。
- 保留旧函数，通过新增函数演进。
- 每个版本维护 C 头文件编译和 ABI 兼容测试。

## 回调和事件

第一版不使用宿主回调。宿主提交命令，Rust 写事件队列，宿主轮询或 drain。这样避免 GC 固定、线程、异常、重入和生命周期问题。

## 序列化

- 配置和调试可用 JSON。
- 高频操作使用 C 结构体和批量数组。
- 快照必须带版本，不直接序列化 Rust 内部布局。
- 二进制格式必须有 schema 和迁移策略。

## 安全测试

覆盖空句柄、重复销毁策略、非法 ID、空指针、极端长度、UTF-8 错误、NaN、Infinity、缓冲区不足、panic containment、C 头文件编译和模糊测试。
