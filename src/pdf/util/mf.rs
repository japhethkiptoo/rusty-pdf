use std::borrow::Cow;

use numfmt::{Formatter, Scales};
use printpdf::{Color, IndirectFontRef, Line, Mm, PdfLayerReference, Point, Rgb};

use super::{Summation, Transaction};

pub fn gen_table_mmf(
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
    let row_height = 8.0;

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
        .scales(Scales::none())
        .precision(numfmt::Precision::Decimals(2));

    let sum_data: Vec<Vec<Cow<str>>> = vec![vec![
        Cow::Borrowed("Summations"),
        Cow::Borrowed(""),
        Cow::Borrowed(""),
        Cow::Owned(format!("{}", f.fmt2(sums.total_deposits.round()))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_interest.round()))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_withdrawal.abs().round()))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_taxs.abs().round()))),
        Cow::Owned(format!("{}", f.fmt2(sums.total_running_bal.round()))),
    ]];

    for transaction in transactions.iter() {
        let trans = transaction;
        let trans_date = trans.trans_date.format("%Y-%m-%d").to_string();
        let trans_id = trans.trans_id.to_string();
        let trans_type = &trans.trans_type;

        let amount = trans.amount.clone().round();

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
            Cow::Owned(format!("{}", f.fmt2(tax_amount.round()))),
            Cow::Owned(format!("{}", f.fmt2(running_balance.round()))),
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
                    7.5,
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
