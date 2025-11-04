//! ===========================================
//! QEMU 退出处理模块 (boards/qemu.rs)
//! ===========================================
//! 这个模块实现了在 QEMU 模拟器中退出虚拟机的功能
//! 主要用于：
//! 1. 正常退出：内核运行完毕后通知 QEMU 成功退出
//! 2. 异常退出：内核 panic 或遇到严重错误时通知 QEMU 失败退出
//! 
//! # QEMU 退出机制
//! QEMU 提供了一个特殊的测试设备（sifive_test），位于内存地址 0x100000
//! 向这个地址写入特定的值可以让 QEMU 退出并返回退出码
//! 
//! 参考：https://github.com/andre-richter/qemu-exit

use core::arch::asm;  // 用于内联汇编

// ===========================================
// 退出码常量定义
// ===========================================

/// 成功退出码：等于 `exit(0)`
/// 当内核正常运行时，使用这个值退出 QEMU
const EXIT_SUCCESS: u32 = 0x5555;

/// 失败标志：用于编码自定义退出码
const EXIT_FAILURE_FLAG: u32 = 0x3333;

/// 失败退出码：等于 `exit(1)`
/// 通过编码函数将退出码 1 编码为 QEMU 可以识别的格式
/// 编码规则：将退出码左移 16 位，然后与失败标志进行 OR 操作
const EXIT_FAILURE: u32 = exit_code_encode(1);

/// 重置退出码：让 QEMU 重置虚拟机
const EXIT_RESET: u32 = 0x7777;

// ===========================================
// QEMUExit Trait 定义
// ===========================================

/// QEMU 退出功能的 Trait（接口）
/// 
/// # Trait 说明
/// Trait 定义了所有 QEMU 退出实现必须提供的方法
/// 不同的架构（RISC-V、x86 等）可能需要不同的实现
/// 
/// # 为什么使用 Trait？
/// 使用 Trait 可以让代码更通用，不同的架构可以实现不同的退出方式
/// 这是 Rust 多态性（Polymorphism）的体现
pub trait QEMUExit {
    /// 使用指定的退出码退出 QEMU
    /// 
    /// # 参数
    /// - `code`: 退出码（32 位无符号整数）
    /// 
    /// # 返回值
    /// 永不返回（返回类型为 !）
    /// 
    /// # 注意
    /// 对于 x86 架构，退出码会被 QEMU 进行二进制 OR 操作（与 0x1）
    fn exit(&self, code: u32) -> !;

    /// 使用成功退出码退出 QEMU（相当于 `exit(0)`）
    /// 
    /// # 注意
    /// 对于 x86 架构，此功能可能不可用
    fn exit_success(&self) -> !;

    /// 使用失败退出码退出 QEMU（相当于 `exit(1)`）
    fn exit_failure(&self) -> !;
}

// ===========================================
// RISCV64 实现
// ===========================================

/// RISC-V 64 位架构的 QEMU 退出配置
/// 
/// # 结构体说明
/// 这个结构体存储了测试设备的地址
/// 在 RISC-V 架构中，QEMU 的测试设备位于固定的内存地址
pub struct RISCV64 {
    /// 测试设备（sifive_test）的内存映射地址
    /// 通过向这个地址写入特定值可以控制 QEMU 的退出行为
    addr: u64,
}

/// 使用失败标志编码退出码
/// 
/// # 参数
/// - `code`: 要编码的退出码（通常是进程退出码，如 1、2、3 等）
/// 
/// # 返回值
/// 编码后的退出码，格式为：(code << 16) | EXIT_FAILURE_FLAG
/// 
/// # 编码规则
/// 将退出码左移 16 位，然后与失败标志（0x3333）进行 OR 操作
/// 例如：exit_code_encode(1) = (1 << 16) | 0x3333 = 0x00013333
/// 
/// # const fn 说明
/// const fn 是常量函数，可以在编译时执行
/// 这使得 EXIT_FAILURE 可以在编译时计算，不需要运行时开销
const fn exit_code_encode(code: u32) -> u32 {
    // << 是左移运算符，将退出码左移 16 位（放在高 16 位）
    // | 是按位或运算符，将失败标志放在低 16 位
    (code << 16) | EXIT_FAILURE_FLAG
}

/// RISCV64 的关联函数和方法
impl RISCV64 {
    /// 创建一个新的 RISCV64 实例
    /// 
    /// # 参数
    /// - `addr`: 测试设备的内存地址
    /// 
    /// # 返回值
    /// RISCV64 结构体实例
    /// 
    /// # const fn 说明
    /// const fn 允许在常量上下文中调用，所以可以创建 const 常量
    pub const fn new(addr: u64) -> Self {
        // 结构体字面量语法：创建 RISCV64 实例
        RISCV64 { addr }
    }
}

