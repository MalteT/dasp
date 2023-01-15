use logos::{Lexer, Logos};

use crate::framework::af::{symbols, Patch};

use super::{expect, ParserError, ParserResult};

#[derive(Debug, PartialEq, Eq, Logos, Clone, Copy)]
pub enum Token {
    #[regex(r"[a-z0-9]+")]
    Text,
    #[regex(" +")]
    Whitespace,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token(":")]
    Colon,
    #[error]
    #[regex(r"[\r\n]+", logos::skip)]
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddDel {
    Add,
    Del,
}

impl AddDel {
    fn arg(&self, arg: symbols::Arg) -> Patch {
        match self {
            Self::Add => Patch::AddArgument(arg),
            Self::Del => Patch::DelArgument(arg),
        }
    }
    fn att(&self, att: symbols::Att) -> Patch {
        match self {
            Self::Add => Patch::AddAttack(att),
            Self::Del => Patch::DelAttack(att),
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
            Ok(add_del.att(symbols::Att {
                from: arg.id,
                to: to.id,
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

fn parse_argument(lex: &mut Lexer<Token>) -> ParserResult<symbols::Arg> {
    expect(lex, Token::Text)?;
    Ok(symbols::Arg {
        id: lex.slice().into(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{arg, att};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_patches() {
        let patches = parse_line("+1 3").unwrap();
        assert_eq!(patches, vec![Patch::AddAttack(att!("1", "3"))]);

        let patches = parse_line("-2 1").unwrap();
        assert_eq!(patches, vec![Patch::DelAttack(att!("2", "1"))]);

        let patches = parse_line("+4:4 1:2 4").unwrap();
        assert_eq!(
            patches,
            vec![
                Patch::AddArgument(arg!("4")),
                Patch::AddAttack(att!("4", "1")),
                Patch::AddAttack(att!("2", "4"))
            ]
        );

        let patches = parse_line("-3").unwrap();
        assert_eq!(patches, vec![Patch::DelArgument(arg!("3"))]);
    }
}
