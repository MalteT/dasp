use logos::Logos;

use crate::{argumentation_framework::symbols, framework::ParserError};

use super::{expect, ParserResult};

#[derive(Debug, PartialEq, Eq, Logos)]
pub enum Token {
    #[token("arg")]
    Arg,
    #[token("att")]
    Attack,
    #[token("opt")]
    Optional,
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

enum ArgOrAttack {
    Arg(String),
    Attack(String, String),
}

pub fn parse_file(input: &str) -> ParserResult<(Vec<symbols::Argument>, Vec<symbols::Attack>)> {
    let mut lex = Token::lexer(input);
    let mut args = vec![];
    let mut attacks = vec![];
    let mut optionals = vec![];
    loop {
        let next = lex.next();
        if let Some(Token::Arg) = next {
            args.push(parse_argument(&mut lex)?);
        } else if let Some(Token::Attack) = next {
            attacks.push(parse_attack(&mut lex)?);
        } else if let Some(Token::Optional) = next {
            optionals.push(parse_optional(&mut lex)?);
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
    optionals.into_iter().try_for_each(|opt| {
        match opt {
            ArgOrAttack::Arg(arg_id) => match args.iter_mut().find(|arg| arg.id == arg_id) {
                Some(arg) => arg.optional = true,
                None => return Err(ParserError::OptionalArgumentNotFound { arg_id }),
            },
            ArgOrAttack::Attack(from, to) => match attacks
                .iter_mut()
                .find(|attack| attack.from == from && attack.to == to)
            {
                Some(attack) => attack.optional = true,
                None => return Err(ParserError::OptionalAttackNotFound { from, to }),
            },
        }
        Ok(())
    })?;
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
    Ok(symbols::Attack {
        from,
        to,
        optional: false,
    })
}

fn parse_argument(lex: &mut logos::Lexer<Token>) -> ParserResult<symbols::Argument> {
    expect(lex, Token::LeftParen)?;
    expect(lex, Token::Text)?;
    let id = lex.slice().to_owned();
    expect(lex, Token::RightParen)?;
    expect(lex, Token::Period)?;
    Ok(symbols::Argument {
        id,
        optional: false,
    })
}

fn parse_optional(lex: &mut logos::Lexer<Token>) -> ParserResult<ArgOrAttack> {
    expect(lex, Token::LeftParen)?;
    let arg_or_attack = match lex.next() {
        Some(Token::Arg) => {
            expect(lex, Token::LeftParen)?;
            expect(lex, Token::Text)?;
            let arg = lex.slice().to_owned();
            expect(lex, Token::RightParen)?;
            Ok(ArgOrAttack::Arg(arg))
        }
        Some(Token::Attack) => {
            expect(lex, Token::LeftParen)?;
            expect(lex, Token::Text)?;
            let from = lex.slice().to_owned();
            expect(lex, Token::Comma)?;
            expect(lex, Token::Text)?;
            let to = lex.slice().to_owned();
            expect(lex, Token::RightParen)?;
            Ok(ArgOrAttack::Attack(from, to))
        }
        Some(next) => Err(ParserError::UnexpectedToken {
            found: Box::from(next),
            expected: vec![Box::from(Token::Arg), Box::from(Token::Attack)],
            position: lex.span(),
            text: lex.slice().to_owned(),
        }),
        None => Err(ParserError::UnexpectedEndOfInput {
            expected: vec![Box::from(Token::Arg), Box::from(Token::Attack)],
        }),
    }?;
    expect(lex, Token::RightParen)?;
    expect(lex, Token::Period)?;
    Ok(arg_or_attack)
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
            (   vec![arg!("some1"), arg!("some2")],
                vec![att!("some1", "some2")],
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
                opt(arg(2)).
                opt(att(2,3)) .
            "#,
        )
        .unwrap();
        assert_eq! {
            af,
            ( vec![arg!("1"), arg!("2" opt), arg!("3"), arg!("4")],
              vec![att!("2", "3" opt), att!("3", "1")],
            )
        }
    }
}
