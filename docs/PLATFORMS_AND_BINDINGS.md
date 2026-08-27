# 平台产物与语言绑定

## 产物矩阵

| 平台或使用方 | 推荐产物 |
|---|---|
| Rust | crates.io `rlib` |
| C/C++ | `.a` / `.lib` / `.so` / `.dll` 加 C 头文件 |
| Windows C#、Python、Unity | `.dll` |
| Linux | `.so` |
| macOS | `.dylib` 或 `.a` |
| iOS / Swift | 静态库封装为 `.xcframework` |
| Android / Kotlin | 多 ABI `.so` 加 JNI/Kotlin 包装和 AAR |
| Web | 独立 WASM 包装，不要求逐项复制 C ABI |

## C 与 C++

- C ABI 是最低层契约。
- 使用 `cbindgen` 生成或校验头文件。
- 提供最小 C 示例和可选 C++ RAII 包装。

## C# 与 Unity

- C# 使用 P/Invoke 和 `SafeHandle` 管理生命周期。
- Unity 层负责 GameObject 到 `MemberId` 映射、Physics 目击者、逻辑时间和事件桥接。
- Unity 依赖不进入核心 crate。

## Swift 与 iOS

- 构建 device 和 simulator 静态库并打包 XCFramework。
- Swift 使用 `OpaquePointer` 和 `deinit` 管理句柄。
- 可发布 Swift Package。
- Rust 不从后台线程直接回调 Swift。

## Kotlin 与 Android

- 使用 `cargo-ndk` 构建 `arm64-v8a`、`armeabi-v7a` 和 `x86_64`。
- 小型 JNI 层调用 C ABI，Kotlin 提供惯用包装。
- 高频操作使用批量命令，减少 JNI 往返。

## Python 与 UniFFI

Python 可用 `ctypes`/`cffi` 包装 C ABI，后续可提供 PyO3。UniFFI 可辅助 Swift、Kotlin 和 Python，但不能替代面向 C、C++、Unity 和任意宿主的 C ABI。

## CI

至少覆盖 Rust 测试、C 头文件编译、Windows/Linux/macOS 构建、Android 多 ABI、iOS device/simulator、绑定生命周期和导出符号检查。
