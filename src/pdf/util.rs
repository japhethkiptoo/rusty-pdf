mod bf;
mod mf;

use ::image::{open, DynamicImage};
use chrono::{DateTime, Utc};
use printpdf::{
    BuiltinFont, Image, ImageTransform, ImageXObject, IndirectFontRef, Mm, PdfConformance,
    PdfDocument, PdfLayerReference, Px,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufWriter, path::Path};
use textwrap::wrap;

use bf::gen_table;
use mf::gen_table_mmf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
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
    running_shares: f64,
    shares: Option<f64>,
    price: Option<f64>,
    netamount: f64,
    mop: String,
    currency: String,
    p_amount: f64,
    w_amount: f64,
    i_amount: f64,
}

pub struct Summation {
    total_running_bal: f64,
    total_taxs: f64,
    total_deposits: f64,
    total_withdrawal: f64,
    total_interest: f64,
}

pub struct BFSummation {
    total_purchase_units: f64,
    total_purchase_costs: f64,
    total_sale_units: f64,
    total_sale_costs: f64,
    total_balance_units: f64,
    latest_nav: f64,
    total_running_bal: f64,
    closing_date: DateTime<Utc>,
}

pub fn create_pdf(data: Vec<Transaction>, pdf_name: String, mmf: bool) {
    let (w, h) = (210.0, 297.0);
    let data_len = data.len();

    let first_page_size = 26;
    let per_page = 31;

    let total_pages: i64 = if data_len <= first_page_size {
        1
    } else {
        let pages = ((data_len - first_page_size) as f64 / per_page as f64).ceil() as i64 + 1;
        pages
    };

    let margin_top = Mm(10.0);
    let margin_bottom = Mm(10.0);
    let margin_left = Mm(10.0);
    let margin_right = Mm(10.0);

    let usable_width = Mm(w) - margin_left - margin_right;
    let usable_height = Mm(h) - margin_top - margin_bottom;

    let (doc, page, layer) = PdfDocument::new("Full Statement", Mm(w), Mm(h), "layer 1");

    let default_font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold_font = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let user_details = &data[0];

    let total_running_bal: f64 = data[data_len - 1].running_balance;
    let total_taxs: f64 = data.iter().map(|t| t.taxamt).sum();
    let total_deposits: f64 = data.iter().map(|t| t.p_amount).sum();
    let total_withdrawal: f64 = data.iter().map(|t| t.w_amount).sum();
    let total_interest: f64 = data.iter().map(|t| t.i_amount).sum();

    let total_purchase_units: f64 = data
        .iter()
        .map(|t| match t.trans_type.as_str() {
            "PURCHASE" => t.shares.unwrap(),
            _ => 0.0,
        })
        .sum();
    let total_purchase_costs: f64 = data
        .iter()
        .map(|t| match t.trans_type.as_str() {
            "PURCHASE" => t.amount,
            _ => 0.0,
        })
        .sum();
    let total_sale_units: f64 = data
        .iter()
        .map(|t| match t.trans_type.as_str() {
            "WITHDRAWAL" => t.shares.unwrap(),
            _ => 0.0,
        })
        .sum();

    let total_sale_costs: f64 = data
        .iter()
        .map(|t| match t.trans_type.as_str() {
            "WITHDRAWAL" => t.amount,
            _ => 0.0,
        })
        .sum();
    let total_balance_units = data[data_len - 1].running_shares;
    let latest_nav: f64 = data[data_len - 1].price.unwrap();
    let closing_date = data[data_len - 1].trans_date;

    for p in 0..total_pages {
        let current_layer: PdfLayerReference = if p == 0 {
            doc.get_page(page).get_layer(layer)
        } else {
            let (new_page, new_layer) = doc.add_page(Mm(w), Mm(h), format!("page {}", p + 1));

            doc.get_page(new_page).get_layer(new_layer)
        };

        if p == 0 {
            main_header(
                current_layer.clone(),
                &default_font,
                &bold_font,
                usable_width,
                usable_height,
                margin_top,
                margin_left,
                &user_details,
            )
        }

        page_footer(current_layer.clone(), usable_width, &default_font);

        if p > 0 {
            page_header(
                current_layer.clone(),
                usable_height,
                usable_width,
                p + 1,
                total_pages,
                &default_font,
            );
        }

        if mmf {
            if p == 0 {
                let first_page_trans: Vec<Transaction> =
                    data.iter().skip(0).take(first_page_size).cloned().collect();
                gen_table_mmf(
                    current_layer,
                    h - 66.0,
                    &default_font,
                    &bold_font,
                    first_page_trans,
                    if total_pages == 1 { true } else { false },
                    Summation {
                        total_running_bal,
                        total_taxs,
                        total_deposits,
                        total_withdrawal,
                        total_interest,
                    },
                );
            } else {
                let trans_data: Vec<Transaction> =
                    data.iter().skip(first_page_size).cloned().collect();
                let start_index = per_page * (p - 1) as usize;

                let trans: Vec<Transaction> = trans_data
                    .iter()
                    .skip(start_index)
                    .take(per_page)
                    .cloned()
                    .collect();
                gen_table_mmf(
                    current_layer,
                    h - 22.0,
                    &default_font,
                    &bold_font,
                    trans,
                    if p + 1 == total_pages { true } else { false },
                    Summation {
                        total_running_bal,
                        total_taxs,
                        total_deposits,
                        total_withdrawal,
                        total_interest,
                    },
                );
            }
        } else {
            if p == 0 {
                let first_page_trans: Vec<Transaction> =
                    data.iter().take(first_page_size).cloned().collect();
                gen_table(
                    current_layer,
                    h - 60.0,
                    &default_font,
                    &bold_font,
                    first_page_trans,
                    if total_pages == 1 { true } else { false },
                    BFSummation {
                        total_purchase_units,
                        total_purchase_costs,
                        total_sale_units,
                        total_sale_costs,
                        total_balance_units,
                        latest_nav,
                        total_running_bal,
                        closing_date,
                    },
                );
            } else {
                let trans_data: Vec<Transaction> =
                    data.iter().skip(first_page_size).cloned().collect();
                let start_index = per_page * (p - 1) as usize;

                let trans: Vec<Transaction> = trans_data
                    .iter()
                    .skip(start_index)
                    .take(per_page)
                    .cloned()
                    .collect();
                gen_table(
                    current_layer,
                    h - 22.0,
                    &default_font,
                    &bold_font,
                    trans,
                    if p + 1 == total_pages { true } else { false },
                    BFSummation {
                        total_purchase_units,
                        total_purchase_costs,
                        total_sale_units,
                        total_sale_costs,
                        total_balance_units,
                        latest_nav,
                        total_running_bal,
                        closing_date,
                    },
                );
            }
        }
    }
    let mut writer =
        BufWriter::new(File::create(format!("storage/{}-temp.pdf", pdf_name)).unwrap());

    doc.with_conformance(PdfConformance::X3_2003_PDF_1_4)
        .save(&mut writer)
        .unwrap();
}

