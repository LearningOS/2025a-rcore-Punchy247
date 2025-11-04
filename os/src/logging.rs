//! ===========================================
//! 全局日志记录器模块 (logging.rs)
//! ===========================================
//! 这个模块实现了一个简单的日志记录系统，支持不同级别的日志输出
//! 功能特点：
//! 1. 支持 5 种日志级别：Error、Warn、Info、Debug、Trace
//! 2. 彩色输出：不同级别使用不同颜色，便于区分
//! 3. 可配置级别：通过环境变量 LOG 控制输出哪些级别的日志
//! 
//! 日志级别从高到低（重要性递减）：
//! Error > Warn > Info > Debug > Trace
//! 设置某个级别后，会输出该级别及以上的所有日志

// 导入 log crate 中的类型和 trait
// log 是一个标准的日志接口 crate，定义了一组通用的日志接口
use log::{Level, LevelFilter, Log, Metadata, Record};

/// 简单的日志记录器结构体
/// 
/// 这是一个零大小类型（ZST），不存储任何状态
/// 它只需要实现 Log trait 中定义的方法即可
struct SimpleLogger;

/// 为 SimpleLogger 实现 Log trait
/// 
/// Log trait 是 log crate 定义的标准接口
/// 任何实现了 Log trait 的类型都可以作为日志记录器使用
impl Log for SimpleLogger {
    /// 检查某个日志级别是否启用
    /// 
    /// # 参数
    /// - `_metadata`: 日志记录的元数据（如级别、模块路径等）
    ///   下划线前缀 `_` 表示这是一个未使用的参数（避免编译器警告）
    /// 
    /// # 返回值
    /// - `true`: 总是返回 true，表示所有级别的日志都启用
    /// 
    /// # 注意
    /// 实际的过滤逻辑在 log::set_max_level 中实现
    /// 这个函数只是简单地返回 true，真正的过滤由 log crate 完成
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    
    /// 记录日志消息
    /// 
    /// # 参数
    /// - `record`: 日志记录，包含级别、消息内容等信息
    /// 
    /// # 实现原理
    /// 1. 检查日志是否启用（虽然这里总是返回 true）
    /// 2. 根据日志级别选择颜色代码（ANSI 转义序列）
    /// 3. 使用 println! 输出带颜色的日志消息
    /// 
    /// # ANSI 转义序列说明
    /// ANSI 转义序列用于控制终端颜色和格式
    /// - \u{1B}[ 是转义序列的开始（ESC 字符）
    /// - {}m 是设置颜色的命令，{} 会被颜色代码替换
    /// - \u{1B}[0m 是重置所有属性的命令
    /// 
    /// # 示例输出
    /// [ERROR] 内核 panic 了！
    /// [WARN ] 内存使用率较高
    /// [INFO ] 系统启动完成
    fn log(&self, record: &Record) {
        // 再次检查是否启用（双重保险）
        if !self.enabled(record.metadata()) {
            return;
        }
        
        // match 表达式：根据日志级别选择颜色代码
        // match 是 Rust 的模式匹配，类似于其他语言的 switch-case
        // 但它更强大，可以进行模式匹配和解构
        let color = match record.level() {
            // Level 是枚举类型（enum），包含所有日志级别
            // => 后面是对应级别使用的 ANSI 颜色代码
            Level::Error => 31,  // 红色 - 用于错误信息
            Level::Warn => 93,   // 亮黄色 - 用于警告信息
            Level::Info => 34,   // 蓝色 - 用于一般信息
            Level::Debug => 32,  // 绿色 - 用于调试信息
            Level::Trace => 90,  // 亮黑色（灰色）- 用于追踪信息（最详细）
        };
        
        // 输出带颜色的日志消息
        // println! 宏用于格式化输出并自动换行
        println!(
            // 格式化字符串说明：
            // \u{1B}[{}m  - 设置颜色（{} 会被 color 的值替换）
            // [{:>5}]     - 日志级别，右对齐，宽度为 5 个字符
            //              {:>5} 表示右对齐，> 是右对齐符号，5 是宽度
            // {}           - 实际的日志消息内容
            // \u{1B}[0m   - 重置颜色和格式
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,                    // 颜色代码（替换第一个 {}）
            record.level(),           // 日志级别（替换 {:>5}）
            record.args(),           // 日志消息内容（替换最后一个 {}）
        );
    }
    
