use logos::{Lexer, Logos};

use crate::argumentation_framework::symbols;

use super::{expect, ParserError, ParserResult, RawArgument, RawAttack};

#[derive(Debug, PartialEq, Eq, Logos, Clone, Copy)]
pub enum Token {
    #[regex(r"[a-z0-9]+")]
    Text,
    #[token("#")]
    Hash,
    #[regex(" +")]
    Whitespace,
    #[error]
    #[regex(r"[\r\n]+", logos::skip)]
    Error,
}

pub fn parse_file(input: &str) -> ParserResult<(Vec<symbols::Arg>, Vec<symbols::Att>)> {
    let mut lex = Token::lexer(input);
    let args = parse_arguments(&mut lex)?;
    let attacks = parse_attacks(&mut lex)?;
    Ok((args, attacks))
}

fn parse_attacks(lex: &mut Lexer<Token>) -> ParserResult<Vec<symbols::Att>> {
    let mut attacks = vec![];
    loop {
        let next = lex.next();
        match next {
            Some(Token::Text) => {
                let from = lex.slice().to_owned();
                expect(lex, Token::Whitespace)?;
                expect(lex, Token::Text)?;
                let to = lex.slice().to_owned();
                attacks.push(symbols::Att { from, to })
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

fn parse_arguments(lex: &mut Lexer<Token>) -> ParserResult<Vec<symbols::Arg>> {
    let mut args = vec![];
    loop {
        let next = lex.next();
        match next {
            Some(Token::Text) => args.push(symbols::Arg {
                id: lex.slice().to_owned(),
            }),
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

impl From<RawArgument> for symbols::Arg {
    fn from(raw: RawArgument) -> Self {
        Self { id: raw.id }
    }
}

impl From<RawAttack> for symbols::Att {
    fn from(raw: RawAttack) -> Self {
        Self {
            from: raw.from,
            to: raw.to,
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
            r#"1
2
3
4
#
1 2
1 3"#,
        )
        .unwrap();
        assert_eq! {
            af,
            (
                vec![arg!("1"), arg!("2"), arg!("3"), arg!("4")],
                vec![att!("1", "2"), att!("1", "3")]
            )
        }
    }
}
