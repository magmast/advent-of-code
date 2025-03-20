use std::{collections::HashMap, fmt::Display, time::Instant};

mod parser {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::{tag, take_while1},
        character::complete::{digit1, space0},
        combinator::map,
        error::{ContextError, ParseError, context},
        sequence::{delimited, preceded, separated_pair},
    };

    use super::{Connection, ConnectionSource, Identifier, Operation, Value};

    fn ws<'a, O, E>(
        parser: impl Parser<&'a str, Output = O, Error = E>,
    ) -> impl Parser<&'a str, Output = O, Error = E>
    where
        E: ParseError<&'a str>,
    {
        delimited(space0, parser, space0)
    }

    fn identifier<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, Identifier<'a>, E> {
        context("identifier", take_while1(|ch| ('a'..='z').contains(&ch))).parse(input)
    }

    fn u32<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, u32, E> {
        context("u32", map(digit1, |digits: &str| digits.parse().unwrap())).parse(input)
    }

    fn value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, Value<'a>, E> {
        alt((map(identifier, Value::Identifier), map(u32, Value::Literal))).parse(input)
    }

    fn operation<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, Operation<'a>, E> {
        context(
            "operation",
            alt((
                map(preceded(ws(tag("NOT")), value), Operation::Not),
                map(
                    separated_pair(value, ws(tag("AND")), value),
                    |(lhs, rhs)| Operation::And(lhs, rhs),
                ),
                map(separated_pair(value, ws(tag("OR")), value), |(lhs, rhs)| {
                    Operation::Or(lhs, rhs)
                }),
                map(
                    separated_pair(value, ws(tag("LSHIFT")), u32::<E>),
                    |(lhs, rhs)| Operation::LShift(lhs, rhs),
                ),
                map(
                    separated_pair(value, ws(tag("RSHIFT")), u32::<E>),
                    |(lhs, rhs)| Operation::RShift(lhs, rhs),
                ),
            )),
        )
        .parse(input)
    }

    fn connection_source<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, ConnectionSource<'a>, E> {
        context(
            "connection_source",
            alt((
                map(operation, ConnectionSource::Operation),
                map(identifier, ConnectionSource::Identifier),
                map(u32, ConnectionSource::Literal),
            )),
        )
        .parse(input)
    }

    pub fn connection<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, Connection<'a>, E> {
        context(
            "connection",
            map(
                separated_pair(connection_source, ws(tag("->")), identifier),
                |(from, to)| Connection { from, to },
            ),
        )
        .parse(input)
    }

    #[cfg(test)]
    mod tests {
        use nom::{Parser, bytes::complete::tag};

        use super::ws;

        #[test]
        fn test_ws() {
            assert_eq!(ws(tag::<_, _, ()>("NOT")).parse("NOT"), Ok(("", "NOT")));
            assert_eq!(ws(tag::<_, _, ()>("NOT")).parse("  NOT"), Ok(("", "NOT")));
            assert_eq!(ws(tag::<_, _, ()>("NOT")).parse("NOT  "), Ok(("", "NOT")));
        }

        #[test]
        fn test_identifier() {
            let input = "abc";
            let result = super::identifier::<()>(input);
            assert_eq!(result, Ok(("", "abc")));
        }
    }
}

type Identifier<'a> = &'a str;

#[derive(Debug, Clone)]
enum Value<'a> {
    Literal(u32),
    Identifier(Identifier<'a>),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(lit) => write!(f, "{}", lit),
            Self::Identifier(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone)]
enum Operation<'a> {
    Not(Value<'a>),
    And(Value<'a>, Value<'a>),
    Or(Value<'a>, Value<'a>),
    LShift(Value<'a>, u32),
    RShift(Value<'a>, u32),
}

impl Display for Operation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Not(value) => write!(f, "NOT {}", value),
            Self::And(lhs, rhs) => write!(f, "{} AND {}", lhs, rhs),
            Self::Or(lhs, rhs) => write!(f, "{} OR {}", lhs, rhs),
            Self::LShift(value, shift) => write!(f, "{} LSHIFT {}", value, shift),
            Self::RShift(value, shift) => write!(f, "{} RSHIFT {}", value, shift),
        }
    }
}

#[derive(Debug)]
enum ConnectionSource<'a> {
    Literal(u32),
    Identifier(Identifier<'a>),
    Operation(Operation<'a>),
}

#[derive(Debug)]
struct Connection<'a> {
    from: ConnectionSource<'a>,
    to: Identifier<'a>,
}

impl<'a> Connection<'a> {
    fn literal(to: Identifier<'a>, value: u32) -> Self {
        Self {
            from: ConnectionSource::Literal(value),
            to,
        }
    }
}

#[derive(Default)]
struct State<'a> {
    signals: HashMap<Identifier<'a>, ConnectionSource<'a>>,
}

impl<'a> State<'a> {
    pub fn set_connection(&mut self, conn: Connection<'a>) {
        self.signals.insert(conn.to, conn.from);
    }
}

struct Evaluator<'a, 'b> {
    state: &'b State<'a>,
    cache: HashMap<Identifier<'a>, Option<u32>>,
}

impl<'a, 'b> Evaluator<'a, 'b> {
    pub fn new(state: &'b State<'a>) -> Self {
        Self {
            state,
            cache: HashMap::new(),
        }
    }

    pub fn eval(&mut self, id: Identifier<'a>) -> Option<u32> {
        if let Some(value) = self.cache.get(id) {
            return *value;
        }

        let source = self.state.signals.get(id)?;
        let result = match source {
            ConnectionSource::Literal(id) => Some(*id),
            ConnectionSource::Identifier(id) => self.eval(id),
            ConnectionSource::Operation(op) => self.eval_op(op),
        };

        self.cache.insert(id, result);
        result
    }

    fn eval_op(&mut self, op: &Operation<'a>) -> Option<u32> {
        match op {
            Operation::Not(value) => self.eval_value(value).map(|v| !v),
            Operation::And(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;
                Some(lhs & rhs)
            }
            Operation::Or(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;
                Some(lhs | rhs)
            }
            Operation::LShift(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                Some(lhs << rhs)
            }
            Operation::RShift(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                Some(lhs >> rhs)
            }
        }
    }

    fn eval_value(&mut self, value: &Value<'a>) -> Option<u32> {
        match value {
            Value::Literal(lit) => Some(*lit),
            Value::Identifier(id) => self.eval(id),
        }
    }
}

async fn state_from_str(input: &str) -> anyhow::Result<State> {
    let state = input
        .lines()
        .map(|line| {
            parser::connection::<()>(&line)
                .map(|(_, conn)| conn)
                .map_err(|err| err.to_owned())
                .map_err(anyhow::Error::from)
        })
        .try_fold(State::default(), |mut state, conn| {
            state.set_connection(conn?);
            Ok::<_, anyhow::Error>(state)
        })?;
    Ok(state)
}

pub async fn p1() -> anyhow::Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d07.txt").await?;
    let state = state_from_str(&input).await?;
    let value = Evaluator::new(&state).eval("a").unwrap();
    println!("Answer: {}", value);
    Ok(())
}

pub async fn p2() -> anyhow::Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d07.txt").await?;
    let mut state = state_from_str(&input).await?;
    let a_value = Evaluator::new(&state).eval("a").unwrap();
    state.set_connection(Connection::literal("b", a_value));
    let a_value = Evaluator::new(&state).eval("a").unwrap();
    println!("Answer: {}", a_value);
    Ok(())
}
