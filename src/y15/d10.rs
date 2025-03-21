use std::fmt::Write;

use anyhow::Result;

const INPUT: &str = "1113122113";

#[derive(Default, Debug)]
struct Acc {
    result: String,
    current: char,
    count: usize,
}

impl Acc {
    fn push(&mut self, ch: char) {
        if self.current == ch {
            self.count += 1;
        } else {
            if self.current != '\0' {
                write!(self.result, "{}{}", self.count, self.current).unwrap();
            }
            self.current = ch;
            self.count = 1;
        }
    }

    fn finish(mut self) -> String {
        if self.current != '\0' {
            write!(self.result, "{}{}", self.count, self.current).unwrap();
        }
        self.result
    }
}

fn look_and_say(n: usize, input: &str) -> String {
    let mut result = input.to_string();
    for _ in 0..n {
        let acc = result.chars().fold(Acc::default(), |mut acc, ch| {
            acc.push(ch);
            acc
        });
        result = acc.finish();
    }
    result
}

pub async fn p1() -> Result<()> {
    let answer = look_and_say(40, INPUT);
    println!("Answer: {}", answer.len());
    Ok(())
}

pub async fn p2() -> Result<()> {
    let answer = look_and_say(50, INPUT);
    println!("Answer: {}", answer.len());
    Ok(())
}
