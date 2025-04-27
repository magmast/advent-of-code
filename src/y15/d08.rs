use anyhow::{Error, Result, anyhow};
use winnow::Parser;

mod parser {
    use winnow::{
        Parser, Result,
        combinator::{alt, delimited, fail, preceded, repeat},
        error::{StrContext, StrContextValue},
        token::{any, none_of, one_of},
    };

    fn hexdigit(input: &mut &str) -> Result<char> {
        any.verify(|c: &char| c.is_ascii_hexdigit())
            .context(StrContext::Label("hexadecimal digit"))
            .parse_next(input)
    }

    fn escaped_char(input: &mut &str) -> Result<char> {
        preceded(
            '\\',
            alt((
                one_of(['"', '\\']),
                preceded('x', (hexdigit, hexdigit).map(|_| 't')),
                fail.context(StrContext::Label("escaped character"))
                    .context(StrContext::Expected(StrContextValue::CharLiteral('"')))
                    .context(StrContext::Expected(StrContextValue::CharLiteral('\\')))
                    .context(StrContext::Expected(StrContextValue::StringLiteral(
                        "unicode sequence",
                    ))),
            )),
        )
        .parse_next(input)
    }

    pub fn string(input: &mut &str) -> Result<String> {
        delimited(
            '"',
            repeat(1.., alt((escaped_char, none_of(['"'])))).fold(String::new, |mut acc, ch| {
                acc.push(ch);
                acc
            }),
            '"',
        )
        .context(StrContext::Label("string"))
        .parse_next(input)
    }

    fn doublescaped_char(input: &mut &str) -> Result<String> {
        preceded(
            '\\',
            alt((
                one_of(['"', '\\']).map(|ch| format!(r"\{}", ch)),
                ('x', hexdigit, hexdigit).take().map(|i| i.to_string()),
                fail.context(StrContext::Label("double escaped character")),
            )),
        )
        .map(|s| format!(r"\\{}", s))
        .parse_next(input)
    }

    pub fn doublescaped(input: &mut &str) -> Result<String> {
        delimited(
            '"',
            repeat(
                1..,
                alt((
                    doublescaped_char,
                    none_of(['"']).map(|ch| format!("{}", ch)),
                    fail.context(StrContext::Label("double escaped string character")),
                )),
            )
            .fold(String::new, |mut acc, s| {
                acc += &s;
                acc
            }),
            '"'.context(StrContext::Label("string termination")),
        )
        .map(|s| format!(r#""\"{}\"""#, s))
        .parse_next(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_doublescaped_char() {
            let i = r#"\""#;
            let s = doublescaped_char(&mut &i[..]).unwrap();
            assert_eq!(s, r#"\\\""#);

            let i = r#"\\"#;
            let s = doublescaped_char(&mut &i[..]).unwrap();
            assert_eq!(s, r"\\\\");

            let i = r#"\x27"#;
            let s = doublescaped_char(&mut &i[..]).unwrap();
            assert_eq!(s, r"\\x27");
        }
    }
}

fn escape(s: &str) -> Result<String> {
    parser::string.parse(s).map_err(|err| anyhow!("\n{err}"))
}

fn doublescape(s: &str) -> Result<String> {
    parser::doublescaped
        .parse(s)
        .map_err(|err| anyhow!("\n{err}"))
}

async fn answer(mut parser: impl FnMut(&str) -> Result<String>) -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d08.txt").await?;
    let escaped = input.lines().map(|line| parser(line.trim())).try_fold(
        String::new(),
        |mut acc, line| {
            acc.push_str(&line?);
            acc.push('\n');
            Ok::<_, Error>(acc)
        },
    )?;
    println!(
        "Answer: {}",
        input.len().max(escaped.len()) - escaped.len().min(input.len())
    );
    Ok(())
}

pub async fn p1() -> Result<()> {
    answer(escape).await
}

pub async fn p2() -> Result<()> {
    answer(doublescape).await
}
