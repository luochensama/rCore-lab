### 引言

#### 本章目的

本章的目的是实现操作系统最基本的一个功能：让应用和硬件隔离，即在裸机上仅依靠RISC-V基础指令集来搭起一个“三叶虫”阶段的操作系统来为上层应用提供执行环境。具体的目标为让我们的“三叶虫”输出“Hello World！”，为什么第一步要实现输出功能呢？回想一下，任何一种语言，在编写程序的过程中都免不了`debug`的过程，而裸机是没有所谓的`debug`功能的，所以这功能需要我们手动实现，方便之后更复杂功能的实现。

#### 本章效果

```
[rustsbi] RustSBI version 0.1.1
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|

[rustsbi] Platform: QEMU (Version 0.1.0)
[rustsbi] misa: RV64ACDFIMSU
[rustsbi] mideleg: 0x222
[rustsbi] medeleg: 0xb1ab
[rustsbi-dtb] Hart count: cluster0 with 1 cores
[rustsbi] Kernel entry: 0x80200000
Hello, world!
.text [0x80200000, 0x80202000)
.rodata [0x80202000, 0x80203000)
.data [0x80203000, 0x80203000)
boot_stack [0x80203000, 0x80213000)
.bss [0x80213000, 0x80213000)
Panicked at src/main.rs:46 Shutdown machine!
```

#### RustSBI

RISC-V指令集的SBI标准规定了类Unix操作系统之下的运行环境规范，显然RustSBI是针对Rust实现的一个规范。RISC-V架构中，存在着定义于操作系统之下的运行环境。这个运行环境不仅将引导启动RISC-V下的操作系统， 还将常驻后台，为操作系统提供一系列二进制接口，以便其获取和操作硬件信息。 RISC-V给出了此类环境和二进制接口的规范，称为“操作系统二进制接口”，即“SBI”。

其实RustSBI就相当于为我们实现了操作系统所需要的一些最基础的内容，如bootloader，输出字符，让我们能屏蔽一些不必要的硬件细节。毕竟我们的目标还是理解到底何为操作系统，而不是RISC-V。



### 应用程序执行环境与平台支持

#### 采用RISC-V架构的原因

x86 架构为了在升级换代的同时保持对基于旧版架构应用程序/内核的兼容性，存在大量的历史包袱，也就是一些对于目前的应用场景没有任何意义，但又必须花大量时间正确设置才能正常使用 CPU 的奇怪设定。为了建立并维护架构的应用生态，这确实是必不可少的，但站在教学的角度几乎完全是在浪费时间。而新生的 RISC-V 架构十分简洁，架构文档需要阅读的核心部分不足百页，且这些功能已经足以用来构造一个具有相当抽象能力的内核了。

总结一下就是RISC-V够简洁，便于教学学习使用。

#### Rust 核心库

在编译Rust程序的时候可以选择编译的环境，如果我们把环境换成 `riscv64gc-unknown-none-elf` ，程序将返回以下结果：

```shell
cargo run --target riscv64gc-unknown-none-elf
   Compiling os v0.1.0 (/home/luochensama/os)
error[E0463]: can't find crate for `std`
  |
  = note: the `riscv64gc-unknown-none-elf` target may not support the standard library
  = note: `std` is required by `os` because it does not declare `#![no_std]`
  = help: consider building the standard library from source with `cargo build -Zbuild-std`
```

没有 `riscv64gc-unknown-none-elf` 的原因并不是真的没有安装这个环境，而是因为该环境根本就没有std库，std库是针对有操作系统的环境的rust标准库，对于没有操作系统的库，仅仅实现了core库，该库已经足以实现rust语言的大多数特性。所以接下来，我们需要移除掉std库的依赖。

### 移除标准库依赖

#### 改造程序

需要移除的是 `println!` `main` ，这两个都是 `std` 库为我们实现的功能，还需要加入两个标签 `#![no_std]` 和 `	#![no_main]` ，这些还不够，如果要正确通过编译的话还需要实现 `panic!` 宏，在代码上需要实现 `#[panic_handler]`，实现了之后才能正确的编译程序。 `PanicInfo` 中有错误的具体信息。

> 因为工程目录的命名不同 ，原工程命名为os，在本笔记github中目录为rCore-lab1，请自行代换。对于输出文件的命名，可以在 `cargo.toml`文件下更改，这样执行命令时就不需要修改了。

