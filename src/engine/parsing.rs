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

    pub fn parse_int(self) -> ParseResult<i64> {
        let p = self.match_pred(u8::is_ascii_digit, "is_ascii_digit")?;
        let x = p.get().parse::<i64>()?;
        Ok(p.replace(x))
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

    pub fn match_pred(mut self, pred: fn(&u8) -> bool, pred_name: &str) -> ParseResult<String> {
        let mut offset = 0;
        let rest_bytes = &self.s.as_bytes()[self.i..];
        for c in rest_bytes {
            if !pred(c) {
                break;
            }
            offset += 1;
        }

        if offset == 0 {
            let err = format!(
                "Expected bytes matching \"{}\", but got \"{}\"",
                pred_name,
                &self.s[self.i..],
            );
            return Err(Error(err));
        }

        let word = String::from_utf8(rest_bytes[..offset].to_vec())?;
        self.i += offset;
        Ok(self.replace(word))
    }

    // One or more repetitions of `once`.
    pub fn repeat<T2: Clone>(self, once: Transformer<(), T2>) -> ParseResult<Vec<T2>> {
        let mut p = self.drop();
        let mut xs = Vec::new();
        loop {
            // This is really slow right now because we would copy `self.s`.
            // Fix is to change `self.s` to a `&str`.
            let pp = once(p.clone());
            match pp {
                Ok(pp) => {
                    xs.push(pp.get());
                    p = pp.drop();
                }
                Err(e) => {
                    if xs.is_empty() {
                        return Err(e);
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(p.replace(xs))
    }

    pub fn drop(self) -> Parsing<()> {
        self.replace(())
    }
}

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Error {
        Error(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Error {
        Error(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TR = Result<(), Error>;

    #[test]
    fn test_skip() -> TR {
        let p = Parsing::new("foo".to_string()).skip("fo")?;
        assert_eq!(p.i, 2);
        assert_eq!(p.s, "foo");
        Ok(())
    }

    #[test]
    fn test_parse_int_respects_end_of_string() -> TR {
        let p = Parsing::new("123".to_string()).parse_int()?;
        assert_eq!(p.val, 123);
        assert_eq!(p.i, 3);
        Ok(())
    }

    #[test]
    fn test_parse_int_respects_alpha_chars() -> TR {
        let p = Parsing::new("456foo".to_string()).parse_int()?;
        assert_eq!(p.val, 456);
        assert_eq!(p.i, 3);
        Ok(())
    }

    #[test]
    fn test_done() -> TR {
        let p = Parsing::new("123".to_string()).parse_int()?.done()?;
        assert_eq!(p.val, 123);
        assert_eq!(p.i, 3);
        Ok(())
    }

    #[test]
    fn test_match_pred() -> TR {
        let p = Parsing::new("abcdf".to_string())
            .match_pred(|c| (*c as char) < 'd', "c < 'd'")?;
        assert_eq!(p.val, "abc");
        Ok(())
    }

    #[test]
    fn test_repeat() -> TR {
        let p = Parsing::new("a, a, b, c".to_string()).repeat(|p| p.skip("a, "))?;
        assert_eq!(p.val.len(), 2);
        assert_eq!(p.i, 6);
        Ok(())
    }

    /* Model test:

    #[test]
    fn test_name() -> TR {
        let p = Parsing::new("".to_string())?;
        Ok(())
    }

    */
}
