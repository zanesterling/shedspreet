use std::error::Error;
use std::cmp::max;

pub struct Spreadsheet {
    // The maximum X and Y values of filled cells in the sheet.
    max_x: usize,
    max_y: usize,
    // The width and height of the cells array.
    arr_w: usize,
    arr_h: usize,

    cells: Vec<Cell>,
}

impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        Spreadsheet {
            max_x: 0,
            max_y: 0,
            arr_w: 1,
            arr_h: 1,
            cells: vec![Cell::empty()],
        }
    }

    pub fn show_cell(&self, x: usize, y: usize) -> String {
        let mut cell = &Cell::empty();
        if x < self.arr_w && y < self.arr_h {
            cell = &self.cells[x + y * self.arr_w]
        };

        match cell.contents.strip_prefix("=") {
            None => cell.contents.clone(),
            Some(rest) => {
                let e = Expr::parse(rest);
                match e {
                    Ok(expr) => expr.eval().to_string(),
                    Err(errstr) => errstr.to_string(),
                }
            },
        }
    }

    pub fn get_max_dims(&self) -> (usize, usize) {
        (self.max_x, self.max_y)
    }

    pub fn set(&mut self, x: usize, y: usize, contents: String) {
        self.max_x = max(x, self.max_x);
        self.max_y = max(y, self.max_y);

        if x >= self.arr_w || y >= self.arr_h {
            self.grow_array_to_fit(x, y)
        }

        let new_cell = &mut self.cells[x + y * self.arr_w];
        new_cell.contents = contents;
    }

    fn grow_array_to_fit(&mut self, x: usize, y: usize) {
        let mut new_arr_w = self.arr_w;
        let mut new_arr_h = self.arr_h;
        while new_arr_w <= x {
            new_arr_w *= 2;
        }

        while new_arr_h <= y {
            new_arr_h *= 2;
        }
        let mut new_cells = Vec::with_capacity(new_arr_w * new_arr_h);
        for yy in 0..self.arr_h {
            for xx in 0..self.arr_w {
                new_cells.push(self.cells[xx + yy * self.arr_w].clone());
            }
            for _ in self.arr_w..new_arr_w {
                new_cells.push(Cell::empty());
            }
        }
        new_cells.resize(new_arr_w * new_arr_h, Cell::empty());
        self.cells = new_cells;
        self.arr_w = new_arr_w;
        self.arr_h = new_arr_h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grow_sheet() {
        let mut sheet = Spreadsheet::new();
        sheet.set(0, 0, "hi".to_string());
        sheet.set(1, 1, "hello".to_string());
        assert_eq!(sheet.show_cell(0, 0), "hi");
        assert_eq!(sheet.show_cell(1, 1), "hello");
    }
}

#[derive(Clone)]
struct Cell {
    contents: String,
    backrefs: Vec<CellRef>,
}

impl Cell {
    pub fn empty() -> Cell {
        Cell {
            contents: "".to_string(),
            backrefs: vec![],
        }
    }
}

#[derive(Clone)]
struct CellRef(usize, usize);

#[derive(PartialEq, Eq, Debug)]
enum Expr {
    Int(i64),
    Plus(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn parse(s: &str) -> Result<Expr, Box<dyn Error>> {
        match s.split_once('+') {
            Some((x, rest)) => {
                let x: i64 = x.parse()?;
                let rest = Expr::parse(rest)?;
                Ok(Expr::Plus(
                    Box::new(Expr::Int(x)),
                    Box::new(rest)))
            },
            None => {
                let x = s.parse::<i64>()?;
                Ok(Expr::Int(x))
            }
        }
    }

    fn eval(&self) -> i64 {
        match self {
            Expr::Int(x) => *x,
            Expr::Plus(bx, by) => bx.eval() + by.eval(),
        }
    }
}


#[cfg(test)]
mod expr_tests {
    use super::*;

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let e = Expr::parse("13+2+5")?;
        assert_eq!(
            e,
            Expr::Plus(Box::new(Expr::Int(13)),
                       Box::new(Expr::Plus(Box::new(Expr::Int(2)),
                                           Box::new(Expr::Int(5))))));
        Ok(())
    }

    #[test]
    fn test_addition() -> Result<(), Box<dyn Error>> {
        let e = Expr::parse("13+2+5")?;
        assert_eq!(e.eval(), 20);
        Ok(())
    }
}
