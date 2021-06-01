mod engine;

fn main() {
    let mut sheet = engine::new_sheet();
    sheet.set(0, 0, "hi".to_string());
    sheet.set(1, 1, "hello".to_string());
    show_spreadsheet(&sheet);
}

fn show_spreadsheet(sheet: &engine::Spreadsheet) {
    let (w, h) = sheet.get_max_dims();
    for y in 0..h+1 {
        for x in 0..w+1 {
            let s = sheet.show_cell(x, y);
            print!("{}", if s == "" { "_" } else { &s });
            if x < w { print!(",\t"); }
        }
        println!();
    }
}
