#![no_std]
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::result::Result::Ok;
use fdt::Fdt;

type Result<T> = core::result::Result<T, String>;

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo> {
    let fdt = unsafe {
        match Fdt::from_ptr(dtb_pa as *const u8) {
            Ok(fdt) => fdt,
            Err(e) => return Err(format!("Bad dtb {:?}", e)),
        }
    };
    let dtb_info = {
        let mut dtb_info = DtbInfo {
            memory_addr: 0,
            memory_size: 0,
            mmio_regions: Vec::new(),
        };
        let memory_node = fdt.memory();
        let mut regions = memory_node.regions();
        let first_region = match regions.next() {
            Some(region) => region,
            None => return Err(format!("Bad dtb: memory region is None")),
        };
        dtb_info.memory_addr = first_region.starting_address as usize;
        dtb_info.memory_size = match first_region.size {
            Some(size) => size,
            None => return Err(format!("Bad dtb: memory size is None")),
        };
        for node in fdt.find_all_nodes("/soc/virtio_mmio") {
            let regions = match node.reg() {
                Some(regions) => regions,
                None => return Err(format!("Bad dtb: memory region is None")),
            };
            for region in regions {
                dtb_info
                    .mmio_regions
                    .push((region.starting_address as usize, region.size.unwrap()));
            }
        }
        dtb_info
    };
    Ok(dtb_info)
}
