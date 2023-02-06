use logos::Logos;

use crate::{argumentation_framework::symbols, framework::ParserError};

use super::{expect, ParserResult};

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
    #[regex(r"[a-z0-9]+")]
    Text,
    #[error]
    #[regex(r"[ \r\n]+", logos::skip)]
    Error,
}

pub fn parse_file(input: &str) -> ParserResult<(Vec<symbols::Argument>, Vec<symbols::Attack>)> {
    let mut lex = Token::lexer(input);
    let mut args = vec![];
    let mut attacks = vec![];
    loop {
        let next = lex.next();
        if let Some(Token::Arg) = next {
            args.push(parse_argument(&mut lex)?);
        } else if let Some(Token::Attack) = next {
            attacks.push(parse_attack(&mut lex)?);
        } else if let Some(next) = next {
            return Err(ParserError::UnexpectedToken {
                found: Box::from(next),
                expected: vec![Box::from(Token::Arg), Box::from(Token::Attack)],
                position: lex.span(),
                text: lex.slice().to_owned(),
            });
        } else {
            break;
        }
    }
    Ok((args, attacks))
}

fn parse_attack(lex: &mut logos::Lexer<Token>) -> ParserResult<symbols::Attack> {
    expect(lex, Token::LeftParen)?;
    expect(lex, Token::Text)?;
    let from = lex.slice().to_owned();
    expect(lex, Token::Comma)?;
    expect(lex, Token::Text)?;
    let to = lex.slice().to_owned();
    expect(lex, Token::RightParen)?;
    expect(lex, Token::Period)?;
    Ok(symbols::Attack(from, to))
}

fn parse_argument(lex: &mut logos::Lexer<Token>) -> ParserResult<symbols::Argument> {
    expect(lex, Token::LeftParen)?;
    expect(lex, Token::Text)?;
    let id = lex.slice().to_owned();
    expect(lex, Token::RightParen)?;
    expect(lex, Token::Period)?;
    Ok(symbols::Argument(id))
}

#[cfg(test)]
mod tests {
    use crate::macros::{arg, att};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_files() {
        let af = parse_file(r#"arg(some1).arg(some2). att(some1, some2)."#).unwrap();
        assert_eq! {
                    af,
        (                vec![arg!("some1"), arg!("some2")],
                        vec![symbols::Attack("some1".into(), "some2".into())],
                    )
                }

        let af = parse_file(
            r#"
                arg(1).
                arg(2).
                arg(3).
                arg(4).
                att(2, 3).
                att (3,1) .
            "#,
        )
        .unwrap();
        assert_eq! {
                    af,
        (                vec![arg!("1"), arg!("2"), arg!("3"), arg!("4")],
                        vec![att!("2", "3"), att!("3", "1")],
         )
                }
    }
}
