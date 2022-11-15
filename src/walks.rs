use std::{str::FromStr, fmt::Display};

use serde::Deserialize;

const COULD_NOT_PARSE: &'static str = "Could not parse";

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum WalkType {
    Simple = 0,
    NoReturns = 1
}

impl Display for WalkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Simple"),
            Self::NoReturns => write!(f, "No Immediate Returns")
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum GridType {
    Square = 0,
    Triangular = 1,
}

impl Display for GridType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Square => write!(f, "Square"),
            Self::Triangular => write!(f, "Triangular"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum SeqType {
    Arithmetic = 0,
    Geometric = 1,
}

impl Display for SeqType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arithmetic => write!(f, "Arithmetic"),
            Self::Geometric => write!(f, "Geometric")
        }
    }
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

#[derive(Deserialize)]
pub struct WalkParams {
    pub seed: i32,
    pub walk_type: WalkType,
    pub grid_type: GridType,
    pub num_walks_coef: f64,
    pub seq_type: SeqType,
    pub start_seq: i32,
    pub arithm_step: i32,
    pub geom_step: f64,
    pub num_steps: i32,
    pub trace_name: String
}

pub struct WalkResult {
    pub num_steps: i32,
    pub num_walks: i64,
    pub mean: f64,
    pub stderr: f64
}

pub struct FitResult {
    pub c0: f64,
    pub c1: f64,
    pub cov00: f64,
    pub cov01: f64,
    pub cov11: f64,
    pub sumsq: f64
}