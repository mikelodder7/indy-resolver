use super::*;
use ffi_support::{FfiStr, ExternError};

#[no_mangle]
pub extern fn indy_res_context_create(pool_handle: Handle,
                                      submitter_did: FfiStr<'_>,
                                      submitter_did_private_key: &ByteArray,
                                      context_json: FfiStr<'_>,
                                      context_did: &mut OutString,
                                      err: &mut ExternError) -> i32 {
    let submitter_did = submitter_did.as_str();
    let submitter_did_buffer = submitter_did_private_key.to_vec();
    let context_json = context_json.as_str();
    *err = ExternError::success();

    //TODO: do some work

    let s = r#"{"id":"did:sov1qazxsw2wsdxcde34erfgvbg"}"#;

    assign_out_string!(context_did, s);
    if err.get_code().code() > 0 {
        0
    } else {
        1
    }
}


#[cfg(test)]
mod tests {
    use ffi_support::{FfiStr, ExternError};
    use std::os::raw::c_char;
    use crate::ffi::pool_context::indy_res_context_create;
    use std::ffi::CString;

    #[test]
    fn vanilla_test() {
        let pool_handle = 0;
        let did = std::ffi::CString::new("did:sov:aksjdhgkasjhtkhrg").unwrap();
        let submitter_did = unsafe { FfiStr::from_raw(did.as_ptr()) };
        let submitter_private_key = vec![1u8; 32];
        let context = std::ffi::CString::new("test").unwrap();
        let context_json = unsafe { FfiStr::from_raw(context.as_ptr()) };
        let mut context_did: *const c_char = std::ptr::null();
        let mut err = ExternError::success();

        let res = indy_res_context_create(pool_handle,
                                                       submitter_did,
                                                       submitter_private_key.as_slice().as_ptr() as *const u8,
                                                       submitter_private_key.len(),
                                                       context_json,
                                                       &mut context_did,
                                                       &mut err);

        let s = unsafe { std::ffi::CStr::from_ptr(context_did) };
        let ctx_did = s.to_str().unwrap();
        dbg!(ctx_did);
    }
}
