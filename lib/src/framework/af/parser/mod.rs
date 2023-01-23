use crate::{framework::ParserError, Result};

use super::{symbols, ArgumentID};

pub mod apx;
mod apxm;
mod tgf;
pub mod tgfm;

type ParserResult<T> = Result<T, ParserError>;

#[macro_export]
macro_rules! arg {
    ($name:literal) => {
        symbols::Arg { id: $name.into() }
    };
}

#[macro_export]
macro_rules! att {
    ($from:literal, $to:literal) => {
        symbols::Att {
            from: $from.into(),
            to: $to.into(),
        }
    };
}

pub fn parse_apx_tgf(input: &str) -> ParserResult<(Vec<symbols::Arg>, Vec<symbols::Att>)> {
    apx::parse_file(input).or_else(|_| tgf::parse_file(input))
}

#[derive(Debug)]
pub struct RawArgument {
    id: ArgumentID,
}

#[derive(Debug)]
pub struct RawAttack {
    from: ArgumentID,
    to: ArgumentID,
}

fn expect<'l, T>(lex: &mut logos::Lexer<'l, T>, expected: T) -> ParserResult<T>
where
    T: logos::Logos<'l, Source = str> + std::cmp::PartialEq + std::fmt::Debug + 'static,
{
    let next = lex.next();
    match next {
        Some(next) if next == expected => Ok(next),
        Some(next) => Err(ParserError::UnexpectedToken {
            found: Box::from(next),
            expected: vec![Box::from(expected)],
            position: lex.span(),
            text: lex.slice().to_owned(),
        }),
        None => Err(ParserError::UnexpectedEndOfInput {
            expected: vec![Box::from(expected)],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_apx_tgf_files() {
        let af = parse_apx_tgf(r#"arg(a1).arg(b3)."#).unwrap();
        assert_eq!(af, (vec![arg!("a1"), arg!("b3")], vec![],));

        let af = parse_apx_tgf(
            r#"1
2
#
2 1"#,
        )
        .unwrap();
        assert_eq!(af, (vec![arg!("1"), arg!("2")], vec![att!("2", "1")],));
    }
}
