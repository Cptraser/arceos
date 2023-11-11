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
        if let Some(size) = fdt.memory().regions().next().unwrap().size {
            let soc = fdt.find_node("/soc");
            if let Some(soc) = soc {
                let mut regions = Vec::new();
                for child in soc.children() {
                    if child.name.contains("mmio") {
                        if let Some(reg_prop) = child.property("reg") {
                            // axlog::info!("childname:{}", child.name);
                            let reg_values: &[u8] = reg_prop.value;
                            // for x in reg_values {
                            //     axlog::info!("{:#x}", x);
                            // }
                            let address: usize = (reg_values[4] as usize) << 24 |
                                                 (reg_values[5] as usize) << 16 |
                                                 (reg_values[6] as usize) << 8 | 
                                                 (reg_values[7] as usize);
                            let size: usize = (reg_values[12] as usize) << 24 |
                                              (reg_values[13] as usize) << 16 |
                                              (reg_values[14] as usize) << 8 | 
                                              (reg_values[15] as usize);
                            regions.push( (address, size) );
                        } else {
                            return Err(Error);
                        }
                    }
                }
                let res = Ok(DtbInfo {
                    memory_addr: addr,
                    memory_size: size,
                    mmio_regions: regions,
                });
                return res;
            }
            return Err(Error);
        }
        Err(Error)
    }
    
}


