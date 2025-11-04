//! ===========================================
//! 控制台驱动模块 (console.rs)
//! ===========================================
//! 这个模块提供了控制台输出功能，封装了底层的 SBI 调用
//! 主要功能：
//! 1. 实现了 Rust 标准库的 Write trait，提供格式化输出能力
//! 2. 定义了 print! 和 println! 宏，方便进行格式化输出
//! 
//! 在裸机环境下，我们无法使用 Rust 标准库的 println!，
//! 所以需要自己实现一个类似的接口

// 导入 SBI 控制台输出函数和 Rust 核心库的格式化相关类型
use crate::sbi::console_putchar;
use core::fmt::{self, Write};  // Write trait 用于格式化输出

/// Stdout 结构体：标准输出
/// 
/// 这是一个零大小类型（Zero-Sized Type，ZST），不占用任何内存
/// 它只是一个标记，用于实现 Write trait
/// 在裸机环境中，我们没有真正的"文件"，所以不需要存储任何状态
struct Stdout;

/// 为 Stdout 实现 Write trait
/// 
/// Write trait 是 Rust 中用于格式化输出的核心 trait
/// 实现了 Write trait 后，就可以使用 format_args!、write!、writeln! 等宏
/// 
/// # Trait 说明
/// Trait 是 Rust 中的接口（Interface）概念，类似于其他语言的接口
/// 通过实现 trait，我们可以让自定义类型具备特定的行为
impl Write for Stdout {
    /// 写入字符串到标准输出
    /// 
    /// # 参数
    /// - `self`: &mut Self（可变的自身引用）
    /// - `s`: &str（字符串切片，要输出的内容）
    /// 
    /// # 返回值
    /// - `fmt::Result`: 格式化操作的结果（Ok(()) 或 Err）
    /// 
    /// # 实现原理
    /// 遍历字符串中的每个字符，调用底层 SBI 函数逐个输出
    /// 
    /// # Rust 字符串处理
    /// - `s.chars()`: 返回字符串的字符迭代器
    ///   Rust 字符串是 UTF-8 编码的，每个字符可能占用 1-4 个字节
    ///   chars() 会正确地处理 UTF-8 字符边界，返回 Unicode 字符
    /// - `c as usize`: 将字符转换为 usize（在 RISC-V 64位中，usize 是 64 位整数）
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 遍历字符串中的每个字符
        // for...in 是 Rust 的迭代器语法，等价于其他语言的 for-each
        for c in s.chars() {
            // 调用底层 SBI 函数输出字符
            // 在裸机环境中，字符通过串口（UART）输出到宿主机的终端
            console_putchar(c as usize);
        }
        
        // 返回成功结果
        // Ok(()) 表示操作成功，() 是单元类型（类似 void）
        Ok(())
    }
}

/// 格式化输出函数
/// 
/// # 参数
/// - `args`: fmt::Arguments（格式化参数）
/// 
/// # 功能
/// 这个函数接收格式化参数（通常由 format_args! 宏生成），
/// 然后使用 Stdout 实例进行格式化输出
/// 
/// # 使用方式
/// 通常不直接调用此函数，而是通过 print! 或 println! 宏间接调用
/// 
/// # unwrap 说明
/// unwrap() 会解包 Result，如果失败会 panic
/// 在正常情况下，write_fmt 不应该失败，所以使用 unwrap 是安全的
pub fn print(args: fmt::Arguments) {
    // 创建 Stdout 实例（零大小类型，不占用内存）
    // 调用 write_fmt 进行格式化输出
    // write_fmt 是 Write trait 提供的方法，用于处理格式化字符串
    Stdout.write_fmt(args).unwrap();
}

/// print! 宏：格式化输出（不换行）
/// 
/// # 宏说明
/// 宏（Macro）是 Rust 的元编程功能，在编译时展开为代码
/// 使用 macro_rules! 可以定义声明式宏（Declarative Macro）
/// 
/// # 宏语法
/// - `#[macro_export]`: 将宏导出到 crate 根，使其他模块可以使用
/// - `macro_rules! print`: 定义宏名称为 print
/// - `($fmt: literal $(, $($arg: tt)+)?)`: 宏参数模式
///   - `$fmt: literal`: 第一个参数必须是字面量字符串（如 "Hello {}"）
///   - `$(, ...)?`: 可选的可变参数列表
///   - `$($arg: tt)+`: 一个或多个类型为 tt（Token Tree）的参数
/// 
/// # 使用示例
/// ```rust
/// print!("Hello, world!");
/// print!("Number: {}", 42);
/// ```
#[macro_export]
macro_rules! print {
    // 宏模式匹配
    // $fmt 是格式化字符串，$(...)? 表示可选的参数列表
    ($fmt: literal $(, $($arg: tt)+)?) => {
        // 宏展开后的代码
        // format_args! 是 Rust 内置宏，用于创建格式化参数
        // $crate::console::print 调用我们定义的 print 函数
        // $crate 是当前 crate 的根路径，确保宏在任意位置都能正确工作
        $crate::console::print(format_args!($fmt $(, $($arg)+)?))
    }
}

/// println! 宏：格式化输出（自动换行）
/// 
/// # 功能
/// 与 print! 宏类似，但在输出末尾自动添加换行符（\n）
/// 
/// # 实现原理
/// 使用 concat! 宏将格式化字符串与 "\n" 拼接
/// 
/// # 使用示例
/// ```rust
/// println!("Hello, world!");  // 输出后会换行
/// println!("Number: {}", 42);  // 输出后会换行
/// ```
#[macro_export]
macro_rules! println {
    // 宏模式匹配
    ($fmt: literal $(, $($arg: tt)+)?) => {
        // concat! 宏用于在编译时拼接字符串
        // concat!($fmt, "\n") 会将格式化字符串与换行符拼接
        // 例如：concat!("Hello {}", "\n") 展开为 "Hello {}\n"
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

// ===========================================
// Rust 知识补充：🤨
// ===========================================
// 1. Trait（特质）
//    - Trait 类似于其他语言的接口，定义了一组方法签名
//    - 通过 impl Trait for Type 为类型实现 trait
//    - 实现了某个 trait 的类型可以调用该 trait 的方法
// 
// 2. 宏（Macro）
//    - 宏在编译时展开为代码，提供代码生成能力
//    - macro_rules! 用于定义声明式宏
//    - #[$attr] 形式的属性用于修饰项（如函数、结构体等）
// 
// 3. 零大小类型（ZST）
//    - 不占用内存的类型（如 (), struct Foo;）
//    - 编译时优化，运行时不存在
//    - 用于实现 trait 而不存储状态
// 
// 4. 字符串和字符
//    - Rust 字符串是 UTF-8 编码的字节序列
//    - &str 是字符串切片，不可变
//    - char 是 Unicode 标量值（4 字节）
//    - chars() 方法返回字符迭代器，正确处理 UTF-8
