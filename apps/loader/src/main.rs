#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0xffff_ffc0_8010_0000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // let apps_start = (PLASH_START + 512) as *const u8;
    let code = unsafe { core::slice::from_raw_parts(PLASH_START as *const u8, 1) };
    let app_num = code[0];
    println!("app_num={}", app_num);
    let mut offset: usize = 1;
    (0..app_num).for_each(|_i| {
        let code = unsafe { core::slice::from_raw_parts((PLASH_START + offset) as *const u8, 2) };
        let apps_size = ((code[0] as usize) << 8) + code[1] as usize; // Dangerous!!! We need to get accurate size of apps.
        println!("apps_size={}",apps_size);

        println!("Load payload ...");

        let code = unsafe { core::slice::from_raw_parts((PLASH_START + 2 + offset) as *const u8, apps_size) };
        println!("content: {:#x}", bytes_to_usize2(&code[..], apps_size));

        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, apps_size)
        };
        run_code.copy_from_slice(code);

        offset += 2 + apps_size;
        println!("Load payload ok!");

        println!("Execute app ...");
        // execute app
        unsafe { core::arch::asm!("
            li t2, {run_start}
            jalr t2",
            run_start = const RUN_START,
        );}
    })
    
}

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
}

#[inline]
fn bytes_to_usize2(bytes: &[u8], size: usize) -> usize {
    if size >= 8 {
        return bytes_to_usize(bytes)
    }
    let mut result: usize = 0;
    (0..size).for_each(|i| {
        result = result << 8 | (bytes[i] as usize);
    });
    return result
}