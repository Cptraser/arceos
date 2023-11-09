//! HashMap Std

#[cfg(feature = "alloc")]
mod hashmap;

#[cfg(feature = "alloc")]
pub use self::hashmap::HashMap;