    /// 刷新日志缓冲区
    /// 
    /// # 功能
    /// 在某些日志系统中，日志可能被缓冲，需要显式刷新才能输出
    /// 在我们的简单实现中，每次 log 都直接输出，不需要缓冲
    /// 所以这个函数是空实现（什么都不做）
    fn flush(&self) {
        // 空实现：不进行任何操作
        // 因为我们使用 println! 直接输出，不需要缓冲
    }
}

/// 初始化日志记录器
/// 
/// # 功能
/// 1. 创建并注册全局日志记录器
/// 2. 根据环境变量设置日志过滤级别
/// 
/// # 调用时机
/// 这个函数应该在操作系统启动早期调用（在 main 函数中）
/// 这样后续的所有日志都能正常工作
/// 
/// # 环境变量配置
/// 通过设置环境变量 LOG 来控制日志级别：
/// - LOG=ERROR: 只显示错误日志
/// - LOG=WARN:  显示警告和错误
/// - LOG=INFO:  显示信息、警告和错误（推荐）
/// - LOG=DEBUG: 显示调试及以上级别
/// - LOG=TRACE: 显示所有日志（最详细）
/// - 未设置:     关闭所有日志输出
/// 
/// # 使用示例
/// ```bash
/// # 在命令行中设置环境变量
/// LOG=INFO make run
/// ```
pub fn init() {
    // static 关键字定义静态变量（全局变量）
    // LOGGER 是静态的 SimpleLogger 实例，生命周期为整个程序运行期间
    // const 关键字表示这是一个编译时常量（但 SimpleLogger 是 ZST，所以可以是 const）
    static LOGGER: SimpleLogger = SimpleLogger;
    
    // 注册日志记录器
    // set_logger 将我们的 SimpleLogger 设置为全局日志记录器
    // unwrap() 会解包 Result，如果失败（比如已设置过记录器）会 panic
    // 在正常情况下，这应该只会调用一次，所以使用 unwrap 是安全的
    log::set_logger(&LOGGER).unwrap();
    
    // 设置最大日志级别（过滤级别）
    // set_max_level 决定哪些级别的日志会被输出
    // 只有小于等于此级别的日志才会被记录
    log::set_max_level(
        // option_env! 是编译时宏，用于读取环境变量
        // 与 env! 不同，option_env! 返回 Option<&str>，如果环境变量不存在返回 None
        // 这样不会在编译时 panic（env! 如果变量不存在会在编译时报错）
        match option_env!("LOG") {
            // match 表达式匹配环境变量的值
            // Some(value) 表示环境变量存在，value 是变量的值
            // None 表示环境变量不存在
            Some("ERROR") => LevelFilter::Error,  // 只显示错误
            Some("WARN") => LevelFilter::Warn,    // 显示警告及以上
            Some("INFO") => LevelFilter::Info,    // 显示信息及以上（常用）
            Some("DEBUG") => LevelFilter::Debug,  // 显示调试及以上
            Some("TRACE") => LevelFilter::Trace,  // 显示所有日志
            // _ 是通配符，匹配所有其他情况（包括 None 和其他值）
            _ => LevelFilter::Off,  // 默认关闭所有日志
        }
    );
}

// ===========================================
// Rust 知识补充：
// ===========================================
// 1. 枚举（Enum）
//    - Level 和 LevelFilter 都是枚举类型
//    - 枚举可以包含不同的变体（Variant）
//    - 使用 match 表达式进行模式匹配
// 
// 2. 静态变量（Static）
//    - static 定义的变量生命周期为整个程序
//    - 可以有可变和不可变两种
//    - 静态变量在程序的静态内存区域，不是栈上的
// 
// 3. 模式匹配（Pattern Matching）
//    - match 是 Rust 最强大的控制流结构
//    - 可以匹配各种模式：值、范围、结构体、枚举等
//    - _ 是通配符，匹配所有情况
// 
// 4. 宏（Macro）
//    - option_env! 是编译时宏，在编译时读取环境变量
//    - 返回 Option<&str>，安全地处理环境变量不存在的情况
// 
// 5. Result 和 unwrap
//    - Result<T, E> 是 Rust 的错误处理类型
//    - Ok(value) 表示成功，Err(error) 表示失败
//    - unwrap() 解包 Result，失败时会 panic
//    - 在生产代码中应使用 ? 操作符或 match 处理错误
