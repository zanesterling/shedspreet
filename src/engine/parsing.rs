// ```
// Parsing::new(input).try_one([
//     |p| Err(Error("oh no, an error!")),
//     |p| {
//         let p1 = p.parse_literal()?;
//         let p2 = p1.skip("+")?.parse_literal()?;
//         Ok(p2.replace(Expr::Plus(Box::new(p1.val), Box::new(e2.val))))
//     },
// ])?;
// ```

use std::num;

// TODO: Reduce number of clones used.

#[derive(Clone)]
pub struct Parsing<T>
where
    T: Clone,
{
    // TODO: Use a &str with proper lifetime constraint so that I don't have to
    // copy this left and right.
    s: String,
    i: usize,
    val: T,
}

pub type P<T> = Parsing<T>;
pub type ParseResult<T> = Result<Parsing<T>, Error>;
pub type Transformer<T1, T2> = fn(Parsing<T1>) -> ParseResult<T2>;

impl Parsing<()> {
    pub fn new(s: String) -> Parsing<()> {
        Parsing {
            s: s,
            i: 0,
            val: (),
        }
    }
}

impl<T: Clone> Parsing<T> {
    pub fn get(&self) -> T {
        self.val.clone()
    }

    pub fn replace<T2: Clone>(self, val: T2) -> Parsing<T2> {
        Parsing {
            s: self.s.clone(),
            i: self.i,
            val: val,
        }
    }

    pub fn try_one<T2: Clone>(self, methods: Vec<Transformer<T, T2>>) -> ParseResult<T2> {
        for method in methods {
            let result = method(self.clone());
            if result.is_ok() {
                return result;
            }
        }

        Err(Error(format!(
            "No method worked parsing at \"{}\"",
            self.s.get(self.i..).unwrap_or("")
        )))
    }

    pub fn skip(mut self, s: &str) -> ParseResult<T> {
        if self.s[self.i..].starts_with(s) {
            self.i += s.len();
            Ok(self)
        } else {
            let err = format!(
                "Expected \"{}\" but found \"{}\" instead",
                s,
                self.s[self.i..].to_string()
            );
            Err(Error(err))
        }
    }

    pub fn parse_int(mut self) -> ParseResult<i64> {
        let mut int_end = 0;
        let s_rest = self.s[self.i..].as_bytes(); // TODO: Support unicode.
        while int_end < s_rest.len() && s_rest[int_end].is_ascii_digit() {
            int_end += 1;
        }

        if int_end == 0 {
            let err = format!("Expected an int, instead found \"{}\"", &self.s[self.i..]);
            Err(Error(err))
        } else {
            let x = self.s[self.i..self.i + int_end].parse::<i64>()?;
            self.i += int_end;
            Ok(self.replace(x))
        }
    }

    pub fn done(self) -> ParseResult<T> {
        if self.i == self.s.len() {
            Ok(self)
        } else {
            Err(Error(format!(
                "expected end of string, instead found \"{}\"",
                self.s[self.i..].to_string()
            )))
        }
    }

    pub fn wrapped<T2: Clone>(
        self,
        left: &str,
        inner: Transformer<T, T2>,
        right: &str,
    ) -> ParseResult<T2> {
        let p = self.skip(left)?;
        inner(p)?.skip(right)
    }
}

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Error {
        Error(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip() -> Result<(), Error> {
        let p = Parsing::new("foo".to_string()).skip("fo")?;
        assert_eq!(p.i, 2);
        assert_eq!(p.s, "foo");
        Ok(())
    }

    #[test]
    fn test_parse_int_respects_end_of_string() -> Result<(), Error> {
        let p = Parsing::new("123".to_string()).parse_int()?;
        assert_eq!(p.val, 123);
        assert_eq!(p.i, 3);
        Ok(())
    }

    #[test]
    fn test_parse_int_respects_alpha_chars() -> Result<(), Error> {
        let p = Parsing::new("456foo".to_string()).parse_int()?;
        assert_eq!(p.val, 456);
        assert_eq!(p.i, 3);
        Ok(())
    }

    #[test]
    fn test_done() -> Result<(), Error> {
        let p = Parsing::new("123".to_string()).parse_int()?.done()?;
        assert_eq!(p.val, 123);
        assert_eq!(p.i, 3);
        Ok(())
    }
}
