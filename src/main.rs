#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]

use core::fmt;
use core::fmt::Write;
use crate::console::*;
use crate::sbi::shutdown;
use crate::MyLogger;
use log::{error, info,debug,warn,trace,LevelFilter};

#[macro_use]
mod lang_items;
mod sbi;
mod console;

global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    info!(".text [{:#x}, {:#x})", sbss as usize, ebss as usize);
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

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
#[no_mangle]
pub fn rust_main(){
    init();
    print_section();
    let str = "hello world";
    error!("{}",str);
    warn!("{}",str);
    info!("{}",str);
    debug!("{}",str);
    trace!("{}",str);

    shutdown();
}