fn main_header(
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
            translate_y: Some(usable_height - margin_top - Mm(5.0)),
            scale_x: Some(0.7),
            scale_y: Some(0.7),
            ..Default::default()
        },
    );

    //customer details
    current_layer.begin_text_section();
    current_layer.set_text_cursor(Mm(0.0) + margin_left, usable_height - Mm(23.0));
    current_layer.set_font(&default_font, 8.5);
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
    current_layer.set_font(&default_font, 8.5);
    current_layer.set_text_cursor(
        usable_width - Mm(40.0),
        usable_height + margin_top - Mm(5.0),
    );
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

    current_layer.end_text_section();

    current_layer.use_text(
        format!(
            "{} | {}",
            user_details.descript,
            Utc::now().format("%d-%m-%Y")
        ),
        9.0,
        usable_width - Mm(40.0),
        usable_height - Mm(45.0),
        bold_font,
    );
}

fn page_header(
    layer: PdfLayerReference,
    usable_height: Mm,
    usable_width: Mm,
    page: i64,
    total_pages: i64,
    font: &IndirectFontRef,
) {
    let logo = load_logo();
    logo.add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(0.0) + Mm(7.0)),
            translate_y: Some(usable_height + Mm(2.0)),
            scale_x: Some(0.4),
            scale_y: Some(0.4),
            ..Default::default()
        },
    );

    layer.use_text(
        format!("page {}/{}", page, total_pages),
        7.0,
        usable_width - Mm(0.00),
        usable_height + Mm(7.0),
        font,
    );
}

fn page_footer(layer: PdfLayerReference, usable_width: Mm, font: &IndirectFontRef) {
    let paragraph = "Bank account details: CO-OP Bank, CIC Dollar Fund Collection account, Acct No: 02120190806600 Branch: CO-OP House, Swift Code:KCOOKENA. Remember to always quote your Member No. indicated at the top right hand corner.";

    let wrapped_text = wrap(paragraph, 100);
    for (i, line) in wrapped_text.iter().enumerate() {
        let text_width = line.len();
        let center_x = (usable_width - Mm(text_width as f32)) / 2.0;
        let y = Mm(9.0) - Mm(i as f32 * 3.0);
        layer.use_text(line.to_string(), 6.5, center_x, y, font)
    }
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
    let img = open(&Path::new(image_path)).unwrap();
    img
}
