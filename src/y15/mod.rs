use std::{
    iter::Step,
    ops::{Add, Mul},
};

use nom::{AsChar, Input, Parser, character::complete::space0, sequence::delimited};

#[derive(Default, Debug, Hash, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct Vec2<T> {
    x: T,
    y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vec2<T>
where
    T: Mul<T> + Copy,
{
    pub fn area(&self) -> T::Output {
        self.x * self.y
    }
}

impl<T> Vec2<T>
where
    T: Add<T> + Copy,
    <T as Add<T>>::Output: Mul<i32>,
{
    pub fn perimeter(&self) -> <<T as Add<T>>::Output as Mul<i32>>::Output {
        (self.x + self.y) * 2
    }
}

impl<T> Vec2<T>
where
    T: Step + Copy,
{
    pub fn points_to_inclusive(self, rhs: Vec2<T>) -> PointRangeInclusive<T> {
        PointRangeInclusive {
            start: self,
            end: rhs,
            curr: None,
        }
    }
}

/// An inclusive range of points in a 2D grid.
#[derive(Debug, Clone, Copy)]
pub struct PointRangeInclusive<T>
where
    T: Step + Copy,
{
    start: Vec2<T>,
    end: Vec2<T>,
    curr: Option<Vec2<T>>,
}

impl<T> Iterator for PointRangeInclusive<T>
where
    T: Step + Copy,
{
    type Item = Vec2<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(curr) = &mut self.curr {
            if curr.x < self.end.x {
                curr.x = T::forward_checked(curr.x, 1)?;
                return Some(*curr);
            }

            curr.x = self.start.x;
            if curr.y < self.end.y {
                curr.y = T::forward_checked(curr.y, 1)?;
                return Some(*curr);
            }

            None
        } else {
            self.curr = Some(self.start);
            self.curr
        }
    }
}

pub fn ws<I, P>(parser: P) -> impl Parser<I, Output = P::Output, Error = P::Error>
where
    I: Input,
    I::Item: AsChar,
    P: Parser<I>,
{
    delimited(space0, parser, space0)
}

macro_rules! define_year {
    ($($day_num:ident),+) => {
        $( mod $day_num; )*

        #[derive(clap::Subcommand)]
        enum DaySubcommand {
            P1,
            P2,
        }

        #[derive(clap::Args)]
        struct DayArgs {
            #[command(subcommand)]
            command: DaySubcommand,
        }

        #[derive(clap::Subcommand)]
        #[allow(non_camel_case_types)]
        enum Subcommand {
            $(
                $day_num(DayArgs),
            )*
        }

        #[derive(clap::Args)]
        pub struct Args {
            #[command(subcommand)]
            command: Subcommand,
        }

        impl Args {
            pub async fn run(&self) -> anyhow::Result<()> {
                match &self.command {
                    $(
                        Subcommand::$day_num(args) => {
                            match args.command {
                                DaySubcommand::P1 => $day_num::p1().await,
                                DaySubcommand::P2 => $day_num::p2().await,
                            }
                        }
                    )*
                }
            }
        }
    };
}

define_year!(
    d01, d02, d03, d04, d05, d06, d07, d08, d09, d10, d11, d12, d13
);

#[cfg(test)]
mod tests {
    mod point_range_inclusive {
        use crate::y15::Vec2;

        #[test]
        fn test_single_point() {
            let mut range = Vec2::new(0, 0).points_to_inclusive(Vec2::new(0, 0));
            assert_eq!(range.next(), Some(Vec2::new(0, 0)));
            assert_eq!(range.next(), None);
        }

        #[test]
        fn test_horizontal_line_points() {
            let mut range = Vec2::new(0, 0).points_to_inclusive(Vec2::new(3, 0));
            assert_eq!(range.next(), Some(Vec2::new(0, 0)));
            assert_eq!(range.next(), Some(Vec2::new(1, 0)));
            assert_eq!(range.next(), Some(Vec2::new(2, 0)));
            assert_eq!(range.next(), Some(Vec2::new(3, 0)));
            assert_eq!(range.next(), None);
        }

        #[test]
        fn test_vertical_line_points() {
            let mut range = Vec2::new(0, 0).points_to_inclusive(Vec2::new(0, 3));
            assert_eq!(range.next(), Some(Vec2::new(0, 0)));
            assert_eq!(range.next(), Some(Vec2::new(0, 1)));
            assert_eq!(range.next(), Some(Vec2::new(0, 2)));
            assert_eq!(range.next(), Some(Vec2::new(0, 3)));
            assert_eq!(range.next(), None);
        }

        #[test]
        fn test_rect_points() {
            let mut range = Vec2::new(0, 0).points_to_inclusive(Vec2::new(1, 1));
            assert_eq!(range.next(), Some(Vec2::new(0, 0)));
            assert_eq!(range.next(), Some(Vec2::new(1, 0)));
            assert_eq!(range.next(), Some(Vec2::new(0, 1)));
            assert_eq!(range.next(), Some(Vec2::new(1, 1)));
            assert_eq!(range.next(), None);
        }
    }
}
