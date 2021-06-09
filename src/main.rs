mod engine;

use std::io::{self, Write};

fn main() {
    let mut sheet = engine::Spreadsheet::new();
    let mut line = String::new();
    loop {
        if !line.is_empty() {
            match run_line(&line, &mut sheet) {
                Ok(_) => (),
                Err(err) => println!("{}", err),
            }
        }

        show_spreadsheet(&sheet);

        print!("> ");
        io::stdout().flush().unwrap();
        line.clear();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!("bye bye!");
                break;
            }
            Ok(_) => {
                // Remove the newline.
                line.pop();
            }
            Err(error) => println!("error while reading: {}", error),
        }
    }
}

fn run_line(line: &String, sheet: &mut engine::Spreadsheet) -> Result<(), engine::Error> {
    let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
    match cmd {
        "help" => help(),
        "set" => match rest.splitn(3, ' ').collect::<Vec<_>>().as_slice() {
            [col, row, val] => {
                let col = col.parse::<usize>()?;
                let row = row.parse::<usize>()?;
                sheet.set(col, row, val.to_string());
            }
            _ => println!("expected \"set col row val\", got: \"{}\"", rest),
        },
        _ => println!("command not recognized: \"{}\"", cmd),
    }
    Ok(())
}

fn help() {
    println!("commands:");
    println!("\thelp\tprints this help screen");
    println!("\tset col row val\tsets the contents of cell col:row to val");
}

// TODO: Align columns for longer cell values.
fn show_spreadsheet(sheet: &engine::Spreadsheet) {
    let (w, h) = sheet.get_max_dims();
    for y in 0..h + 1 {
        for x in 0..w + 1 {
            let s = sheet.show_cell(x, y);
            print!("{}", if s == "" { "_" } else { &s });
            if x < w {
                print!(",\t");
            }
        }
        println!();
    }
}
