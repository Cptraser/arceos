#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0x22000000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let start = PLASH_START as *const u8;
    let apps_num = unsafe {
        let num = core::slice::from_raw_parts(start, 1);
        num[0]
    };

    println!("Load payload ...");
    let mut start_now = PLASH_START + 1;
    for i in 0..apps_num {
        let size: &[u8] = unsafe { core::slice::from_raw_parts(start_now as *const u8, 2) };
        let apps_size = (((size[0] as usize) << 8) + size[1] as usize) as usize; 
        let apps_start = (start_now + 2) as *const u8;
        let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
        println!("app{}_content: {:?}: ", i, code);
        // println!("appsize:{}", apps_size);
        start_now += 2 + apps_size;
    }

    println!("Load payload ok!");
}