use anyhow::Context;

const INITIAL_FLOOR: i32 = 0;

const UP_CHAR: char = '(';

const DOWN_CHAR: char = ')';

pub fn traverse_apartment(directions: &str) -> impl Iterator<Item = i32> {
    directions.trim().chars().scan(INITIAL_FLOOR, |acc, ch| {
        match ch {
            UP_CHAR => *acc += 1,
            DOWN_CHAR => *acc -= 1,
            _ => panic!("Invalid character in input: {:?}", ch),
        };

        Some(*acc)
    })
}

#[derive(clap::Subcommand)]
enum Subcommand {
    P1,
    P2,
}

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

impl Args {
    pub async fn run(&self) -> anyhow::Result<()> {
        let input = tokio::fs::read_to_string("inputs/y15_d01.txt").await?;

        match self.command {
            Subcommand::P1 => {
                let answer = traverse_apartment(&input)
                    .last()
                    .context("Directions are empty")?;
                println!("Answer: {}", answer);
            }
            Subcommand::P2 => {
                let answer = traverse_apartment(&input)
                    .enumerate()
                    .find(|(_, floor)| *floor == -1)
                    .map(|(i, _)| i)
                    .context("Basement never entered");
                println!("Answer: {}", answer.unwrap() + 1);
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::traverse_apartment;

    #[test]
    fn test_traverse_apartment() {
        assert_eq!(traverse_apartment("(())").last(), Some(0));
        assert_eq!(traverse_apartment("(((").last(), Some(3));
        assert_eq!(traverse_apartment("(()(()(").last(), Some(3));
        assert_eq!(traverse_apartment("))(((((").last(), Some(3));
        assert_eq!(traverse_apartment("())").last(), Some(-1));
        assert_eq!(traverse_apartment("))(").last(), Some(-1));
        assert_eq!(traverse_apartment(")))").last(), Some(-3));
        assert_eq!(traverse_apartment(")())())").last(), Some(-3));
    }
}
