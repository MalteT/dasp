use crate::{framework::ParserError, Result};

use super::{symbols, ArgumentID, Patch};

mod apx;
mod apxm;
mod tgf;
mod tgfm;
type ParserResult<T> = Result<T, ParserError>;

pub fn parse_apx_tgf(input: &str) -> ParserResult<(Vec<symbols::Arg>, Vec<symbols::Att>)> {
    apx::parse_file(input).or_else(|_| tgf::parse_file(input))
}

pub fn parse_apxm_tgfm_patch_line(input: &str) -> ParserResult<Vec<Patch>> {
    apxm::parse_line(input).or_else(|_| tgfm::parse_line(input))
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

/// Expect the given Token and fail if it's not present
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
    use crate::macros::{arg, att};

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
