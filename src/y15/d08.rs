use anyhow::Result;

mod parser {
    use nom::{
        AsChar, Compare, IResult, Input, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, none_of, one_of, satisfy},
        combinator::map,
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
        I: Input + Compare<&'static str>,
        <I as Input>::Item: AsChar,
        E: ParseError<I>,
    {
        delimited(
            tag("\""),
            fold_many1(
                alt((escaped_char, none_of("\""))),
                || String::new(),
                |mut acc, ch| {
                    acc.push(ch);
                    acc
                },
            ),
            tag("\""),
        )
        .parse(input)
    }
}

fn escape(s: &str) -> String {
    let (_, escaped) = parser::string::<_, ()>(s).unwrap();
    escaped
}

pub async fn p1() -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d08.txt").await?;
    let escaped =
        input
            .lines()
            .map(|line| escape(line.trim()))
            .fold(String::new(), |mut acc, line| {
                acc.push_str(&line);
                acc.push('\n');
                acc
            });
    println!("Answer: {}", input.len() - escaped.len());
    Ok(())
}

pub async fn p2() -> Result<()> {
    todo!()
}
