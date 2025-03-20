use std::str::FromStr;

use anyhow::Context;
use futures::TryStreamExt;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tokio_stream::wrappers::LinesStream;

use super::Vec2;

struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl FromStr for Vec3 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut s = s.split("x");
        Ok(Vec3 {
            x: s.next().context("Dimension must have 3 parts")?.parse()?,
            y: s.next().context("Dimension must have 3 parts")?.parse()?,
            z: s.next().context("Dimension must have 3 parts")?.parse()?,
        })
    }
}

impl Vec3 {
    fn surface_area(&self) -> i32 {
        let xy = self.x * self.y;
        let yz = self.y * self.z;
        let xz = self.x * self.z;
        2 * (xy + yz + xz)
    }

    fn volume(&self) -> i32 {
        self.x * self.y * self.z
    }

    fn sides(&self) -> [Vec2; 3] {
        [
            Vec2 {
                x: self.x,
                y: self.y,
            },
            Vec2 {
                x: self.y,
                y: self.z,
            },
            Vec2 {
                x: self.x,
                y: self.z,
            },
        ]
    }
}

async fn answer(f: impl Fn(i32, Vec3) -> i32) -> anyhow::Result<()> {
    let input = File::open("inputs/y15_d02.txt").await?;
    let input = BufReader::new(input);
    let answer = LinesStream::new(input.lines())
        .map_err(anyhow::Error::from)
        .try_fold(0, {
            async |acc, line| {
                let dims = Vec3::from_str(&line)?;
                Ok(f(acc, dims))
            }
        })
        .await?;
    println!("Answer: {}", answer);
    Ok(())
}

pub async fn p1() -> anyhow::Result<()> {
    answer(|acc, dims| {
        acc + dims.surface_area() + dims.sides().map(|side| side.area()).iter().min().unwrap()
    })
    .await
}

pub async fn p2() -> anyhow::Result<()> {
    answer(|acc, dims| {
        acc + dims.volume()
            + dims
                .sides()
                .map(|side| side.perimeter())
                .iter()
                .min()
                .unwrap()
    })
    .await
}
