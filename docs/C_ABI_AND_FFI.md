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

### 数值字段

- C ABI 数值字段使用明确命名的 `float`，对应 IEEE-754 binary32。
- PAD、affinity、traits、impact 字段使用 `[-1.0f, 1.0f]`。
- confidence、aggression 字段使用 `[0.0f, 1.0f]`。
- 字段名和文档必须明确数值语义，不以百分制表示，也不允许隐式单位转换。
- 外部越界、NaN 和 Infinity 返回 `PF_INVALID_ARGUMENT`，不自动钳制。
- `-0.0f` 在 ABI 输入边界规范化为 `+0.0f`。

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
- 内部出现 NaN、Infinity 或最终状态越界返回 `PF_INTERNAL_ERROR`，且失败调用不得产生部分状态或事件。
- 批量 API 先完整校验；失败时整体不提交，并返回从零开始的失败项索引。无法归因到具体元素时使用 `UINT64_MAX`。

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
- 快照中的浮点值使用明确格式的 little-endian IEEE-754 binary32；`-0.0` 规范化为 `+0.0`。
- C ABI 只承诺行为级确定性，不承诺不同 CPU 的浮点结果逐位一致；跨平台测试使用明确数值容差。

## 安全测试

覆盖空句柄、重复销毁策略、非法 ID、空指针、极端长度、UTF-8 错误、NaN、Infinity、缓冲区不足、panic containment、C 头文件编译和模糊测试。

## ABI v0 实现契约

The live Rust core is exposed through the hand-maintained header
`crates/personaflux-ffi/include/personaflux.h`. ABI v0 uses
`PF_ABI_VERSION == 0` and the fixed evaluation model reports
`PF_MODEL_VERSION == 1`. Snapshots, rumors, personality data, and language
wrappers are intentionally outside this version.

The public C surface covers simulation lifetime, faction/member creation and
queries, all three relationship layers, direct-witness submission (including
the atomic batch form), logical time, memory queries, and event count/read/clear.
The Rust core remains the only implementation of these behaviors; the FFI
layer only validates and translates wire values.

The layout fixture is `crates/personaflux-ffi/tests/header_smoke.c`. Supported
build environments should compile it with a C11 compiler and the include path
`crates/personaflux-ffi/include`.

### Wire rules

- IDs are `uint64_t`; result codes are fixed-width `int32_t` values.
- Public extensible structs begin with `uint32_t struct_size` and
  `uint32_t api_version`. New fields are append-only.
- Rust `Option`, enums, and booleans are represented by explicit presence or
  tag fields. Boolean inputs accept only `0` and `1`.
- Floating-point fields are C `float` values using the documented normalized
  ranges. Invalid finite/range values return `PF_INVALID_ARGUMENT`.
- A deed with `has_target == 0` must set `target` to zero. No special member ID
  represents a missing target.

### Ownership and errors

All byte arrays, memory records, submissions, and events use caller-owned
buffers. Count/read operations never retain caller pointers. A read with an
insufficient capacity returns `PF_BUFFER_TOO_SMALL` and does not mutate the
simulation or event queue. Event reads are non-destructive; `pf_events_clear`
is the only event queue clearing operation.

Every exported function contains panics and returns `PF_INTERNAL_ERROR` if a
panic occurs. Diagnostic text is thread-local and copied through
`pf_last_error_message_copy`; callers must use the numeric result code for
control flow. A simulation handle must be accessed serially by its host. A
null destroy is a no-op; other null handles or required null pointers are
invalid arguments.

### Stable tags

Submission, relationship source, memory kind, memory decision, and event tags
are fixed numeric constants in the header. Explicit neutral relationships use
`present == 1` and `affinity == 0.0`; missing relationships use
`present == 0`.

## Binding verification

The v0 wrappers live under `bindings/csharp`, `bindings/swift`, and
`bindings/kotlin`. They are intentionally translation-only layers: all
evaluation, relationship resolution, state mutation, memory retention, time,
deduplication, and event ordering remain in `personaflux-core` and
`personaflux-ffi`.

Each wrapper owns an opaque handle and copies caller-provided strings, arrays,
records, and events. C# uses `SafeHandle` and Cdecl P/Invoke; Swift uses
`OpaquePointer` and `deinit`; Kotlin uses `AutoCloseable` and a small JNI
bridge over direct buffers. Public wrapper APIs expose value types and map
numeric result codes to language-native errors while preserving batch error
indices and unknown future tags.

The CI workflow validates C11 layout, Rust exports, .NET loading, Swift
device/simulator packaging, and Android AAR assembly. Native artifacts are
build outputs and are not checked into the repository.

ABI v0 is not silently promoted to v1 by binding changes. After all supported
language and platform tests pass, a separate maintainer-approved release may
publish a versioned ABI v1 header and artifacts. Existing v0 headers, symbols,
and compatibility tests remain available for long-term consumers.
