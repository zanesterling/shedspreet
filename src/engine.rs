use std::cmp::max;
use std::fmt;
use std::num;

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
                    Ok(expr) => expr
                        .eval()
                        .map_or_else(|e| e.to_string(), |v| v.to_string()),
                    Err(errstr) => errstr.to_string(),
                }
            }
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
mod spreadsheet_tests {
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

// TODO: Implement cross-cell expression references.
#[derive(Clone)]
struct CellRef(usize, usize);

/***** Parsing, Expressions, Evaluation, Values. *****/

#[derive(PartialEq, Eq, Debug, Clone)]
enum Expr {
    Int(i64),
    Bool(bool),
    Plus(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

mod parsing;
use parsing::{ParseResult, Parsing, Transformer, P};

impl<T: Clone> P<T> {
    fn e_int(self) -> ParseResult<Expr> {
        let p = self.parse_int()?;
        let e = Expr::Int(p.get());
        Ok(p.replace(e))
    }

    fn e_bool(self) -> ParseResult<Expr> {
        self.try_one(vec![
            |p| Ok(p.skip("true")?.replace(Expr::Bool(true))),
            |p| Ok(p.skip("false")?.replace(Expr::Bool(false))),
        ])
    }

    fn plus(self) -> ParseResult<Expr> {
        let inner: Transformer<T, Expr> = |p| {
            let p1 = p.expr()?;
            let e1 = Box::new(p1.get());
            let p2 = p1.skip("+")?.expr()?;
            let e2 = Box::new(p2.get());
            Ok(p2.replace(Expr::Plus(e1, e2)))
        };
        self.wrapped("(", inner, ")")
    }

    fn expr(self) -> ParseResult<Expr> {
        self.try_one(vec![|p| p.e_int(), |p| p.e_bool(), |p| p.plus()])
    }
}

impl Expr {
    // TODO: Add parsing for Bool, Eq
    fn parse(s: &str) -> Result<Expr, Error> {
        let p = Parsing::new(s.to_string()).expr()?.done()?;
        Ok(p.get())
    }

    fn eval(&self) -> Result<Value, Error> {
        match self {
            Expr::Int(x) => Ok(Value::Int(*x)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Plus(x, y) => match (x.eval()?, y.eval()?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
                _ => Err(Error::TypeError),
            },
            Expr::Eq(x, y) => match (x.eval()?, y.eval()?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x == y)),
                _ => Err(Error::TypeError),
            },
            Expr::If(b, x, y) => match b.eval()? {
                Value::Bool(b) => Ok(if b { x.eval()? } else { y.eval()? }),
                _ => Err(Error::TypeError),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Value {
    Int(i64),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Int(x) => x.to_string(),
                Value::Bool(b) => b.to_string(),
            }
        )
    }
}

#[derive(Debug, Clone)]
enum Error {
    //ParseError(Box<dyn error::Error>),
    DescriptiveError(String),
    TypeError,
}

impl From<parsing::Error> for Error {
    fn from(e: parsing::Error) -> Error {
        let parsing::Error(s) = e;
        Error::DescriptiveError(s)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Error {
        Error::DescriptiveError(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod expr_tests {
    use super::*;

    #[test]
    fn test_parse_int() -> Result<(), Error> {
        let e = Expr::parse("52")?;
        assert_eq!(e, Expr::Int(52));
        Ok(())
    }

    #[test]
    fn test_parse_bool() -> Result<(), Error> {
        let e = Expr::parse("true")?;
        assert_eq!(e, Expr::Bool(true));
        let e = Expr::parse("false")?;
        assert_eq!(e, Expr::Bool(false));
        Ok(())
    }

    #[test]
    fn test_parse_plus_basic() -> Result<(), Error> {
        //let e = Expr::parse("(13+2)")?;
        let e = Parsing::new("(13+2)".to_string()).plus()?.done()?.get();
        assert_eq!(
            e,
            Expr::Plus(Box::new(Expr::Int(13)), Box::new(Expr::Int(2)))
        );
        Ok(())
    }

    #[test]
    fn test_parse_plus_nested() -> Result<(), Error> {
        let e = Expr::parse("(13+(2+5))")?;
        assert_eq!(
            e,
            Expr::Plus(
                Box::new(Expr::Int(13)),
                Box::new(Expr::Plus(Box::new(Expr::Int(2)), Box::new(Expr::Int(5))))
            )
        );
        Ok(())
    }

    #[test]
    fn test_addition() -> Result<(), Error> {
        let e = Expr::parse("(13+(2+5))")?;
        assert_eq!(e.eval()?, Value::Int(20));
        Ok(())
    }

    #[test]
    fn test_eq() -> Result<(), Error> {
        let e = Expr::Eq(Box::new(Expr::Int(2)), Box::new(Expr::Int(2)));
        assert_eq!(e.eval()?, Value::Bool(true));

        let e = Expr::Eq(Box::new(Expr::Int(2)), Box::new(Expr::Int(3)));
        assert_eq!(e.eval()?, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_if() -> Result<(), Error> {
        let e = Expr::If(
            Box::new(Expr::Bool(true)),
            Box::new(Expr::Int(2)),
            Box::new(Expr::Int(3)),
        );
        assert_eq!(e.eval()?, Value::Int(2));

        let e = Expr::If(
            Box::new(Expr::Bool(false)),
            Box::new(Expr::Int(2)),
            Box::new(Expr::Int(3)),
        );
        assert_eq!(e.eval()?, Value::Int(3));
        Ok(())
    }
}
