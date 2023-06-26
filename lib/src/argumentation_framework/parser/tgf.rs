use logos::{Lexer, Logos};

use crate::argumentation_framework::symbols;

use super::{expect, ParserError, ParserResult, RawArgument, RawAttack};

#[derive(Debug, PartialEq, Eq, Logos, Clone, Copy)]
pub enum Token {
    #[error]
    #[regex(r"[\r\n]+", logos::skip)]
    Error,
    #[token("#")]
    Hash,
    #[token("?")]
    Optional,
    #[regex(r"[a-z][a-zA-Z0-9_-]*")]
    Text,
    #[regex(" +")]
    Whitespace,
}

pub fn parse_file(input: &str) -> ParserResult<(Vec<symbols::Argument>, Vec<symbols::Attack>)> {
    let mut lex = Token::lexer(input);
    let args = parse_arguments(&mut lex)?;
    let attacks = parse_attacks(&mut lex)?;
    Ok((args, attacks))
}

fn parse_attacks(lex: &mut Lexer<Token>) -> ParserResult<Vec<symbols::Attack>> {
    let mut attacks = vec![];
    loop {
        let next = lex.next();
        match next {
            Some(Token::Text) => {
                let from = lex.slice().to_owned();
                expect(lex, Token::Whitespace)?;
                expect(lex, Token::Text)?;
                let to = lex.slice().to_owned();
                let optional = if lex.remainder().starts_with('?') {
                    lex.next();
                    true
                } else {
                    false
                };
                attacks.push(symbols::Attack { from, to, optional })
            }
            Some(token) => {
                break Err(ParserError::UnexpectedToken {
                    found: Box::from(token),
                    expected: vec![Box::from(Token::Text)],
                    position: lex.span(),
                    text: lex.slice().to_owned(),
                })
            }
            None => break Ok(attacks),
        }
    }
}

fn parse_arguments(lex: &mut Lexer<Token>) -> ParserResult<Vec<symbols::Argument>> {
    let mut args = vec![];
    loop {
        let next = lex.next();
        match next {
            Some(Token::Text) => {
                let id = lex.slice().to_owned();
                let optional = if lex.remainder().starts_with('?') {
                    lex.next();
                    true
                } else {
                    false
                };
                args.push(symbols::Argument { id, optional })
            }
            Some(Token::Hash) => break,
            Some(token) => {
                return Err(ParserError::UnexpectedToken {
                    found: Box::from(token),
                    expected: vec![Box::from(Token::Text), Box::from(Token::Hash)],
                    position: lex.span(),
                    text: lex.slice().to_owned(),
                })
            }
            None => {}
        }
    }
    Ok(args)
}

impl From<RawArgument> for symbols::Argument {
    fn from(raw: RawArgument) -> Self {
        Self {
            id: raw.id,
            optional: false,
        }
    }
}

impl From<RawAttack> for symbols::Attack {
    fn from(raw: RawAttack) -> Self {
        Self {
            from: raw.from,
            to: raw.to,
            optional: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::{arg, att};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_files() {
        let af = parse_file(
            r#"a1
a2
a3?
a4
#
a1 a2?
a1 a3"#,
        )
        .unwrap();
        assert_eq! {
            af,
            (
                vec![arg!("a1"), arg!("a2"), arg!("a3" opt), arg!("a4")],
                vec![att!("a1", "a2" opt), att!("a1", "a3")]
            )
        }
    }
}
