#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process::exit;

/// Head: 0 u8: app number, then two u8: app size, the size of u8...

const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0xffff_ffc0_8010_0000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];
fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_terminate() {
    println!("[ABI:Terminate] ArceOS Terminate...");
    exit(0);
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let start = PLASH_START as *const u8;
    let apps_num = unsafe {
        let num = core::slice::from_raw_parts(start, 1);
        num[0]
    };

    println!("Load payload ...\n");
    let mut start_now = PLASH_START + 1;
    for i in 0..apps_num {
        let size: &[u8] = unsafe { core::slice::from_raw_parts(start_now as *const u8, 2) };

        let apps_size = (((size[0] as usize) << 8) + size[1] as usize) as usize; 
        let apps_start = (start_now + 2) as *const u8;
        let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
        println!("app{}_content: {:?}; address: [{:?}]", i, code, code.as_ptr());
        start_now += 2 + apps_size;

        // app running aspace
        // SBI(0x80000000) -> App <- Kernel(0x80200000)
        // 0xffff_ffc0_0000_0000
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, apps_size)
        };
        run_code.copy_from_slice(code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        // println!("Execute app{} ...", i);
        // // execute app
        // unsafe { core::arch::asm!("
        //     li t2, {run_start}
        //     jalr t2",
        //     run_start = const RUN_START,
        // );}
        // println!();

    }
    println!("Load payload ok!");
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);

    println!("Execute app ...");
    let arg0: u8 = b'A';
    // execute app
    unsafe { core::arch::asm!("
        li t0, {abi_num}
        slli t0, t0, 3
        la t1, {abi_table}
        add t1, t1, t0
        ld t1, (t1)
        jalr t1
        li t2, {run_start}
        jalr t2
        j .",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
        // abi_num = const SYS_HELLO,
        // abi_num = const SYS_PUTCHAR,
        abi_num = const SYS_TERMINATE,
        in("a0") arg0,
    )}
}