```rust
// os/src/lang_items.rs
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// os/src/main.rs
#![no_std]
#![no_main]

mod lang_items;

// os/.cargo/config
[build]
target = "riscv64gc-unknown-none-elf" 
```

#### 分析程序

下面使用的三个工具：

- `file` 该命令可以识别文件的类型，根据输出信息可以得知移除依赖后的程序是一个合法的RISC-V x64的程序。
- `rust-readobj ` 该命令可以读取到文件头信息，其中最关键的是 `Entry：0x0` 这说明了文件的入口地址，之后也要多次使用到该命令。
- `rust-objdump` 反汇编程序，根据结果可以得知我们的程序反汇编出来是空的。这是因为还没有设置合法的入口函数。

 ```shell
 [文件格式]
 file target/riscv64gc-unknown-none-elf/debug/os
 target/riscv64gc-unknown-none-elf/debug/os: ELF 64-bit LSB executable, UCB RISC-V, ......
 
 [文件头信息]
 rust-readobj -h target/riscv64gc-unknown-none-elf/debug/os
    File: target/riscv64gc-unknown-none-elf/debug/os
    Format: elf64-littleriscv
    Arch: riscv64
    AddressSize: 64bit
    ......
    Type: Executable (0x2)
    Machine: EM_RISCV (0xF3)
    Version: 1
    Entry: 0x0
    ......
    }
 
 [反汇编导出汇编程序]
 rust-objdump -S target/riscv64gc-unknown-none-elf/debug/os
    target/riscv64gc-unknown-none-elf/debug/os:       file format elf64-littleriscv
 ```

### 构建用户态执行环境

这一节的内容是构建在用户态环境下的，还有底层的Linux支持，接下来的一节中会在内核态中来实现输出功能。目的是加深对执行环境的理解。体现在qemu中一个是使用 `qemu-riscv64` 命令，一个是使用 `qemu-system-riscv64` 命令，利用 这两种命令来切换用户态和内核态。在用户态中，我们可以直接调用 `ecall`系统调用，这是因为底层的Linux系统已经准备好了该功能 。回顾一下上一节的代码，我们已经完全移除了标准库的依赖，目前的程序是空的 ，接下来我们要在用户态下实现“恐龙虾”操作系统。

#### 用户态最小化执行环境

为了正确的运行一段“有意义”的程序，我们需要入口函数和退出函数。

入口函数：

```rust
// os/src/main.rs
#[no_mangle]
extern "C" fn _start() {
    loop{};
}
```

退出函数（如果没有该函数会出现段错误的问题）：

```rust
// os/src/main.rs
#![no_std]
#![no_main]
#![feature(llvm_asm)]

mod lang_items;

const SYSCALL_EXIT: usize = 93;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize = 0;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}

pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
}

#[no_mangle]
extern "C" fn _start() {
    sys_exit(9);
}
```

此时可以使用之前介绍的三个工具来重新分析我们的程序，可以看到的是入口不再是  `0` 了，而是有了一个正确地址，这是入口函数的功劳。 

```shell
[文件格式]
luochensama@luochensama-G3-3590:~/rcore/lab/rCore-lab1$ file target/riscv64gc-unknown-none-elf/debug/os
target/riscv64gc-unknown-none-elf/debug/os: ELF 64-bit LSB executable, UCB RISC-V, version 1 (SYSV), statically linked, with debug_info, not stripped

[文件头信息]
luochensama@luochensama-G3-3590:~/rcore/lab/rCore-lab1$ rust-readobj -h target/riscv64gc-unknown-none-elf/debug/os
File: target/riscv64gc-unknown-none-elf/debug/os
Format: elf64-littleriscv
Arch: riscv64
AddressSize: 64bit
LoadName: <Not found>
...
Entry: 0x11166
...

[反汇编导出程序]
luochensama@luochensama-G3-3590:~/rcore/lab/rCore-lab1$ rust-objdump -S target/riscv64gc-unknown-none-elf/debug/os

target/riscv64gc-unknown-none-elf/debug/os:     file format elf64-littleriscv

Disassembly of section .text:
...

```

正确运行：

```    shell
luochensama@luochensama-G3-3590:~/rcore/lab/rCore-lab1$ cargo build --target riscv64gc-unknown-none-elf
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
luochensama@luochensama-G3-3590:~/rcore/lab/rCore-lab1$ qemu-riscv64 target/riscv64gc-unknown-none-elf/debug/os; echo $?
9
```