/// 为 RISCV64 实现 QEMUExit trait
/// 
/// 这个实现提供了在 RISC-V 64 位架构下退出 QEMU 的具体方法
impl QEMUExit for RISCV64 {
    /// 使用指定退出码退出 QEMU
    /// 
    /// # 实现原理
    /// 1. 检查退出码是否是特殊值（EXIT_SUCCESS、EXIT_FAILURE、EXIT_RESET）
    /// 2. 如果是特殊值，直接使用；否则进行编码
    /// 3. 使用汇编指令将退出码写入测试设备地址
    /// 4. 如果写入后 QEMU 仍未退出，进入无限循环等待
    /// 
    /// # 内存映射 I/O（MMIO）说明
    /// QEMU 的测试设备是通过内存映射 I/O 访问的
    /// 向特定内存地址写入数据会被 QEMU 解释为设备操作，而不是普通的内存写入
    fn exit(&self, code: u32) -> ! {
        // 如果退出码不是特殊值，需要编码
        // match 表达式进行模式匹配
        let code_new = match code {
            // | 表示"或"，匹配三个特殊值中的任意一个
            EXIT_SUCCESS | EXIT_FAILURE | EXIT_RESET => code,  // 直接使用原值
            // _ 是通配符，匹配所有其他情况
            _ => exit_code_encode(code),  // 对退出码进行编码
        };

        // unsafe 块：汇编代码无法由 Rust 类型系统保证安全
        unsafe {
            // asm! 宏：内联汇编
            // "sw {0}, 0({1})" 是 RISC-V 汇编指令
            // - sw: Store Word（存储字，32位）
            // - {0}: 第一个操作数（code_new），写入的值
            // - 0({1}): 第二个操作数（self.addr），目标地址
            //   0({1}) 表示：地址 = self.addr + 0（即直接使用 self.addr 作为地址）
            asm!(
                "sw {0}, 0({1})",  // 将 code_new 的值存储到地址 self.addr
                in(reg) code_new,  // 输入：code_new 放入某个寄存器
                in(reg) self.addr  // 输入：self.addr 放入某个寄存器
            );

            // 如果 QEMU 退出尝试失败，进入无限循环
            // 这可以防止程序继续执行，造成不可预期的行为
            // 
            // 为什么不能调用 panic!()？
            // 因为 panic!() 可能会调用 exit_failure()，而 exit_failure() 又会调用 exit()
            // 如果 exit() 本身失败了，再次调用可能会造成无限递归
            loop {
                // wfi: Wait For Interrupt（等待中断）
                // 让 CPU 进入低功耗等待状态，等待中断唤醒
                // 
                // options(nomem, nostack):
                // - nomem: 表示汇编代码不访问内存（只访问寄存器）
                // - nostack: 表示汇编代码不需要栈空间
                // 这些选项帮助编译器进行优化
                asm!("wfi", options(nomem, nostack));
            }
        }
    }

    /// 成功退出：调用 exit(EXIT_SUCCESS)
    fn exit_success(&self) -> ! {
        self.exit(EXIT_SUCCESS);
    }

    /// 失败退出：调用 exit(EXIT_FAILURE)
    fn exit_failure(&self) -> ! {
        self.exit(EXIT_FAILURE);
    }
}

// ===========================================
// 全局常量定义
// ===========================================

/// 虚拟测试设备的基地址
/// 这是 QEMU 在 RISC-V 虚拟机中映射的测试设备地址
/// 向这个地址写入特定值可以控制 QEMU 的行为
const VIRT_TEST: u64 = 0x100000;

/// 全局 QEMU 退出句柄
/// 
/// # 说明
/// 这是一个全局常量，可以在程序的任何地方使用
/// 不需要每次都创建新实例，节省内存和时间
/// 
/// # 使用示例
/// ```rust
/// use crate::board::QEMUExit;
/// QEMU_EXIT_HANDLE.exit_success();  // 成功退出
/// QEMU_EXIT_HANDLE.exit_failure(); // 失败退出
/// ```
pub const QEMU_EXIT_HANDLE: RISCV64 = RISCV64::new(VIRT_TEST);

// ===========================================
// Rust 知识补充：
// ===========================================
// 1. Trait（特质）
//    - trait 定义了一组方法签名，类似接口
//    - impl Trait for Type 为类型实现 trait
//    - 实现了 trait 的类型可以调用 trait 的方法
//    - trait 支持多态：同一个 trait 可以有多个实现
// 
// 2. 关联函数（Associated Function）
//    - impl 块中可以定义两种函数：
//      * 关联函数：第一个参数不是 &self（如 new()）
//      * 方法：第一个参数是 &self、&mut self 或 self
//    - 关联函数通过 Type::function() 调用
//    - 方法通过 instance.method() 调用
// 
// 3. const 关键字
//    - const 用于定义常量（编译时已知的值）
//    - const fn 是常量函数，可以在编译时执行
//    - const 常量必须是 'static 生命周期
// 
// 4. 位运算
//    - << : 左移运算符（相当于乘以 2^n）
//    - >> : 右移运算符（相当于除以 2^n）
//    - |  : 按位或运算符
//    - &  : 按位与运算符
//    - ^  : 按位异或运算符
// 
// 5. 内存映射 I/O（MMIO）
//    - 某些硬件设备通过内存地址访问，而不是专门的 I/O 端口
//    - 读写这些地址会被硬件解释为设备操作
//    - 在 QEMU 中，测试设备就是通过 MMIO 访问的
// 
// 6. 内联汇编
//    - asm! 宏允许在 Rust 代码中直接使用汇编指令
//    - 需要 unsafe 块，因为汇编无法由类型系统保证安全
//    - 可以精确控制寄存器和内存的访问
