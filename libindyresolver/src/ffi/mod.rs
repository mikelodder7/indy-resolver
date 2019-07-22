#[macro_use]
pub mod macros;
pub mod pool_context;

use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct ByteArray {
    length: usize,
    data: *const u8
}

impl ByteArray {
    pub fn to_vec(&self) -> Vec<u8> {
        if self.data.is_null() || self.length == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.length).to_vec() }
        }
    }
}

#[repr(C)]
pub struct NymHandle {
    did: *const c_char,      //Fully Qualified DID
    ver_key: ByteArray,     //Can be null for reads
    private_key: ByteArray  //Can be null for reads
}

pub type Handle = i32;
pub type OutString = *const c_char;

define_bytebuffer_destructor!(indy_res_free_bytebuffer);
define_string_destructor!(indy_res_free_string);
