use logos::{Lexer, Logos};

use crate::argumentation_framework::{symbols, Patch};

use super::{expect, ParserError, ParserResult};

#[derive(Debug, PartialEq, Eq, Logos)]
pub enum Token {
    #[token("arg")]
    Arg,
    #[token("att")]
    Attack,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token(".")]
    Period,
    #[token(",")]
    Comma,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token(":")]
    Colon,
    #[regex(r"[a-z0-9]+")]
    Text,
    #[error]
    #[regex(r"[ \r\n]+", logos::skip)]
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddDel {
    Add,
    Del,
}

impl AddDel {
    fn arg(&self, arg: symbols::Argument) -> Patch {
        match self {
            Self::Add => Patch::AddArgument(arg),
            Self::Del => Patch::RemoveArgument(arg),
        }
    }
    fn att(&self, att: symbols::Attack) -> Patch {
        match self {
            Self::Add => Patch::AddAttack(att),
            Self::Del => Patch::RemoveAttack(att),
        }
    }
}

/// Parse a full update line
///
/// # Example
/// - `+att(a1, a3).`
/// - `-att(a2,a1).`
/// - `+arg(a4):att(a4, a1):att(a2, a4).`
/// - `-arg(a3).`
pub fn parse_line(input: &str) -> ParserResult<Vec<Patch>> {
    let mut lex = Token::lexer(input);
    let add_del = parse_add_del(&mut lex)?;
    let mut patches = vec![parse_patch(&mut lex, add_del)?];
    loop {
        match lex.next() {
            Some(Token::Colon) => {
                // A colon leads to another patch
                patches.push(parse_patch(&mut lex, add_del)?);
            }
            Some(Token::Period) => break Ok(patches),
            None => {
                break Err(ParserError::UnexpectedEndOfInput {
                    expected: vec![Box::from(Token::Colon), Box::from(Token::Period)],
                })
            }
            Some(other) => {
                break Err(ParserError::UnexpectedToken {
                    found: Box::from(other),
                    expected: vec![Box::from(Token::Colon), Box::from(Token::Period)],
                    position: lex.span(),
                    text: lex.slice().into(),
                })
            }
        }
    }
}

fn parse_patch(lex: &mut Lexer<Token>, add_del: AddDel) -> ParserResult<Patch> {
    let patch = match lex.next() {
        Some(Token::Arg) => add_del.arg(parse_arg_singleton(lex)?),
        Some(Token::Attack) => add_del.att(parse_att_tuple(lex)?),
        Some(other) => {
            return Err(ParserError::UnexpectedToken {
                found: Box::from(other),
                expected: vec![Box::from(Token::Arg), Box::from(Token::Attack)],
                position: lex.span(),
                text: lex.slice().into(),
            })
        }
        None => {
            return Err(ParserError::UnexpectedEndOfInput {
                expected: vec![Box::from(Token::Arg), Box::from(Token::Attack)],
            })
        }
    };
    Ok(patch)
}

fn parse_att_tuple(lex: &mut Lexer<Token>) -> ParserResult<symbols::Attack> {
    expect(lex, Token::LeftParen)?;
    expect(lex, Token::Text)?;
    let from = lex.slice().to_owned();
    expect(lex, Token::Comma)?;
    expect(lex, Token::Text)?;
    let to = lex.slice().to_owned();
    expect(lex, Token::RightParen)?;
    Ok(symbols::Attack(from, to))
}

fn parse_arg_singleton(lex: &mut Lexer<Token>) -> ParserResult<symbols::Argument> {
    expect(lex, Token::LeftParen)?;
    expect(lex, Token::Text)?;
    let id = lex.slice().to_owned();
    expect(lex, Token::RightParen)?;
    Ok(symbols::Argument(id))
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

#[cfg(test)]
mod tests {
    use crate::macros::{arg, att};

    use super::*;

    #[test]
    fn basic_updates() {
        let patches = parse_line("+att(a1,a3).").unwrap();
        assert_eq!(patches, vec![Patch::AddAttack(att!("a1", "a3"))]);

        let patches = parse_line("-att(a2, a1).").unwrap();
        assert_eq!(patches, vec![Patch::RemoveAttack(att!("a2", "a1"))]);

        let patches = parse_line("+arg(a4):att(a4, a1):att(a2,a4).").unwrap();
        assert_eq!(
            patches,
            vec![
                Patch::AddArgument(arg!("a4")),
                Patch::AddAttack(att!("a4", "a1")),
                Patch::AddAttack(att!("a2", "a4"))
            ]
        );

        let patches = parse_line("-arg(a3).").unwrap();
        assert_eq!(patches, vec![Patch::RemoveArgument(arg!("a3"))]);
    }
}
