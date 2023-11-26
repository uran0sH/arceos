#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process::exit;

const PLASH_START: usize = 0x22000000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe {
        ABI_TABLE[num] = handle;
    }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
    unsafe { core::arch::asm!("la   a7, {}", sym ABI_TABLE) }
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
    unsafe { core::arch::asm!("la   a7, {}", sym ABI_TABLE) }
}

fn abi_terminate() {
    exit(0);
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // let apps_start = (PLASH_START + 512) as *const u8;
    let code = unsafe { core::slice::from_raw_parts(PLASH_START as *const u8, 1) };
    let app_num = code[0];
    println!("app_num={}", app_num);
    let mut app_size: [usize; 100] = [0; 100];
    let mut app_start: [usize; 100] = [0; 100];
    let mut offset: usize = 1;
    (0..app_num).for_each(|i| {
        let code = unsafe { core::slice::from_raw_parts((PLASH_START + offset) as *const u8, 2) };
        app_size[i as usize] = ((code[0] as usize) << 8) + code[1] as usize;
        app_start[i as usize] = (PLASH_START + 2 + offset);
        offset += 2 + app_size[i as usize];
        println!("app_size[{}]={}", i, app_size[i as usize]);
    });
    // switch aspace from kernel to app
    unsafe {
        init_app_page_table();
    }
    unsafe {
        switch_app_aspace();
    }
    const RUN_START: usize = 0x4010_0000;
    (0..app_num).for_each(|i| {
        // let code = unsafe { core::slice::from_raw_parts((PLASH_START + offset) as *const u8, 2) };
        // let apps_size = ((code[0] as usize) << 8) + code[1] as usize; // Dangerous!!! We need to get accurate size of apps.
        // println!("apps_size={}", apps_size);

        println!("Load payload ...");

        let code = unsafe {
            core::slice::from_raw_parts(app_start[i as usize] as *const u8, app_size[i as usize])
        };
        // println!("content: {:#x}", bytes_to_usize2(&code[..], apps_size));

        let run_code = unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_size[i as usize]) };
        run_code.copy_from_slice(code);

        // offset += 2 + app_size[i as usize];
        println!("Load payload ok!");

        register_abi(SYS_HELLO, abi_hello as usize);
        register_abi(SYS_PUTCHAR, abi_putchar as usize);
        register_abi(SYS_TERMINATE, abi_terminate as usize);

        println!("Execute app ...");
        // let arg0: u8 = b'A';

        // execute app
        unsafe {
            core::arch::asm!("
            la      a7, {abi_table}
            li      t2, {run_start}
            jalr    t2",
                clobber_abi("C"),
                run_start = const RUN_START,
                abi_table = sym ABI_TABLE,
            )
        }
        println!("Execute app ok!");
    })
}

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
}

#[inline]
fn bytes_to_usize2(bytes: &[u8], size: usize) -> usize {
    if size >= 8 {
        return bytes_to_usize(bytes);
    }
    let mut result: usize = 0;
    (0..size).for_each(|i| {
        result = result << 8 | (bytes[i] as usize);
    });
    return result;
}

//
// App aspace
//

#[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [u64; 512] = [0; 512];

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

unsafe fn switch_app_aspace() {
    use riscv::register::satp;
    let page_table_root = APP_PT_SV39.as_ptr() as usize - axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}
