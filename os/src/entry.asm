    // ===========================================
    // 汇编入口文件 entry.asm
    // ===========================================
    // 这个文件是操作系统内核的入口点，使用 RISC-V 汇编语言编写
    // 它负责在 Rust 代码运行前进行必要的初始化工作
    
    // .section 指令定义了一个节（section）
    // .text.entry 表示这是一个特殊的代码段，用于存放入口代码
    // 链接器会确保这段代码位于程序的最开始位置
    .section .text.entry
    
    // .globl 声明符号 _start 是全局可见的，可以被链接器使用
    // _start 是程序的入口点，链接器会从这里开始执行
    .globl _start
    
// 程序入口点标签
_start:
    // la 指令（Load Address）将 boot_stack_top 的地址加载到栈指针寄存器 sp
    // sp（Stack Pointer）是 RISC-V 架构中的栈指针寄存器（x2）
    // 栈是用于存储局部变量和函数调用信息的连续内存区域
    // 在调用 Rust 函数之前，必须先设置好栈，因为函数调用需要栈来保存返回地址和局部变量
    la sp, boot_stack_top
    
    // call 指令调用 rust_main 函数，这是 Rust 代码的入口点
    // call 指令相当于：jal ra, rust_main
    // - jal (Jump And Link) 会跳转到 rust_main，同时将返回地址保存到 ra 寄存器
    // - 但在裸机环境下，rust_main 通常不会返回（使用 ! 返回类型表示永不返回）
    call rust_main

    // ===========================================
    // 栈空间定义
    // ===========================================
    // .section .bss.stack 定义了一个 BSS（Block Started by Symbol）段的栈区域
    // BSS 段用于存放未初始化的全局变量，在程序启动时会被清零
    // 这里我们为操作系统内核预分配一块栈空间
    .section .bss.stack
    
    // 声明 boot_stack_lower_bound 为全局符号，表示栈的底部边界
    // 栈通常从高地址向低地址增长，所以 lower_bound 是栈的最高地址
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    // .space 指令分配指定大小的未初始化内存空间
    // 4096 * 16 = 65536 字节 = 64 KB
    // 这是一个固定大小的栈，对于简单的内核来说已经足够
    // 注意：在生产环境中，栈大小可能需要根据实际情况调整
    .space 4096 * 16
    
    // 声明 boot_stack_top 为全局符号，表示栈的顶部（实际是最低地址）
    // 栈从高地址（boot_stack_lower_bound）向低地址（boot_stack_top）增长
    .globl boot_stack_top
boot_stack_top: