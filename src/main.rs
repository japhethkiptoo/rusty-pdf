use chrono::{DateTime, Utc};
use image::DynamicImage;

use numfmt::Formatter;
use printpdf::{
    Color, Image, ImageTransform, ImageXObject, IndirectFontRef, Line, Mm, PdfDocument,
    PdfLayerReference, Point, Px, Rgb,
};

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ffi::CString, fs::File, io::BufWriter, os::raw::c_char, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transaction {
    member_no: String,
    town: String,
    e_mail: String,
    allnames: String,
    post_address: String,
    gsm_no: String,
    descript: String,
    security_code: String,
    trans_id: i64,
    trans_date: DateTime<Utc>,
    account_no: String,
    taxamt: f64,
    trans_type: String,
    amount: f64,
    running_balance: f64,
    noofshares: Option<f64>,
    netamount: f64,
    mop: String,
    currency: String,
    p_amount: f64,
    w_amount: f64,
    i_amount: f64,
}

#[derive(Debug, Deserialize)]
struct Payload {
    pdf_name: String,
    transactions: Vec<Transaction>,
}

struct Summation {
    total_running_bal: f64,
    total_taxs: f64,
    total_deposits: f64,
    total_withdrawal: f64,
    total_interest: f64,
}

#[no_mangle]
pub extern "C" fn generate_pdf(payload: *const c_char) {
    let (w, h) = (210.0, 297.0); //A4
                                 // let x = 4; //totalpages;

    let c_str = unsafe {
        assert!(!payload.is_null());
        CString::from_raw(payload as *mut c_char)
    };

    let json_str = c_str.to_str().expect("Data failed to load");
    let data: Payload = serde_json::from_str(json_str).expect("Failed to load data");

    let pdf_name = &data.pdf_name;
    // println!("Trans {}", data.transactions.len());
    let transactions = &data.transactions;
    let transaction_one = &transactions[0];

    let data_len = transactions.len();

    let total_running_bal: f64 = transactions[data_len - 1].running_balance;
    let total_taxs: f64 = transactions.iter().map(|t| t.taxamt).sum();
    let total_deposits: f64 = transactions.iter().map(|t| t.p_amount).sum();
    let total_withdrawal: f64 = transactions.iter().map(|t| t.w_amount).sum();
    let total_interest: f64 = transactions.iter().map(|t| t.i_amount).sum();
    //totalpages

    let total_pages: i64 = if data_len <= 28 {
        1
    } else {
        let pages = ((data_len - 28) as f64 / 35.0).ceil() as i64 + 1;
        pages
    };

    println!("pages {}", total_pages);

    let margin_top = Mm(10.0);
    let margin_bottom = Mm(10.0);
    let margin_left = Mm(10.0);
    let margin_right = Mm(10.0);

    let usable_width = Mm(w) - margin_left - margin_right;
    let usable_height = Mm(h) - margin_top - margin_bottom;

    let (doc, page, layer) = PdfDocument::new("Full Statement", Mm(w), Mm(h), "layer 1");

    let default_font = doc
        .add_external_font(File::open("assets/fonts/Lato/Lato-Regular.ttf").unwrap())
        .unwrap();
    let bold_font = doc
        .add_external_font(File::open("assets/fonts/Lato/Lato-Bold.ttf").unwrap())
        .unwrap();

    for i in 0..total_pages {
        let current_layer: PdfLayerReference = if i == 0 {
            doc.get_page(page).get_layer(layer)
        } else {
            let (new_page, new_layer) = doc.add_page(Mm(w), Mm(h), format!("page {}", i + 1));

            doc.get_page(new_page).get_layer(new_layer)
        };

        page_footer(current_layer.clone(), usable_width, &default_font);

        let first_page_size = 28;
        let page_size = 35;

        if i == 0 {
            //top-header
            top_header(
                current_layer.clone(),
                &default_font,
                &bold_font,
                usable_width,
                usable_height,
                margin_top,
                margin_left,
                transaction_one,
            );
            let first_page_trans: Vec<Transaction> =
                transactions.iter().take(first_page_size).cloned().collect();
            gen_table(
                current_layer.clone(),
                h - 73.0,
                &default_font,
                &bold_font,
                first_page_trans,
                if total_pages == 1 { true } else { false },
                Summation {
                    total_running_bal,
                    total_deposits,
                    total_taxs,
                    total_withdrawal,
                    total_interest,
                },
            );
        }

        if i >= 1 {
            let logo = load_logo();
            logo.add_to_layer(
                current_layer.clone(),
                ImageTransform {
                    translate_x: Some(Mm(0.0) + Mm(7.0)),
                    translate_y: Some(usable_height + Mm(2.0)),
                    scale_x: Some(0.4),
                    scale_y: Some(0.4),
                    ..Default::default()
                },
            );

            current_layer.use_text(
                format!("page {}/{}", i + 1, total_pages),
                7.0,
                usable_width - Mm(0.00),
                usable_height + Mm(10.0),
                &default_font,
            );

            let trans_data: Vec<Transaction> =
                transactions.iter().skip(first_page_size).cloned().collect();
            let start_index = page_size * (i - 1) as usize;

            let trans: Vec<Transaction> = trans_data
                .iter()
                .skip(start_index)
                .take(page_size)
                .cloned()
                .collect();

            gen_table(
                current_layer.clone(),
                h - 25.0,
                &default_font,
                &bold_font,
                trans,
                if i + 1 == total_pages { true } else { false },
                Summation {
                    total_running_bal,
                    total_deposits,
                    total_taxs,
                    total_withdrawal,
                    total_interest,
                },
            );
        }
    }

    doc.with_conformance(printpdf::PdfConformance::X3_2003_PDF_1_4)
        .save(&mut BufWriter::new(
            File::create(format!("storage/{}-temp.pdf", pdf_name)).unwrap(),
        ))
        .unwrap();
}

