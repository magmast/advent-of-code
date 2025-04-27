use std::{collections::HashMap, fmt::Display};

use anyhow::{Error, anyhow};
use winnow::Parser;

mod parser {
    use winnow::{
        ModalResult, Parser,
        ascii::dec_uint,
        combinator::{alt, cut_err, fail, preceded, separated_pair},
        error::{StrContext, StrContextValue},
        token::take_while,
    };

    use crate::y15::ws;

    use super::{Connection, ConnectionSource, Identifier, Operation, Value};

    fn identifier<'a>(input: &mut &'a str) -> ModalResult<Identifier<'a>> {
        take_while(1.., |ch: char| ch.is_ascii_lowercase()).parse_next(input)
    }

    fn value<'a>(input: &mut &'a str) -> ModalResult<Value<'a>> {
        alt((
            identifier.map(Value::Identifier),
            dec_uint.map(Value::Literal),
            fail.context(StrContext::Label("value"))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "identifier",
                )))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "unsigned integer",
                ))),
        ))
        .parse_next(input)
    }

    fn operation<'a>(input: &mut &'a str) -> ModalResult<Operation<'a>> {
        alt((
            preceded(ws("NOT"), cut_err(value)).map(Operation::Not),
            separated_pair(value, ws("AND"), value).map(|(lhs, rhs)| Operation::And(lhs, rhs)),
            separated_pair(value, ws("OR"), value).map(|(lhs, rhs)| Operation::Or(lhs, rhs)),
            separated_pair(value, ws("LSHIFT"), dec_uint)
                .map(|(lhs, rhs)| Operation::LShift(lhs, rhs)),
            separated_pair(value, ws("RSHIFT"), dec_uint)
                .map(|(lhs, rhs)| Operation::RShift(lhs, rhs)),
            fail.context(StrContext::Label("operation"))
                .context(StrContext::Expected(StrContextValue::StringLiteral("not")))
                .context(StrContext::Expected(StrContextValue::StringLiteral("or")))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "lshift",
                )))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "rshift",
                ))),
        ))
        .parse_next(input)
    }

    fn connection_source<'a>(input: &mut &'a str) -> ModalResult<ConnectionSource<'a>> {
        alt((
            operation.map(ConnectionSource::Operation),
            identifier.map(ConnectionSource::Identifier),
            dec_uint.map(ConnectionSource::Literal),
            fail.context(StrContext::Label("connection source"))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "operation",
                )))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "identifier",
                )))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "unsigned integer",
                ))),
        ))
        .parse_next(input)
    }

    pub fn connection<'a>(input: &mut &'a str) -> ModalResult<Connection<'a>> {
        separated_pair(connection_source, ws("->"), identifier)
            .map(|(from, to)| Connection { from, to })
            .context(StrContext::Label("connection"))
            .parse_next(input)
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

impl<'a> TryFrom<&'a str> for Connection<'a> {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        parser::connection
            .parse(s)
            .map_err(|err| anyhow!("\n{err}"))
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
    let state =
        input
            .lines()
            .map(Connection::try_from)
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
