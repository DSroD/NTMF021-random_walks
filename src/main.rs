use std::str::FromStr;
use std::fmt::Debug;
use rand::prelude::{thread_rng};
use rand::Rng;
use std::io::{stdin, stdout};
use plotly::{Plot, Scatter, Layout};
use plotly::layout::{Legend};
use crossterm::{execute, style, cursor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::style::Stylize;

include!(concat!(env!("OUT_DIR"), "/walk.rs"));

const COULD_NOT_PARSE: &'static str = "Could not parse";

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

#[derive(Debug, Clone, Copy)]
enum SeqType {
    Arithmetic = 0,
    Geometric = 1,
}

#[derive(Debug, Clone, Copy)]
enum SeqLength {
    BabyLength = 0,
    NormalLength = 1,
    ChadLength = 2,
    MetacentrumSigmaGrindsetLength = 3,
}

impl FromStr for WalkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Simple),
            "1" => Ok(Self::NoReturns),
            _ => Err(COULD_NOT_PARSE)
        }
    }
}

impl FromStr for GridType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Square),
            "1" => Ok(Self::Triangular),
            "2" => Ok(Self::Hexagonal),
            _ => Err(COULD_NOT_PARSE)
        }
    }
}

impl FromStr for SeqType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Arithmetic),
            "1" => Ok(Self::Geometric),
            _ => Err(COULD_NOT_PARSE)
        }
    }
}

impl FromStr for SeqLength {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::BabyLength),
            "1" => Ok(Self::NormalLength),
            "2" => Ok(Self::ChadLength),
            "3" => Ok(Self::MetacentrumSigmaGrindsetLength),
            _ => Err(COULD_NOT_PARSE)
        }
    }
}

fn input<T>(val: &mut T, text: &[&str], empty_allowed: bool)
    where 
        T: Debug + FromStr,
        T::Err: Debug,
{
    let mut input = String::new();
    for val in text {
        println!("{}", val);
    }
    stdin().read_line(&mut input).expect("Could not read input");
    if input.trim() == "" && empty_allowed {
        return;
    }
    *val = input.trim().parse::<T>().expect("Could not parse input.");
}

fn gen_arithm(init: i32, step: i32, num: i32) -> Vec<i32> {
    (0..num).map(|x| init + x * step).collect::<Vec<i32>>()
}

fn gen_geometric(a0: i32, q: f64, num: i32) -> Vec<i32> {
    (0..num).map(|x| ((a0 as f64) * q.powi(x)).ceil() as i32).collect::<Vec<i32>>()
}

fn gen_lengths(init_length: i32, n_arithm: i32, arithm_step: i32, n_geom: i32, geom_step: f64, seq_type: SeqType) -> Vec<i32> {
    match seq_type {
        SeqType::Arithmetic => gen_arithm(init_length, arithm_step, n_arithm),
        SeqType::Geometric => gen_geometric(init_length, geom_step, n_geom)
    }
}

fn generate_seq(seq_type: SeqType, seq_length: SeqLength) -> Vec<i32> {
    match seq_length {
        SeqLength::BabyLength => gen_lengths(2, 100, 5, 50, 1.06, seq_type),
        SeqLength::NormalLength => gen_lengths(5, 200, 10, 55, 1.05, seq_type),
        SeqLength::ChadLength => gen_lengths(100, 500, 20, 60, 1.04, seq_type),
        SeqLength::MetacentrumSigmaGrindsetLength => gen_lengths(500, 1000, 50, 70, 1.03, seq_type)
    }
}

fn main() {
    let mut rng = thread_rng();

    let mut seed: i32 = rng.gen();
    let mut walk_type: WalkType = WalkType::Simple;
    let mut grid_type: GridType = GridType::Square;
    let mut num_walks: i64 = 10000;
    let mut num_samples: i32 = 1000;
    let mut seq_type: SeqType = SeqType::Arithmetic;
    let mut seq_length: SeqLength = SeqLength::NormalLength;
    let mut graph_file = String::with_capacity(40);

    input(&mut seed, &["Seed (leave empty to generate random)"], true);
    input(&mut walk_type, &["Walk type:", "0 - Simple walk", "1 - No immediate retuns"], false);
    input(&mut grid_type, &["Grid type:", "0 - Square grid", "1 - Triangular grid", "2 - Hexagonal grid"], false);
    input(&mut num_walks, &["Number of walks:"], false);
    input(&mut num_samples, &["Bucket-Walk length:"], false);
    input(&mut seq_type, &["Sequence type:", "0 - Arithmetic", "1 - Geometric"], false);
    input(&mut seq_length, &["Sequence length:",
     "0 - Baby length (xyz)",
      "1 - Normal length (xyz)",
       "2 - Chad length (xyz)",
        "3 - Metacentrum sigma grindset length (xyz)"], false);

    println!("Name of the output:");
    stdin().read_line(&mut graph_file).expect("Could not read input");

    // Generate bucket size for each value
    let buckets = generate_seq(seq_type, seq_length)
        .iter()
        .map(|x| (*x, num_samples))
        .collect::<Vec<(i32, i32)>>();

    println!("Walking....");

    let mut stdout = stdout();
    let ctx = Context::new().unwrap();

    let stats = buckets.iter()
        .enumerate()
        .map(|(i, bucket)| {
                let str = format!("{} / {}", i, buckets.len());
                execute!(stdout, Clear(ClearType::CurrentLine), cursor::MoveToColumn(0) ,style::PrintStyledContent(str.as_str().magenta())).expect("Error printing results...");
                ctx.one_walk(seed, walk_type as i32, grid_type as i32, num_walks, bucket.0, bucket.1).unwrap()
        })
        .collect::<Vec<(f64, f64)>>();
    
    println!("");
    println!("Exporting graph....");

    let mut plot = Plot::new();

    let xs = buckets.iter().map(|x| x.0 * x.1).collect::<Vec<i32>>();
    let means = stats.iter().map(|x| x.0).collect::<Vec<f64>>();
    let trace_mean = Scatter::new(xs, means)
        .name(format!("{} walks (walk type: {:?}, grid type: {:?})", num_walks, walk_type, grid_type))
        .show_legend(true);

    let layout = Layout::new()
        .legend(Legend::new())
        .show_legend(true);
    
    plot.set_layout(layout);
    plot.add_trace(trace_mean);
    plot.write_html(format!("plot/{}.html", graph_file));
}
