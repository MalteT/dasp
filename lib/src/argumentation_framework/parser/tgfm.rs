use logos::{Lexer, Logos};

use crate::argumentation_framework::{symbols, Patch};

use super::{expect, ParserError, ParserResult};

#[derive(Debug, PartialEq, Eq, Logos, Clone, Copy)]
pub enum Token {
    #[token(":")]
    Colon,
    #[error]
    #[regex(r"[\r\n]+", logos::skip)]
    Error,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[regex(r"[a-z][a-zA-Z0-9_-]*")]
    Text,
    #[regex(" +")]
    Whitespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddDel {
    Add,
    Del,
}

impl AddDel {
    fn arg(&self, arg: symbols::Argument) -> Patch {
        match self {
            Self::Add => Patch::EnableArgument(arg),
            Self::Del => Patch::DisableArgument(arg),
        }
    }
    fn att(&self, att: symbols::Attack) -> Patch {
        match self {
            Self::Add => Patch::EnableAttack(att),
            Self::Del => Patch::DisableAttack(att),
        }
    }
}

pub fn parse_line(input: &str) -> ParserResult<Vec<Patch>> {
    let mut lex = Token::lexer(input);
    let add_del = parse_add_del(&mut lex)?;
    let mut patches = vec![];
    while !lex.remainder().is_empty() {
        patches.push(parse_patch(&mut lex, add_del)?);
    }
    Ok(patches)
}

fn parse_patch(lex: &mut Lexer<Token>, add_del: AddDel) -> ParserResult<Patch> {
    let arg = parse_argument(lex)?;
    match lex.next() {
        // End of patch, just add/del the single argument
        None | Some(Token::Colon) => Ok(add_del.arg(arg)),
        // Whitespace followed by a second argument to describe an attack change
        Some(Token::Whitespace) => {
            let to = parse_argument(lex)?;
            match lex.next() {
                // What we expect here
                Some(Token::Colon) | None => {}
                Some(other) => {
                    return Err(ParserError::UnexpectedToken {
                        found: Box::from(other),
                        expected: vec![Box::from(Token::Colon)],
                        position: lex.span(),
                        text: lex.slice().into(),
                    })
                }
            }
            Ok(add_del.att(symbols::Attack {
                from: arg.id,
                to: to.id,
                optional: false,
            }))
        }
        Some(other) => Err(ParserError::UnexpectedToken {
            found: Box::from(other),
            expected: vec![Box::from(Token::Colon), Box::from(Token::Whitespace)],
            position: lex.span(),
            text: lex.slice().into(),
        }),
    }
}

fn parse_add_del(lex: &mut Lexer<Token>) -> ParserResult<AddDel> {
    match lex.next() {
        Some(Token::Plus) => Ok(AddDel::Add),
        Some(Token::Minus) => Ok(AddDel::Del),
        Some(other) => Err(ParserError::UnexpectedToken {
            found: Box::from(other),
            expected: vec![Box::from(Token::Plus), Box::from(Token::Minus)],
            position: lex.span(),
            text: lex.slice().into(),
        }),
        None => Err(ParserError::UnexpectedEndOfInput {
            expected: vec![Box::from(Token::Plus), Box::from(Token::Minus)],
        }),
    }
}

fn parse_argument(lex: &mut Lexer<Token>) -> ParserResult<symbols::Argument> {
    expect(lex, Token::Text)?;
    Ok(symbols::Argument {
        id: lex.slice().into(),
        optional: false,
    })
}

#[cfg(test)]
mod tests {
    use crate::macros::{arg, att};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_patches() {
        let patches = parse_line("+1 3").unwrap();
        assert_eq!(patches, vec![Patch::EnableAttack(att!("1", "3"))]);

        let patches = parse_line("-2 1").unwrap();
        assert_eq!(patches, vec![Patch::DisableAttack(att!("2", "1"))]);

        let patches = parse_line("+4:4 1:2 4").unwrap();
        assert_eq!(
            patches,
            vec![
                Patch::EnableArgument(arg!("4")),
                Patch::EnableAttack(att!("4", "1")),
                Patch::EnableAttack(att!("2", "4"))
            ]
        );

        let patches = parse_line("-3").unwrap();
        assert_eq!(patches, vec![Patch::DisableArgument(arg!("3"))]);
    }
}
