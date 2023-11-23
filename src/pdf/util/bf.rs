use printpdf::{Color, IndirectFontRef, Line, Mm, PdfLayerReference, Point, Rgb};
use std::iter::Zip;

pub fn gen_table(
    layer: PdfLayerReference,
    top_pos: f32,
    font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
) {
    let table_width = 190.0;
    let cell_padding = 5.0;
    let table_x = 10.0;
    let table_y = top_pos;
    let row_height = 8.0;
    let header_column_widths = vec![47.0, 47.0, 47.0, 47.0];
    let column_widths = vec![
        10.0, 12.0, 15.0, 12.0, 12.0, 12.0, 12.0, 15.0, 15.0, 15.0, 10.0,
    ];
    let columns = vec![
        "Trans Id",
        "Trans Date",
        "Description",
        "Units",
        "Price",
        "Cost",
        "Units",
        "Price",
        "Cost",
        "Units",
        "Nav",
    ];

    let red = Rgb::new(190.0 / 256.0, 0.0 / 256.0, 0.0 / 256.0, None);
    let gold = Rgb::new(255.0 / 256.0, 215.0 / 256.0, 0.0 / 256.0, None);
    let dark = Rgb::new(80.0 / 256.0, 80.0 / 256.0, 80.0 / 256.0, None);
    let gray = Rgb::new(230.0 / 256.0, 230.0 / 256.0, 230.0 / 256.0, None);

    let colors = table_colors();

    for (row_index, row) in (0..30).enumerate() {
        if row_index == 0 {
            for (col_index, header) in ["", "Purchases", "Sales", "Balance"].iter().enumerate() {
                let text_len = header.len();
                let col_width = header_column_widths[..col_index].iter().sum::<f32>();
                let hx = table_x
                    + (col_width + (text_len * col_index) as f32 / 2.0)
                    + cell_padding * col_index as f32;
                let hy = table_y - row_index as f32 * row_height - cell_padding;

                let points = vec![
                    (
                        Point::new(Mm(hx), Mm(hy - row_height + cell_padding)),
                        false,
                    ),
                    (
                        Point::new(Mm(table_x + 190.0), Mm(hy - row_height + cell_padding)),
                        false,
                    ),
                ];
                let line = Line {
                    points,
                    is_closed: false,
                };

                let color = if col_index == 1 {
                    Rgb::new(244.0 / 256.0, 164.0 / 256.0, 96.0 / 256.0, None)
                } else if col_index == 2 {
                    Rgb::new(87.0 / 256.0, 75.0 / 256.0, 144.0 / 256.0, None)
                } else {
                    Rgb::new(80.0 / 256.0, 80.0 / 256.0, 80.0 / 256.0, None)
                };

                layer.set_outline_thickness(0.8);
                layer.set_outline_color(Color::Rgb(red.clone()));
                layer.add_line(line);
                layer.set_fill_color(printpdf::Color::Rgb(color.clone()));
                layer.use_text(header.to_string(), 8.0, Mm(hx), Mm(hy), bold_font);
            }

            for (col_index, (title, color)) in columns.iter().zip(colors.iter()).enumerate() {
                let col_width = column_widths[..col_index].iter().sum::<f32>();
                let hx = table_x + col_width + cell_padding * col_index as f32;
                let hy = table_y - row_height - row_index as f32 * row_height - cell_padding;
                layer.set_fill_color(Color::Rgb(color.clone()));
                layer.use_text(title.to_string(), 7.5, Mm(hx), Mm(hy), bold_font);

                let points = vec![
                    (
                        Point::new(Mm(hx), Mm(hy - row_height + cell_padding)),
                        false,
                    ),
                    (
                        Point::new(Mm(table_x + 190.0), Mm(hy - row_height + cell_padding)),
                        false,
                    ),
                ];
                let line = Line {
                    points,
                    is_closed: false,
                };
                layer.set_outline_thickness(0.7);
                layer.set_outline_color(Color::Rgb(gray.clone()));
                layer.add_line(line);
            }
        }
        if row_index > 0 {
            for (col_index, (cell_data, color)) in (0..=10).zip(colors.iter()).enumerate() {
                let start_y = table_y - row_height;
                let col_width = column_widths[..col_index].iter().sum::<f32>();
                let x = table_x + col_width + cell_padding * col_index as f32;
                let y = start_y - row_index as f32 * row_height - cell_padding;

                let points = vec![
                    (Point::new(Mm(x), Mm(y - row_height + cell_padding)), false),
                    (
                        Point::new(Mm(table_x + 190.0), Mm(y - row_height + cell_padding)),
                        false,
                    ),
                ];
                let line = Line {
                    points,
                    is_closed: false,
                };
                layer.set_outline_thickness(0.7);
                layer.set_outline_color(Color::Rgb(gray.clone()));
                layer.set_fill_color(Color::Rgb(color.clone()));
                layer.use_text("0.0", 7.0, Mm(x), Mm(y), font);
                layer.add_line(line);
            }
        }
    }
}

fn table_colors() -> Vec<Rgb> {
    let red = Rgb::new(190.0 / 256.0, 0.0 / 256.0, 0.0 / 256.0, None);
    let brown = Rgb::new(244.0 / 256.0, 164.0 / 256.0, 96.0 / 256.0, None);
    let gray = Rgb::new(230.0 / 256.0, 230.0 / 256.0, 230.0 / 256.0, None);
    let dark = Rgb::new(80.0 / 256.0, 80.0 / 256.0, 80.0 / 256.0, None);
    let blue = Rgb::new(87.0 / 256.0, 75.0 / 256.0, 144.0 / 256.0, None);

    vec![
        dark.clone(),
        dark.clone(),
        dark.clone(),
        brown.clone(),
        brown.clone(),
        brown.clone(),
        blue.clone(),
        blue.clone(),
        blue.clone(),
        dark.clone(),
        dark.clone(),
    ]
}
