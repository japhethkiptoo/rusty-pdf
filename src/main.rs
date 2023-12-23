mod pdf;

use crate::pdf::util::*;
use serde::Deserialize;
use std::{ffi::CString, os::raw::c_char};

#[derive(Debug, Deserialize)]
struct Payload {
    pdf_name: String,
    transactions: Vec<Transaction>,
}

#[no_mangle]
pub extern "C" fn generate_statement(payload: *const c_char, mmf: bool) {
    let c_str = unsafe {
        assert!(!payload.is_null());
        CString::from_raw(payload as *mut c_char)
    };

    let json_str = c_str.to_str().expect("Data failed to load");
    let data: Payload = serde_json::from_str(json_str).expect("Failed to load data");
    println!("{:?}", data.transactions[0]);
    create_pdf(data.transactions, data.pdf_name, mmf);
}

#[allow(dead_code)]
fn main() {}
