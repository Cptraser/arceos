use core::fmt::Error;
use alloc::vec::Vec;

#[allow(unused_imports)]
use axlog;

// CONSTANTS
const DTB_MAGIC: u32 = 0xD00DFEED;
const DTB_VERSION: u32 = 17;

#[allow(dead_code)]
struct DtbHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

fn check_header(header: &DtbHeader) -> bool {
    u32::from_be(header.magic) == DTB_MAGIC && u32::from_be(header.version) == DTB_VERSION
}

pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo, Error> {
    unsafe {
        let fdt = fdt::Fdt::from_ptr(dtb_pa as *const u8).unwrap();
        let address = dtb_pa as *const u8;
        let header = &*(address as *const DtbHeader);
        if !check_header(header) {
            return Err(Error);
        }
        let addr = fdt.memory().regions().next().unwrap().starting_address as usize;
        let size = fdt.memory().regions().next().unwrap().size.unwrap();
        let mut regions = Vec::new();
        for mmio_node in fdt.find_all_nodes("/soc/virtio_mmio") {
            regions.push( (mmio_node.reg().unwrap().next().unwrap().starting_address as usize, 
                           mmio_node.reg().unwrap().next().unwrap().size.unwrap()) );
        }
        let res = Ok(DtbInfo {
            memory_addr: addr,
            memory_size: size,
            mmio_regions: regions,
        });
        res
    }
    
}