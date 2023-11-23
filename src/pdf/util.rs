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

pub fn create_pdf(data: Vec<Transaction>, pdf_name: String, mmf: bool) {
    let (w, h) = (210.0, 297.0);
    let total_pages = 2;

    let pdf_name = "test";

    let margin_top = Mm(10.0);
    let margin_bottom = Mm(10.0);
    let margin_left = Mm(10.0);
    let margin_right = Mm(10.0);

    let usable_width = Mm(w) - margin_left - margin_right;
    let usable_height = Mm(h) - margin_top - margin_bottom;

    let (doc, page, layer) = PdfDocument::new("Full Statement", Mm(w), Mm(h), "layer 1");

    let default_font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold_font = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

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
                &Transaction {
                    member_no: "0000".to_owned(),
                    town: "Nairobi".to_owned(),
                    e_mail: "Jk@gmail.com".to_owned(),
                    allnames: "Kaamil Too".to_owned(),
                    post_address: "".to_owned(),
                    gsm_no: "254724765149".to_owned(),
                    descript: "Money Market".to_owned(),
                    security_code: "002".to_owned(),
                    trans_id: 1234,
                    trans_date: Utc::now(),
                    account_no: "account_no".to_owned(),
                    taxamt: 0.0,
                    trans_type: "Purchase".to_owned(),
                    amount: 0.0,
                    running_balance: 0.0,
                    shares: Some(0.0),
                    price: Some(0.0),
                    netamount: 0.0,
                    mop: "MPESA".to_owned(),
                    currency: "KEs".to_owned(),
                    p_amount: 0.0,
                    w_amount: 0.0,
                    i_amount: 0.0,
                },
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
                gen_table_mmf(
                    current_layer,
                    h - 70.0,
                    &default_font,
                    &bold_font,
                    data.clone(),
                    if total_pages == 1 { true } else { false },
                    Summation {
                        total_running_bal: 0.0,
                        total_taxs: 0.0,
                        total_deposits: 0.0,
                        total_withdrawal: 0.0,
                        total_interest: 0.0,
                    },
                );
            } else {
                gen_table_mmf(
                    current_layer,
                    h - 22.0,
                    &default_font,
                    &bold_font,
                    data.clone(),
                    if total_pages == 1 { true } else { false },
                    Summation {
                        total_running_bal: 0.0,
                        total_taxs: 0.0,
                        total_deposits: 0.0,
                        total_withdrawal: 0.0,
                        total_interest: 0.0,
                    },
                );
            }
        } else {
            if p == 0 {
                gen_table(current_layer, h - 70.0, &default_font, &bold_font);
            } else {
                gen_table(current_layer, h - 22.0, &default_font, &bold_font);
            }
        }
    }
    let mut writer = BufWriter::new(File::create(format!("{}-temp.pdf", pdf_name)).unwrap());

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
    page: usize,
    total_pages: usize,
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
        usable_height + Mm(10.0),
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
        layer.use_text(line.to_string(), 7.0, center_x, y, font)
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
