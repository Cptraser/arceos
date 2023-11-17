#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process::exit;

#[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [u64; 512] = [0; 512];

/// Head: 0 u8: app number, then two u8: app size, the size of u8...

const PLASH_START: usize = 0x22000000;
// const RUN_START: usize = 0xffff_ffc0_8010_0000;
const RUN_START: usize = 0x4010_0000;

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

    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);

    let mut start_now = PLASH_START + 1;

    for i in 0..apps_num {

        unsafe { init_app_page_table(); }
        unsafe { switch_app_aspace(i as usize); }

        println!("Load payload ...");
        let size: &[u8] = unsafe { core::slice::from_raw_parts(start_now as *const u8, 2) };

        let apps_size = (((size[0] as usize) << 8) + size[1] as usize) as usize; 
        let apps_start = (start_now + 2) as *const u8;
        let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
        println!("app{}_size: {}; address: [{:?}]", i, code.len(), code.as_ptr());
        start_now += 2 + apps_size;

        // app running aspace
        // SBI(0x80000000) -> App <- Kernel(0x80200000)
        // 0xffff_ffc0_0000_0000
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, apps_size)
        };
        run_code.copy_from_slice(code);
        println!("run code size {}; address [{:?}]", run_code.len(), run_code.as_ptr());
        println!("Load payload ok!");

        println!("Execute app{} ...", i);
        // execute app
        unsafe { core::arch::asm!("
            la a0, {abi_table}
            li t2, {run_start}
            jalr t2",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
        );}
        println!();

    }

    println!("Execute app done, total {} apps", apps_num);

}

unsafe fn init_app_page_table() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0x102] = (0x80000 << 10) | 0xef;
    // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0] = (0x00000 << 10) | 0xef;
    // For App aspace!
    // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[1] = (0x80000 << 10) | 0xef;
}
unsafe fn switch_app_aspace(asid: usize) {
    use riscv::register::satp;
    let page_table_root = APP_PT_SV39.as_ptr() as usize -
    axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, asid, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}