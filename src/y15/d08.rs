use anyhow::Result;

mod parser {
    use nom::{
        AsChar, IResult, Input, Offset, Parser,
        branch::alt,
        character::complete::{char, none_of, one_of, satisfy},
        combinator::{map, recognize},
        error::ParseError,
        multi::fold_many1,
        sequence::{delimited, preceded},
    };

    fn hexdigit<I, E>(input: I) -> IResult<I, char, E>
    where
        I: Input,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        satisfy(|c| c.is_ascii_hexdigit()).parse(input)
    }

    fn escaped_char<I, E>(input: I) -> IResult<I, char, E>
    where
        I: Input,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        preceded(
            char('\\'),
            alt((
                one_of("\"\\"),
                preceded(char('x'), map((hexdigit, hexdigit), |_| 't')),
            )),
        )
        .parse(input)
    }

    pub fn string<I, E>(input: I) -> IResult<I, String, E>
    where
        I: Input,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        delimited(
            char('"'),
            fold_many1(
                alt((escaped_char, none_of("\""))),
                || String::new(),
                |mut acc, ch| {
                    acc.push(ch);
                    acc
                },
            ),
            char('"'),
        )
        .parse(input)
    }

    pub trait AsStr {
        fn as_str(&self) -> &str;
    }

    impl AsStr for &str {
        fn as_str(&self) -> &str {
            self
        }
    }

    impl AsStr for &[u8] {
        fn as_str(&self) -> &str {
            std::str::from_utf8(self).unwrap()
        }
    }

    fn doublescaped_char<I, E>(input: I) -> IResult<I, String, E>
    where
        I: Input + Offset + AsStr,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        map(
            preceded(
                char('\\'),
                alt((
                    map(one_of(r#"\""#), |ch| format!(r"\{}", ch)),
                    map(recognize((char('x'), hexdigit, hexdigit)), |i: I| {
                        i.as_str().to_string()
                    }),
                )),
            ),
            |s| format!(r"\\{}", s),
        )
        .parse(input)
    }

    pub fn doublescaped<I, E>(input: I) -> IResult<I, String, E>
    where
        I: Input + Offset + AsStr,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        map(
            delimited(
                char('"'),
                fold_many1(
                    alt((
                        doublescaped_char,
                        map(none_of("\""), |ch| format!("{}", ch)),
                    )),
                    || String::new(),
                    |mut acc, s| {
                        acc += &s;
                        acc
                    },
                ),
                char('"'),
            ),
            |s| format!(r#""\"{}\"""#, s),
        )
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_doublescaped_char() {
            let (_, s) = doublescaped_char::<_, ()>(r#"\""#).unwrap();
            assert_eq!(s, r#"\\\""#);

            let (_, s) = doublescaped_char::<_, ()>(r#"\\"#).unwrap();
            assert_eq!(s, r"\\\\");

            let (_, s) = doublescaped_char::<_, ()>(r#"\x27"#).unwrap();
            assert_eq!(s, r"\\x27");
        }
    }
}

fn escape(s: &str) -> String {
    let (_, escaped) = parser::string::<_, ()>(s).unwrap();
    escaped
}

fn doublescape(s: &str) -> String {
    let (_, doublescaped) = parser::doublescaped::<_, ()>(s).unwrap();
    doublescaped
}

async fn answer(mut parser: impl FnMut(&str) -> String) -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d08.txt").await?;
    let escaped =
        input
            .lines()
            .map(|line| parser(line.trim()))
            .fold(String::new(), |mut acc, line| {
                acc.push_str(&line);
                acc.push('\n');
                acc
            });
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
