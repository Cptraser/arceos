//! [ArceOS] Parse DTB (binary format of FDT), 
//! print physical memory range and all virtio_ Mmio range.

#![no_std]
#[macro_use]
mod dtb;

pub use dtb::DtbInfo;
pub use dtb::parse_dtb;

extern crate alloc;