通过该程序的编写，我们可以发现的是在用户态下，要编写一个只调用了 `SYSCALL_EXIT`  的程序是很简单的一件事，这是因为底层的Linux操作系统帮助我们完成了程序加载、程序退出、资源分配、资源回收等各种琐事。在下一节中，我们就需要自己来实现这之中的某些功能了 。

####  有显示支持的用户态执行环境

Rust 的 core 库内建了以一系列帮助实现显示字符的基本 Trait 和数据结构，函数等，我们可以对其中的关键部分进行扩展，就可以实现定制的 `println!` 功能。

首先封装一下对 `SYSCALL_WRITE` 系统调用。这个是 Linux 操作系统内核提供的系统调用，其 `ID` 就是 `SYSCALL_WRITE`。

```rust
const SYSCALL_WRITE: usize = 64;

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
  syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}
```

然后实现基于 `Write` Trait 的数据结构，并完成 `Write` Trait 所需要的 `write_str` 函数，并用 `print` 函数进行包装。

```rust
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sys_write(1, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}
```

最后，实现基于 `print` 函数，实现Rust语言 **格式化宏** ( [formatting macros](https://doc.rust-lang.org/std/fmt/#related-macros) )。

```rust
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
```

这里面的代码不需要完全读懂，尤其是定义宏那段 ，只需要知道Rust是这样来定义 `println!`宏即可。

如此便可成功实现输出功能。

```shell
cargo build --target riscv64gc-unknown-none-elf	
  Finished dev [unoptimized + debuginfo] target(s) in 0.61s

$ qemu-riscv64 target/riscv64gc-unknown-none-elf/debug/os; echo $?
  Hello, world!
  9
```

 至此，“恐龙虾“操作系统构建完毕。

### 构建裸机执行环境

在这一节中，我们将脱离舒适区，在裸机外加上RustSBI的支持下完成我们的“三叶虫”系统，其中RustSBI只是提供了最基础的引导启动，字符输出等功能。还有一系列的功能是需要我们一步步去完善的。

#### 了解硬件组成和裸机启动过程

QEMU中模拟的  `risc-v64`  硬件中物理内存空间主要有两段比较重要的区域：

- `VIRT_DRAM`：这是计算机的物理内存，DRAM的内存起始地址是 `0x80000000` ，缺省大小为128MB。在本书中一般限制为8MB。
- `VIRT_UART0`：这是串口的控制寄存器区域，串口相关的寄存器起始地址是 `0x10000000` ，范围是 `0x100` ，我们通过访问这段特殊的区域来实现字符输入输出的管理与控制。

在裸机启动过程中，根据makefile中设置好的参数，我们的程序首先会在地址 `0x1000` 中执行硬件中的一小段固定引导代码，之后会跳转到 `0x80000000` 执行RustSBI的引导程序，之后再跳转到 `0x80200000`中启动我们编写的操作系统。

> 为啥在 `0x80000000` 放置 `Bootloader` ？因为这是QEMU的硬件模拟代码中设定好的 `Bootloader` 的起始地址。
>
> 为啥在 `0x80200000` 放置 `os` ？因为这是 `Bootloader--RustSBI` 的代码中设定好的 `os` 的起始地址。

#### 实现关机功能

如果在裸机上的应用程序执行完毕并通知操作系统后，那么操作系统就没事干了，实现正常关机是一个合理的选择。在裸机坏境下，如果想要正确的退出程序，可以使用SBI库中的 `SBI_SHUTDOWN`功能。

```rust
// main.rs
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
#[no_mangle]
extern "C" fn _start() {
    shutdown();
}

// sbi.rs
pub(crate) fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret = 0;
    unsafe {
        llvm_asm!("ecall" // 执行指令
            : "={x10}" (ret) // output
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which) // input
            : "memory" // 强制修改内存单元
            : "volatile" // rust用于识别指令的格式
        );
    }
    ret
}
```

调用该函数使用的也是 `ecall` ，回忆一下，在上一节中使用Linux提供的系统调用也是用的 `ecall` ，这并不会发生冲突，这是因为它们所在的特权级和特权级转换是不一样的。简单地说，应用程序位于最弱的用户特权级（User Mode），操作系统位于很强大的内核特权级（Supervisor Mode），RustSBI位于完全掌控机器的机器特权级（Machine Mode），通过 `ecall` 指令，可以完成从弱的特权级到强的特权级的转换。之后编译连接会生成 `ELF`文件，需要转换为 `BIN` 文件才可以在QEMU上运行。

> ELF文件：类Unix下的二进制可执行文件 

#### 设置正确的程序内存布局

> 本节代码运行方式：
>
> cargo build --release
>
> rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
>
> qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000

在完成了上述步骤之后程序是无法正常的退出的，通过 `rust-readobj` 工具分析的时候可以发现是程序的入口地址不对，这是由什么引起的呢？让我们回顾一下用户态到内核态的过程中我们做了什么：切换了QUME的启动模式 ，启动模式变成了裸机模式，指定了SBI的入口地址和程序的入口地址。但是这里的“指定”其实只是告诉了RustSBI我们的程序入口地址，但是对于生成的 `ELF` 文件，它的入口地址是由连接器来决定的，所以接下来，我们需要写自己的ld文件。

```ld
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
```

在ld文件中设置了目标平台，入口函数名称，程序入口地址以及内部section的分布格式。指定完连接文件后，再编译一次，使用 `rust-readobj` 工具查看后就会发现入口地址为 `0x80200000`了  。

#### GDB的使用

在上面配置完linker之后，我们重新编译运行发现还是无法正确的退出程序，这时就需要Debug的帮助来帮我们找出错误了。

```shell
在一个终端执行如下命令：
qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 -S -s

在另外一个终端执行如下命令：
riscv64-unknown-elf-gdb target/riscv64gc-unknown-none-elf/release/os
 target remote :1234
 break *0x80200000
 c
 info reg
 si
 info reg
```

可以看到的是在第二次查看寄存器内容的时候，`sp` 和 `pc` 两个寄存器的内容变化异常，这是因为没有正确设置栈空间导致的。

#### 正确配置栈空间布局

为了正确的设置好栈的空间布局，我们需要引入一段汇编代码。

```assembly
# os/src/entry.asm
    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
```

`.global` 是让标识符对链接器可见，.section是指定接下来的汇编代码会放在哪个section中。在这个汇编文件中，我们将程序的入口设置为 `rust_main` ，且安置了栈空间大小为 `4096*16` 。

这之后我们需要在rust中使用这段汇编代码。

```rust
// os/src/main.rs
#![no_std]
#![no_main]
#![feature(global_asm)]

mod lang_items;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    loop {}
}
```

这样一来程序就可以正常退出了 。

#### 清零 .bss 段

`.bss`  段是未初始化的全局变量的一块内存区域，正常状况下全部都为0，所以我们的系统需要提供这一功能。这只需要我们在主函数中实现 `clear_bss()`  函数，此函数属于执行环境，并在执行环境调用 应用程序的 `rust_main` 主函数前，把 `.bss` 段的全局数据清零。

```rust
// os/src/main.rs
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
```

#### 裸机实现打印函数

得益于RustSBI的帮助，我们实现打印函数比起前一节只需要将系统调用更换为SBI调用即可。

```rust
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        //sys_write(STDOUT, s.as_bytes());
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}
```

还可以添加一个`panic!` 宏

```rust
#![feature(panic_info_message)]

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("Panicked at {}:{} {}", location.file(), location.line(), info.message().unwrap());
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    shutdown()
}
```

### 理解应用程序和执行环境

本节内容主要是介绍了一些汇编原理和编译原理，只做粗略总结。

#### 汇编知识

汇编是最底层的直接和硬件交互的语言，通过操作各个寄存器来进行编程。在函数调用的时候，为了能够正确返回到调用位置，需要使用栈来存放各寄存器本身的值，这里分 `调用者保存寄存器` 和 `被调用者保存寄存器` ，这是为了运行的效率而区分开的。还有五个比较特殊的寄存器如下：

> - zero(x0) 之前提到过，它恒为零，函数调用不会对它产生影响；
> - ra(x1) 是调用者保存的，不过它并不会在每次调用子函数的时候都保存一次，而是在函数的开头和结尾保存/恢复即可。虽然 `ra` 看上去和其它被调用者保存寄存器保存的位置一样，但是它确实是调用者保存的。
> - sp(x2) 是被调用者保存的。这个是之后就会提到的栈指针寄存器。
> - gp(x3) 和 tp(x4) 在一个程序运行期间都不会变化，因此不必放在函数调用上下文中。它们的用途在后面的章节会提到。

每一个函数在调用的时候都会有一个属于自己的 `栈帧` 在每个函数的栈帧中都会有一个独属于自己的栈空间，来实现函数的功能，在访问栈中内容的时候是通过 `sp` 加上一个偏移量来访问的。保存顺序为（高地址到低地址）：

- `ra` 寄存器保存其返回之后的跳转地址，是一个调用者保存寄存器；
- 父亲栈帧的结束地址 `fp` ，是一个被调用者保存寄存器；
- 其他被调用者保存寄存器 `s1` ~ `s11` ；
- 函数所使用到的局部变量。

不过在编译器的帮助下，我们只需要分配好栈的空间，其他事情编译器会帮助我们解决。

#### 程序内存布局

当源代码被编译链接为可执行文件之后，这之中有代码和数据，他们本质上都是些二进制的字符串，需要我们来为这些字符串规划布局。回忆一下linker文件中的各个section名，这之中只有 `text` 是代码段，其余都是全局变量和局部变量。具体如下：

- 已初始化数据段保存程序中那些已初始化的全局数据，分为 `.rodata` 和 `.data` 两部分。前者存放只读的全局数据，通常是一些常数或者是 常量字符串等；而后者存放可修改的全局数据。
- 未初始化数据段 `.bss` 保存程序中那些未初始化的全局数据，通常由程序的加载者代为进行零初始化，即将这块区域逐字节清零；
- **堆** （heap）区域用来存放程序运行时动态分配的数据，如 C/C++ 中的 malloc/new 分配到的数据本体就放在堆区域，它向高地址增长；
- **栈** （stack）区域不仅用作函数调用上下文的保存与恢复，每个函数作用域内的局部变量也被编译器放在它的栈帧内，它向低地址增长。

> **局部变量与全局变量**
>
> 在一个函数的视角中，它能够访问的变量包括以下几种：
>
> - 函数的输入参数和局部变量：保存在一些寄存器或是该函数的栈帧里面，如果是在栈帧里面的话是基于当前 sp 加上一个偏移量来访问的；
> - 全局变量：保存在数据段 `.data` 和 `.bss` 中，某些情况下 gp(x3) 寄存器保存两个数据段中间的一个位置，于是全局变量是基于 gp 加上一个偏移量来访问的。
> - 堆上的动态变量：本体被保存在堆上，大小在运行时才能确定。而我们只能 *直接* 访问栈上或者全局数据段中的 **编译期确定大小** 的变量。 因此我们需要通过一个运行时分配内存得到的一个指向堆上数据的指针来访问它，指针的位宽确实在编译期就能够确定。该指针即可以作为局部变量 放在栈帧里面，也可以作为全局变量放在全局数据段中。

### 练习

#### 编程练习

##### 彩色化log

本次实验需要实现彩色化的log，彩色化的实现使用ASCII码提供的功能即可，log功能官方提供了一个crate帮助我们实现。只需要实现Log trait就可以了。然后在主函数的时候调用一下init()。

```rust
pub fn init() {
    static LOGGER: MyLogger = MyLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}

pub struct MyLogger;
fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn => 93,  // BrightYellow
        Level::Info => 34,  // Blue
        Level::Debug => 32, // Green
        Level::Trace => 90, // BrightBlack
    }
}
impl Log for MyLogger{
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        println!("\x1b[{}m[{}][{}] {}\x1b[0m",level_to_color_code(record.level()),record.level(),0,record.args());
    }

    fn flush(&self) {}
}
```

##### 输出section地址

在rust里面用extern可以导入其他文件中定义的符号（这一部分的具体实现原理暂时不太清楚）。代码如下：

```rust
fn print_section() {
    extern "C" {
        fn stext();
        fn etext();
        fn sdata();
        fn edata();
        fn srodata();
        fn erodata();
        fn sbss();
        fn ebss();
    }
    info!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    info!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    info!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    info!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
}
```

##### rust传递环境变量

在log里面使用了官方提供的宏 `option_env!` 这个宏用来读取环境变量中的值，比如在log中我们需要设定log的显示等级，可以在cargo编译时使用 `LOG=INFO cargo build --release` 命令来传递环境变量，也可以使用make工具 `make run LOG=INFO`来传递变量。



#### 问答作业

暂时答不上来

### 结语

这次实验的笔记写的有点冗余了，之后的章节会调整。目前的知识储备有点不够完成这个实验。



