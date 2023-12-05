mod pdf;

use crate::pdf::util::*;
use num_format::*;
use numfmt::{Formatter, Scales};
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

fn has_leading_zeros_after_decimal(number: f64) -> bool {
    let num_str = number.to_string();

    if let Some(decimal_position) = num_str.find('.') {
        let decimal_part = &num_str[decimal_position + 1..];

        // Check if there are leading zeros after the decimal point
        if decimal_part.chars().take_while(|&c| c == '0').count() >= 2 {
            return true;
        }
    }

    false
}

#[allow(dead_code)]
fn main() {
    let number = 0.345;

    if has_leading_zeros_after_decimal(number) && number < 1.0 {
        let rounded_number = (number * 10000.0 as f64).round() / 10000.0;
        let final_num = format!("{:.4}", rounded_number);
        println!("{}", final_num);
        return;
    }
    let rounded_number = (number * 100.0 as f64).round() / 100.0;
    let mut f: Formatter;
    f = "[.2n/,]".parse().unwrap();

    let formatted_number = format!("{}", f.fmt2(rounded_number));

    println!("Original number: {}", number);
    println!("Rounded number: {}", rounded_number);
    println!("Formatted number: {}", formatted_number);
}