fn top_header(
    current_layer: PdfLayerReference,
    default_font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
    usable_width: Mm,
    usable_height: Mm,
    margin_top: Mm,
    margin_left: Mm,
    user_details: &Transaction,
) {
    let logo = load_logo();
    logo.add_to_layer(
        current_layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(0.0) + Mm(7.0)),
            translate_y: Some(usable_height - margin_top - Mm(10.0)),
            scale_x: Some(0.7),
            scale_y: Some(0.7),
            ..Default::default()
        },
    );

    //customer details
    current_layer.begin_text_section();
    current_layer.set_text_cursor(Mm(0.0) + margin_left, usable_height - Mm(28.0));
    current_layer.set_font(&default_font, 8.0);
    current_layer.set_line_height(12.0);
    current_layer.write_text(format!("{}", user_details.allnames), &default_font);
    current_layer.add_line_break();
    current_layer.write_text(
        format!("P.O Box: {}", user_details.post_address),
        &default_font,
    );
    current_layer.add_line_break();
    current_layer.write_text(format!("Email: {}", user_details.e_mail), &default_font);
    current_layer.add_line_break();
    current_layer.write_text(format!("Tel. No. {}", user_details.gsm_no), &default_font);
    current_layer.end_text_section();

    //address
    current_layer.begin_text_section();
    current_layer.set_font(&default_font, 8.0);
    current_layer.set_text_cursor(usable_width - Mm(40.0), usable_height);
    current_layer.set_line_height(12.0);
    current_layer.write_text("P.O Box: 59485-00200", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("Nairobi, Kenya", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("Tel: 2823000", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("Fax: 2823344", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("CIC Plaza Mara Road,", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("Upper Hill.", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("cic.asset@cic.co.ke", &default_font);
    current_layer.add_line_break();
    current_layer.write_text("www.cic.co.ke", &default_font);

    current_layer.add_line_break();
    current_layer.add_line_break();
    current_layer.write_text(
        format!("Member No. {}", user_details.member_no),
        &default_font,
    );
    current_layer.add_line_break();
    current_layer.write_text(
        format!("Account No. {}", user_details.account_no),
        &default_font,
    );

    current_layer.add_line_break();
    current_layer.add_line_break();

    current_layer.write_text(
        format!(
            "{} | {}",
            user_details.descript,
            Utc::now().format("%Y-%m-%d")
        ),
        &bold_font,
    );

    current_layer.end_text_section();
}

fn page_footer(layer: PdfLayerReference, width: Mm, font: &IndirectFontRef) {
    layer.begin_text_section();
    layer.set_font(font, 7.0);
    layer.set_line_height(10.0);
    layer.set_text_cursor(width - Mm(150.0), Mm(8.0));
    layer.write_text("Bank account details: CO-OP Bank, CIC Dollar Fund Collection account, Acct No: 02120190806600 Branch: CO-OP House, Swift Code:", font);
    layer.add_line_break();
    layer.write_text("KCOOKENA. Remember to always quote your Member No. indicated at the top right hand corner.", font);
    layer.end_text_section();
}

fn load_logo() -> Image {
    let logo_file = load_image("assets/Logo.jpg");

    let image_file = ImageXObject {
        width: Px(logo_file.width() as usize),
        height: Px(logo_file.height() as usize),
        color_space: printpdf::ColorSpace::Rgb,
        bits_per_component: printpdf::ColorBits::Bit8,
        interpolate: true,
        image_data: logo_file.into_bytes(),
        image_filter: None,
        clipping_bbox: None,
    };

    return Image::from(image_file);
}

fn load_image(image_path: &str) -> DynamicImage {
    let img = image::open(&Path::new(image_path)).unwrap();
    img
}

fn gen_table(
    current_layer: PdfLayerReference,
    top_pos: f32,
    font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
    transactions: Vec<Transaction>,
    summations: bool,
    sums: Summation,
) {
    let cell_padding = 5.0;
    let column_widths = vec![15.0, 20.0, 20.0, 15.0, 15.0, 20.0, 23.0, 20.0];
    let table_start_x = 10.0;
    let table_start_y = top_pos;
    let row_height = 7.0;

    let red = Rgb::new(190.0 / 256.0, 0.0 / 256.0, 0.0 / 256.0, None);
    let gray = Rgb::new(230.0 / 256.0, 230.0 / 256.0, 230.0 / 256.0, None);
    let dark = Rgb::new(80.0 / 256.0, 80.0 / 256.0, 80.0 / 256.0, None);

    let mut data: Vec<Vec<Cow<str>>> = vec![vec![
        Cow::Borrowed("Trans No."),
        Cow::Borrowed("Trans Date"),
        Cow::Borrowed("Description"),
        Cow::Borrowed("Deposit"),
        Cow::Borrowed("Interest"),
        Cow::Borrowed("Withdrawal"),
        Cow::Borrowed("Withholding Tax"),
        Cow::Borrowed("Running Balance"),
    ]];

    let mut f = Formatter::new()
        .separator(',')
        .unwrap()
        .precision(numfmt::Precision::Decimals(2));

    let sum_data: Vec<Vec<Cow<str>>> = vec![vec![
        Cow::Borrowed("Summations"),
        Cow::Borrowed(""),
        Cow::Borrowed(""),
        Cow::Owned(format!("{}", f.fmt2(sums.total_deposits))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_interest))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_withdrawal.abs()))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_taxs.abs()))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_running_bal))),
    ]];

    for transaction in transactions.iter() {
        let trans = transaction;
        let trans_date = trans.trans_date.format("%Y-%m-%d").to_string();
        let trans_id = trans.trans_id.to_string();
        let trans_type = &trans.trans_type;

        let amount = trans.amount.clone();

        let deposit = if trans_type == "PURCHASE" {
            format!("{}", f.fmt2(amount))
        } else {
            "".to_string()
        };

        let withdrawal = if trans_type == "WITHDRAWAL" {
            format!("{}", f.fmt2(amount))
        } else {
            "".to_string()
        };

        let interest = if trans_type == "INTEREST" {
            format!("{}", f.fmt2(amount))
        } else {
            "".to_string()
        };

        let tax_amount = trans.taxamt.clone();
        let running_balance = trans.running_balance.clone();

        data.push(vec![
            Cow::Owned(trans_id),
            Cow::Owned(trans_date),
            Cow::Borrowed(""),
            Cow::Owned(deposit),
            Cow::Owned(interest),
            Cow::Owned(withdrawal),
            Cow::Owned(format!("{}", f.fmt2(tax_amount))),
            Cow::Owned(format!("{}", f.fmt2(running_balance))),
        ]);
    }

    for (row_index, row) in data.iter().enumerate() {
        for (col_index, cell_data) in row.iter().enumerate() {
            let x = table_start_x
                + column_widths[..col_index].iter().sum::<f32>()
                + cell_padding * col_index as f32;
            let y = table_start_y - row_index as f32 * row_height - cell_padding;

            let line_y = table_start_y - row_index as f32 * row_height;

            let points = vec![
                (Point::new(Mm(table_start_x), Mm(line_y)), false),
                (Point::new(Mm(table_start_x + 190.0), Mm(line_y)), false),
            ];
            let line = Line {
                points,
                is_closed: false,
            };
            current_layer.set_outline_thickness(0.7);

            if row_index == 0 {
                current_layer.set_outline_color(Color::Rgb(red.clone()));
                current_layer.add_line(line);
                current_layer.use_text(
                    cell_data.to_string(),
                    8.0,
                    Mm(x) + Mm(5.0),
                    Mm(y),
                    bold_font,
                );
            } else {
                current_layer.set_outline_thickness(0.6);
                current_layer.set_fill_color(Color::Rgb(dark.clone()));
                current_layer.use_text(cell_data.to_string(), 8.0, Mm(x) + Mm(5.0), Mm(y), font);
                current_layer.set_outline_color(Color::Rgb(gray.clone()));
                current_layer.add_line(line.clone());

                if row_index + 1 == data.len() && !summations {
                    let b_y = line_y - row_height;
                    let bottom_points = vec![
                        (Point::new(Mm(table_start_x), Mm(b_y)), false),
                        (Point::new(Mm(table_start_x + 190.0), Mm(b_y)), false),
                    ];
                    let bottom_line = Line {
                        points: bottom_points,
                        is_closed: false,
                    };
                    current_layer.add_line(bottom_line.clone());
                }
            }
        }
    }

    //summations
    let last_row_index = data.len();

    if summations {
        for (_row_index, row) in sum_data.iter().enumerate() {
            for (col_index, cell_data) in row.iter().enumerate() {
                let x = table_start_x
                    + column_widths[..col_index].iter().sum::<f32>()
                    + cell_padding * col_index as f32;
                let y = table_start_y - last_row_index as f32 * row_height - cell_padding;

                let line_y = table_start_y - last_row_index as f32 * row_height;
                let last_line_y = y - row_height + cell_padding;

                let points = vec![
                    (Point::new(Mm(table_start_x), Mm(line_y)), false),
                    (Point::new(Mm(table_start_x + 190.0), Mm(line_y)), false),
                ];
                let line = Line {
                    points,
                    is_closed: false,
                };

                let bottom_points = vec![
                    (Point::new(Mm(table_start_x), Mm(last_line_y)), false),
                    (
                        Point::new(Mm(table_start_x + 190.0), Mm(last_line_y)),
                        false,
                    ),
                ];
                let bottom_line = Line {
                    points: bottom_points,
                    is_closed: false,
                };

                current_layer.set_outline_thickness(0.6);
                current_layer.set_fill_color(Color::Rgb(red.clone()));
                current_layer.use_text(cell_data.to_string(), 8.0, Mm(x) + Mm(5.0), Mm(y), font);
                current_layer.set_outline_color(Color::Rgb(red.clone()));
                current_layer.add_line(line.clone());
                current_layer.add_line(bottom_line.clone());
            }
        }
    }
}

#[allow(dead_code)]
fn main() {}
