macro_rules! byte_array_to_rust {
    ($ptr:ident, $len:expr) => {
        if $ptr.is_null() || $len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts($ptr, $len as usize).to_vec() }
        }
    }
}

macro_rules! buffer_to_vec {
    ($buffer:ident) => {
        if $buffer.data.is_null() || $buffer.length == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts($buffer.data, $buffer.length).to_vec() }
        }
    };
}

macro_rules! assign_out_string {
    ($input:expr, $string:expr) => {
        *$input = ffi_support::rust_string_to_c($string);
    }
}
