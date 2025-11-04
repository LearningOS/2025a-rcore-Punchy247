//! ===========================================
//! SBI 调用封装模块 (sbi.rs)
//! ===========================================
//! SBI = Supervisor Binary Interface（监督者二进制接口）
//! 这是一个标准化的接口，允许操作系统内核与底层固件（如 OpenSBI）通信
//! SBI 提供了多种功能，如控制台输出、系统关闭等
//! 
//! 在 RISC-V 架构中，SBI 调用通过 ecall（Environment Call）指令实现
//! ecall 会触发一个环境调用异常，由底层的监控程序（M-mode）处理
//! 
//! 参考文档：RISC-V Supervisor Binary Interface Specification

use core::arch::asm;  // 用于内联汇编，直接生成机器指令

// SBI 扩展 ID：控制台输出字符
// 这是一个预定义的常量，表示我们要调用"控制台输出字符"功能
const SBI_CONSOLE_PUTCHAR: usize = 1;

/// 通用的 SBI 调用函数
/// 
/// # 参数说明
/// - `which`: SBI 扩展 ID，指定要调用哪个 SBI 功能（如控制台输出、定时器等）
/// - `arg0`, `arg1`, `arg2`: 传递给 SBI 调用的三个参数
/// 
/// # 返回值
/// 返回 SBI 调用执行后的结果（通常放在寄存器 x10/a0 中）
/// 
/// # SBI 调用约定（RISC-V Calling Convention）
/// RISC-V 函数调用使用以下寄存器约定：
/// - a0 (x10): 第一个参数，也用于返回值
/// - a1 (x11): 第二个参数
/// - a2 (x12): 第三个参数
/// - a7 (x17): 系统调用号或扩展 ID
/// 
/// # 内联汇编说明
/// `#[inline(always)]` 告诉编译器总是内联这个函数，避免函数调用开销
/// 这对于频繁调用的底层函数很重要
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    // ret 用于保存返回值
    let mut ret;
    
    // unsafe 块允许我们使用内联汇编
    // 汇编代码直接操作 CPU 寄存器，无法由 Rust 的类型系统保证安全
    unsafe {
        // asm! 宏用于内联汇编
        // 格式：asm!("汇编代码模板", 约束条件)
        asm!(
            // "li x16, 0" - Load Immediate（加载立即数）
            // 将寄存器 x16 设置为 0
            // 在某些 SBI 实现中，x16 用于指定 SBI 版本，设为 0 表示使用默认版本
            "li x16, 0",
            
            // "ecall" - Environment Call（环境调用）
            // 这是触发 SBI 调用的关键指令
            // 执行 ecall 后，CPU 会跳转到异常处理程序（通常由 OpenSBI 提供）
            // OpenSBI 会根据寄存器中的参数执行相应的功能，然后将结果放回寄存器
            "ecall",
            
            // 约束条件：指定哪些寄存器用于输入/输出
            // "x10" 是 RISC-V 的 a0 寄存器（第一个参数/返回值寄存器）
            // inlateout 表示 x10 既是输入也是输出
            // arg0 => ret 表示：将 arg0 传入 x10，然后将 x10 的值赋给 ret
            inlateout("x10") arg0 => ret,
            
            // in("x11") 表示将 arg1 传入 x11 寄存器（a1）
            in("x11") arg1,
            
            // in("x12") 表示将 arg2 传入 x12 寄存器（a2）
            in("x12") arg2,
            
            // in("x17") 表示将 which 传入 x17 寄存器（a7）
            // a7 通常用于存放系统调用号或扩展 ID
            in("x17") which,
        );
    }
    
    // 返回 SBI 调用的结果
    ret
}

/// 使用 SBI 调用在控制台输出一个字符
/// 
/// # 参数
/// - `c`: 要输出的字符（以 usize 类型传递）
/// 
/// # 实现原理
/// 这个函数封装了对 SBI 控制台输出功能的调用
/// 在 QEMU 环境中，字符会通过 UART（串口）输出到宿主机的终端
/// 
/// # 使用示例
/// ```rust
/// console_putchar('H' as usize);  // 输出字符 'H'
/// console_putchar('\n' as usize); // 输出换行符
/// ```
pub fn console_putchar(c: usize) {
    // 调用通用 SBI 函数，传入扩展 ID 和字符
    // 后两个参数设为 0（因为控制台输出只需要一个参数）
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

// 导入 QEMU 退出处理相关的类型和常量
use crate::board::QEMUExit;

/// 使用 SBI 调用关闭内核（实际上是通过 QEMU 退出模拟器）
/// 
/// # 返回值
/// 这个函数永远不会返回（返回类型为 !，表示发散函数）
/// 
/// # 使用场景
/// - 内核发生 panic 时调用
/// - 正常关闭操作系统时调用
/// 
/// # 注意
/// 在真实的硬件环境中，shutdown 的实现会有所不同
/// 可能需要通过 ACPI、设备特定寄存器等方式实现
pub fn shutdown() -> ! {
    // 调用 QEMU 退出处理器，以失败状态退出
    // 这会让 QEMU 模拟器退出，退出码为 1（表示失败）
    // 如果内核正常运行完毕，应该调用 exit_success() 而不是 exit_failure()
    crate::board::QEMU_EXIT_HANDLE.exit_failure();
}
