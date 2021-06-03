// ```
// Parsing::new(input).try_one([
//     |p| {
//         let p1 = p.parse_literal()?;
//         let p2 = p1.skip("+")?.parse_literal()?;
//         Ok(Expr::Plus(Box::new(p1.val), Box::new(e2.val)))
//     },
//     |p| Ok(()),
// ])
// ```

use std::num;

// TODO: Reduce number of clones used.

#[derive(Clone)]
struct Parsing<T>
where
    T: Clone,
{
    // TODO: Use a &str with proper lifetime constraint so that I don't have to
    // copy this left and right.
    s: String,
    i: usize,
    // TODO: I should put Result on the outside and use question marks. That
    // way I don't have to check that val.is_okay at start of every call.
    val: Result<T, Error>,
}

impl Parsing<()> {
    fn new(s: String) -> Parsing<()> {
        Parsing {
            s: s,
            i: 0,
            val: Ok(()),
        }
    }
}

impl<T: Clone> Parsing<T> {
    fn replace<T2: Clone>(&self, val: Result<T2, Error>) -> Parsing<T2> {
        Parsing {
            s: self.s.clone(),
            i: self.i,
            val: val,
        }
    }

    fn try_one<T2: Clone>(
        self,
        methods: Vec<Box<dyn Fn(&mut Parsing<T>) -> Result<T2, Error>>>,
    ) -> Parsing<T2> {
        if self.val.is_err() {
            let err = Err(self.val.clone().err().unwrap());
            return self.replace(err);
        }

        for method in methods {
            let mut p = self.clone();
            let result = method(&mut p);
            if result.is_ok() {
                return p.replace(result);
            }
        }

        let err = Error(format!(
            "No method worked parsing at {}",
            self.s.get(self.i..).unwrap_or("")
        ));
        self.replace(Err(err))
    }

    fn skip(mut self, s: &str) -> Parsing<T> {
        if self.val.is_err() {
            return self;
        }

        let rest = self.s.get(self.i..);
        if rest.map(|r| r.starts_with(s)).unwrap_or(false) {
            self.i += s.len();
            return self;
        }

        let err = format!("Expected \"{}\" but found \"{}\" instead", s, self.s);
        self.replace(Err(Error(err)))
    }

    fn parse_int(mut self) -> Result<Parsing<i64>, Error> {
        let mut int_end = self.i;
        let s_rest = self.s[self.i..].as_bytes(); // TODO: Support unicode.
        while int_end < s_rest.len() && s_rest[int_end].is_ascii_digit() {
            int_end += 1;
        }

        if int_end == self.i {
            let err = format!("Expected an int, instead found \"{}\"", &self.s[self.i..]);
            Err(Error(err))
        } else {
            let x = self.s[self.i..int_end].parse::<i64>()?;
            self.i = int_end;
            Ok(self.replace(Ok(x)))
        }
    }
}

#[derive(Debug, Clone)]
struct Error(String);

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Error {
        Error(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip() {
        let p = Parsing::new("foo".to_string());
        let p = p.skip("fo");
        assert_eq!(p.val.is_ok(), true);
        assert_eq!(p.i, 2);
        assert_eq!(p.s, "foo");
    }

    #[test]
    fn test_parse_int() -> Result<(), Error> {
        let p = Parsing::new("123".to_string()).parse_int()?;
        assert_eq!(p.val?, 123);
        assert_eq!(p.i, 3);

        let p = Parsing::new("456foo".to_string()).parse_int()?;
        assert_eq!(p.val?, 456);
        assert_eq!(p.i, 3);
        Ok(())
    }
}
