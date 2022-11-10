use std::str::FromStr;
use std::fmt::Debug;
use rand::prelude::{thread_rng};
use rand::Rng;
use std::io::{Write, Result as R, stdin};

include!(concat!(env!("OUT_DIR"), "/walk.rs"));

#[derive(Debug, Clone, Copy)]
enum WalkType {
    Simple = 0,
    NoReturns = 1
}

#[derive(Debug, Clone, Copy)]
enum GridType {
    Square = 0,
    Triangular = 1,
    Hexagonal = 2
}

impl FromStr for WalkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(WalkType::Simple),
            "1" => Ok(WalkType::NoReturns),
            _ => Err("Could not parse.")
        }
    }
}

impl FromStr for GridType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(GridType::Square),
            "1" => Ok(GridType::Triangular),
            "2" => Ok(GridType::Hexagonal),
            _ => Err("Could not parse.")
        }
    }
}

fn input<T>(val: &mut T, text: &[&str])
    where 
        T: Debug + FromStr,
        T::Err: Debug,
{
    let mut input = String::new();
    for val in text {
        println!("{}", val);
    }
    stdin().read_line(&mut input).expect("Could not read input.");
    *val = input.trim().parse::<T>().expect("Could not parse.");
}

fn get_smallest_divisor_larger_than_d(n: i32, d: i32, max_iter: i32) -> i32 {
    for i in d..(d + max_iter) {
        if n % i == 0 {
            return i;
        }
    }
    return 1;
}

fn get_bucket_size(n: i32) -> (i32, i32) {
    if n < 2000 {
      return (1, n);
    }
    let search = (n as f64).sqrt() as i32;
    let x = get_smallest_divisor_larger_than_d(n, search - 2, n/2);
      (x, n/x)
  }

fn main() -> R<()> {
    let mut rng = thread_rng();

    let mut seed: i32 = rng.gen();
    let mut walk_type: WalkType = WalkType::Simple;
    let mut grid_type: GridType = GridType::Square;
    let mut num_walks: i64 = 10000;
    let mut min_steps: i32 = 500;
    let mut step_mult: f64 = 1.3;
    let mut num_samples: i32 = 1000;

    input(&mut seed, &["Seed (leave empty to generate random - TBD)"]);
    input(&mut walk_type, &["Walk type:", "0 - Simple walk", "1 - No immediate retuns"]);
    input(&mut grid_type, &["Grid type:", "0 - Square grid", "1 - Triangular grid", "2 - Hexagonal grid"]);
    input(&mut num_walks, &["Num walks:"]);
    input(&mut min_steps, &["Start steps:"]);
    input(&mut step_mult, &["Step multiplier:"]);
    input(&mut num_samples, &["Number of samples:"]);

    // Generate bucket size for each value
    let buckets = (0..num_samples) // Number of steps
    .map(|x| (
        min_steps as f64 * step_mult.powf((x).into())) as i32)
    .map(|n| get_bucket_size(n)).collect::<Vec<(i32, i32)>>();

    //TODO: Use crossterm

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let ctx = Context::new().unwrap();
    for (i, bucket) in buckets.iter().enumerate() {
        let data = ctx.one_walk(seed, walk_type as i32, grid_type as i32, num_walks, bucket.0, bucket.1).unwrap();
        writeln!(lock, "{} ({}*{}), {}, {}",
            bucket.0 * bucket.1, bucket.0, bucket.1, data.0, data.1)?;
    }
    Ok(())
}