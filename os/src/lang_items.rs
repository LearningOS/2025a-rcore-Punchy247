//! ===========================================
//! 语言项模块 (lang_items.rs)
//! ===========================================
//! 这个模块实现了 Rust 语言的核心功能在裸机环境下的支持
//! 主要功能：
//! 1. 实现 panic 处理程序（panic handler）
//! 
//! # 什么是语言项（Language Item）？
//! 语言项是 Rust 编译器期望在 crate 中定义的特定函数和类型
//! 在标准库环境中，这些由标准库提供；在裸机环境中，我们必须自己实现
//! 
//! # 为什么需要 panic handler？
//! 当程序遇到不可恢复的错误时（如数组越界、除零等），会触发 panic
//! 在标准库环境中，panic 会打印错误信息并终止程序
//! 在裸机环境中，我们需要定义自己的 panic 处理行为

// 导入所需的模块和类型
use crate::sbi::shutdown;  // 用于关闭系统
use core::panic::PanicInfo;  // PanicInfo 包含 panic 的详细信息

/// panic 处理程序
/// 
/// # 功能说明
/// 当 Rust 程序发生 panic 时（如数组越界、unwrap None 等），
/// 这个函数会被自动调用
/// 
/// # 属性说明
/// `#[panic_handler]` 是 Rust 的语言项属性
/// 告诉编译器：这个函数就是 panic 处理程序
/// 如果没有定义这个函数，编译器会报错（在 no_std 环境中）
/// 
/// # 函数签名说明
/// - `fn panic(info: &PanicInfo) -> !`
///   - `info: &PanicInfo`: panic 的详细信息（位置、消息等）
///   - `-> !`: 返回类型为 `!`，表示这是一个发散函数（Diverging Function）
///     发散函数永远不会返回，要么无限循环，要么终止程序
/// 
/// # panic 的常见原因
/// - 数组越界访问
/// - 除零操作
/// - unwrap() 一个 None 值
/// - 整数溢出（如果启用了溢出检查）
/// - 断言失败（assert!、assert_eq! 等）
/// 
/// # 处理流程
/// 1. 尝试获取 panic 的位置信息（文件名和行号）
/// 2. 打印错误信息（包括位置和消息）
/// 3. 调用 shutdown() 关闭系统（在 QEMU 中会退出模拟器）
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // if let 是 Rust 的模式匹配语法，用于处理 Option 类型
    // info.location() 返回 Option<&Location>
    // - 如果位置信息可用，返回 Some(location)
    // - 如果位置信息不可用，返回 None
    if let Some(location) = info.location() {
        // 如果有位置信息，打印详细错误信息
        println!(
            // 格式化字符串：显示文件名、行号和错误消息
            "[kernel] Panicked at {}:{} {}",
            location.file(),        // 发生 panic 的文件名
            location.line(),        // 发生 panic 的行号
            info.message().unwrap() // panic 的错误消息
            // unwrap() 在正常情况下不应该失败，因为 panic 总是有消息
        );
    } else {
        // 如果没有位置信息，只打印错误消息
        // 这种情况比较少见，通常发生在某些底层代码中
        println!(
            "[kernel] Panicked: {}", 
            info.message().unwrap()
        );
    }
    
    // 关闭系统
    // shutdown() 函数的返回类型也是 !（永不返回）
    // 所以整个 panic 函数永远不会返回，这是正确的行为
    // 在 QEMU 环境中，这会退出模拟器；在真实硬件上，可能需要其他处理方式
    shutdown()
}

// ===========================================
// Rust 知识补充：
// ===========================================
// 1. 属性（Attribute）
//    - #[panic_handler] 是属性宏，用于标记语言项
//    - 属性以 # 开头，可以放在函数、结构体、模块等前面
//    - 属性在编译时处理，用于给编译器提供元信息
// 
// 2. Panic 机制
//    - panic 是 Rust 的错误处理机制之一（另一个是 Result）
//    - panic 用于不可恢复的错误（bug）
//    - Result 用于可预期的错误（如文件不存在）
//    - panic 会展开调用栈，执行清理代码，然后终止程序
// 
// 3. 发散函数（Diverging Function）
//    - 返回类型 ! 表示永不返回的函数
//    - 可能的原因：无限循环、终止程序、永远 panic 等
//    - ! 类型是所有类型的子类型，可以转换为任何类型
// 
// 4. Option 类型和模式匹配
//    - Option<T> 是 Rust 的可空类型，Some(T) 或 None
//    - if let 是轻量级的模式匹配，适用于简单的 Option 处理
//    - match 表达式更强大，可以处理更复杂的模式
// 
// 5. 错误处理策略
//    - 在裸机环境中，panic 处理必须简单快速
//    - 不能依赖文件系统、网络等可能失败的服务
//    - 通常会记录错误信息，然后终止或重启系统
