mod walks;

use std::str::FromStr;
use std::fmt::{Debug, Display};
use plotly::common::{HoverInfo, ErrorData, ErrorType};
use rand::prelude::{thread_rng};
use rand::Rng;
use rgsl::fit::linear_est;
use rgsl::{fit, Value};
use serde::Deserialize;
use walks::{WalkParams, WalkResult, FitResult};
use std::io::{stdin, stdout};
use std::fs;
use plotly::{Plot, Scatter, Layout};
use plotly::layout::{Legend, Axis, AxisType};
use crossterm::{execute, style, cursor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::style::Stylize;
use clap::Parser;

use crate::walks::{SeqType, WalkType, GridType};

include!(concat!(env!("OUT_DIR"), "/walk.rs"));

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config: Option<String>
}
#[derive(Deserialize)]
struct RunConfig {
    output_file: String,
    walks: Vec<WalkParams>
}

fn input<T>(val: &mut T, text: &[&str])
    where 
        T: Debug + FromStr + Display,
        T::Err: Debug,
{
    let mut input = String::new();
    for val in text {
        println!("{}", val);
    }
    println!("Leave empty for value: {}", val);
    stdin().read_line(&mut input).expect("Could not read input");
    if input.trim() == "" {
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

fn rewrite_params(walk_params: &mut WalkParams) {
    input(&mut walk_params.seed, &["Seed:"]);
    input(&mut walk_params.walk_type, &["Walk type:", "0 - Simple walk", "1 - No immediate retuns"]);
    input(&mut walk_params.grid_type, &["Grid type:", "0 - Square grid", "1 - Triangular grid"]);
    input(&mut walk_params.num_walks_coef, &["Number of walks coef (sqrt(walk_length) * coef = total_num_walks):"]);
    input(&mut walk_params.seq_type, &["Sequence type:", "0 - Arithmetic", "1 - Geometric"]);
    input(&mut walk_params.start_seq, &["Initial step count:"]);
    if walk_params.seq_type == SeqType::Arithmetic {
        input(&mut walk_params.arithm_step, &["Step count increase:"]);
    }
    else {
        input(&mut walk_params.geom_step, &["Step count increase quotient:"]);
    }
    input(&mut walk_params.num_steps, &["Number of increase steps:"]);
    println!("Trace name:");
    walk_params.trace_name.clear();
    stdin().read_line(&mut walk_params.trace_name).expect("Could not read input");

}

fn generate_walk(
    walk_params: &WalkParams,
    ctx: &Context
) -> Vec<WalkResult> {

    let bucket_counts = match walk_params.seq_type {
        SeqType::Arithmetic => gen_arithm(walk_params.start_seq, walk_params.arithm_step, walk_params.num_steps),
        SeqType::Geometric => gen_geometric(walk_params.start_seq, walk_params.geom_step, walk_params.num_steps)
    };
    
    let walk_counts = bucket_counts.iter()
    .map(|x| ((*x as f64).sqrt() * walk_params.num_walks_coef).ceil().max(500.0) as i64)
    .collect::<Vec<i64>>();

    println!("");
    println!("Walking....");

    let mut stdout = stdout();

    bucket_counts.iter()
        .zip(&walk_counts)
        .enumerate()
        .map(|(i, (num_steps, wc))| {
                let str = format!("{} / {}", i+1, bucket_counts.len());
                execute!(stdout, Clear(ClearType::CurrentLine), cursor::MoveToColumn(0) ,style::PrintStyledContent(str.as_str().magenta())).expect("Error printing results...");
                // let (mean, stderr) = ctx.one_walk(walk_params.seed, walk_params.walk_type as i32, walk_params.grid_type as i32, *wc, *num_bucket, walk_params.steps_per_sample).unwrap();
                let (mean, stderr) = ctx.walk (walk_params.seed, walk_params.walk_type as i32, walk_params.grid_type as i32, *wc, *num_steps).unwrap();
                WalkResult {
                    num_steps: *num_steps,
                    num_walks: *wc,
                    mean: mean,
                    stderr: stderr
                }
        })
        .collect::<Vec<WalkResult>>()
}

fn add_trace(plot: &mut Plot, trace: &Vec<WalkResult>, walk_params: &WalkParams) {
    
    let xs = trace.iter().map(|x| x.num_steps);
    let means = trace.iter().map(|x| x.mean);
    let stderr = trace.iter().map(|x| (x.stderr * x.num_walks as f64 / (x.num_walks - 1) as f64).sqrt());
    let trace_mean = Scatter::new(xs.collect(), means.collect())
        .name(format!("{} (Walk type: {}, Grid type: {})", walk_params.trace_name, walk_params.walk_type, walk_params.grid_type))
        .show_legend(true)
        .hover_info(HoverInfo::All)
        .hover_text_array(
            trace.iter().map(|x| format!("Number of walks: {}", x.num_walks))
            .collect()
        )
        .error_y(
            ErrorData::new(ErrorType::Data)
            .array(stderr.collect())
            .thickness(0.0)
        );

    let d = trace.iter().map(|x| x.num_steps).chain(trace.iter().map(|x| x.num_steps).rev());
    let s_up = trace.iter().map(|x| x.mean + (x.stderr * x.num_walks as f64 / (x.num_walks - 1) as f64).sqrt());
    let s_down = trace.iter().map(|x| x.mean - (x.stderr * x.num_walks as f64 / (x.num_walks - 1) as f64).sqrt());
    let trace_stderr = Scatter::new(d.collect(), s_up.chain(s_down.rev()).collect())
            .fill(plotly::common::Fill::ToSelf)
            .name(format!("stderr {}", walk_params.trace_name))
            .hover_info(plotly::common::HoverInfo::Skip);
    plot.add_trace(trace_mean);
    plot.add_trace(trace_stderr);
}

fn fit_log_data(trace: &Vec<WalkResult>, trace_name: &str) -> Result<FitResult, String> {
    let x = trace.iter().map(|x| x.num_steps as f64).map(|x| x.ln()).collect::<Vec<f64>>();
    let y = trace.iter().map(|x| x.mean).map(|x| x.ln()).collect::<Vec<f64>>();
    let f = fit::linear(&x[..], 1, &y[..], 1, trace.len());
    match f.0 {
        Value::Success => Ok(FitResult {
            c0: f.1,
            c1: f.2,
            cov00: f.3,
            cov01: f.4,
            cov11: f.5,
            sumsq: f.6,
        }),
        _ => Err(format!("Failed fit for {}", trace_name))
    }  
}

fn add_log_fit_trace(plot: &mut Plot, fit: &FitResult, trace: &Vec<WalkResult>, trace_name: &str) {
    let xs = trace.iter()
        .map(|x| x.num_steps);
    let fit = trace.iter()
        .map(|x| linear_est((x.num_steps as f64).ln(), fit.c0, fit.c1, fit.cov00, fit.cov01, fit.cov11))
        .map(|(_, y, err)| (y.exp(), err))
        .collect::<Vec<(f64, f64)>>();

    let ys = fit.iter().map(
        |x| x.0
    ).collect::<Vec<f64>>();

    let errs = fit.iter().map(
        |x| x.1
    ).collect::<Vec::<f64>>();

    let trace_mean = Scatter::new(xs.collect(), ys)
        .show_legend(true)
        .name(format!("Fit - {}", trace_name))
        .error_y(
            ErrorData::new(ErrorType::Data)
            .array(errs)
        );

    plot.add_trace(trace_mean);
}

fn manual(mut plot: &mut Plot, mut plot_loglog: &mut Plot, mut input_buff: &mut String, mut graph_file: &mut String) {
    let mut rng = thread_rng();

    let mut walk_params = WalkParams {
        seed: rng.gen(),
        walk_type: WalkType::Simple,
        grid_type: GridType::Square,
        num_walks_coef: 20.0,
        seq_type: SeqType::Arithmetic,
        start_seq: 20,
        arithm_step: 5,
        geom_step: 1.1,
        num_steps: 100,
        trace_name: String::with_capacity(40),
    };

    let ctx = Context::new().unwrap();

    loop {
        // Modify params
        rewrite_params(&mut walk_params);
        // Generate walk
        let results = generate_walk(&walk_params, &ctx);
        // Add to trace
        add_trace(&mut plot, &results, &walk_params);
        add_trace(&mut plot_loglog, &results, &walk_params);

        let fit = fit_log_data(&results, &walk_params.trace_name);

        match fit {
            Ok(f) => {
                println!("");
                println!("Fit succesful");
                println!("sumsqr: {}", f.sumsq);
                println!("c0: {}", f.c0);
                println!("c1: {}", f.c1);

                add_log_fit_trace(&mut plot, &f, &results, &walk_params.trace_name);
                add_log_fit_trace(&mut plot_loglog, &f, &results, &walk_params.trace_name);
            },
            Err(s) => print!("{}", s)
        };

        println!("");
        println!("Do you want to generate another trace? (y / n)");
        input_buff.clear();
        stdin().read_line(&mut input_buff).expect("Could not read input");

        if input_buff.trim().to_lowercase() == "n" {
            break;
        }
    }
    println!("");
    println!("Name of the output file:");
    stdin().read_line(&mut graph_file).expect("Could not read input");
}

fn auto(configs: RunConfig, mut plot: &mut Plot, mut plot_loglog: &mut Plot) {
    let ctx = Context::new().unwrap();
    for walk_params in configs.walks {
        // Generate walk
        let results = generate_walk(&walk_params, &ctx);
        // Add to trace
        add_trace(&mut plot, &results, &walk_params);
        add_trace(&mut plot_loglog, &results, &walk_params);

        let fit = fit_log_data(&results, &walk_params.trace_name);

        match fit {
            Ok(f) => {
                add_log_fit_trace(&mut plot, &f, &results,
                     &format!("{} (c = {:.4}, alpha = {:.4})", walk_params.trace_name, f.c0.exp(), f.c1));
                add_log_fit_trace(&mut plot_loglog, &f, &results, 
                    &format!("{} (c = {:.4}, alpha = {:.4})", walk_params.trace_name, f.c0.exp(), f.c1));
            },
            Err(s) => print!("{}", s)
        };
    }
}

fn main() {
    let mut plot = Plot::new();
    let layout = Layout::new()
        .legend(Legend::new())
        .show_legend(true);
    plot.set_layout(layout);

    let mut plot_loglog = Plot::new();
    let layout2 = Layout::new()
        .legend(Legend::new())
        .show_legend(true)
        .x_axis(Axis::new().type_(AxisType::Log))
        .y_axis(Axis::new().type_(AxisType::Log));
    plot_loglog.set_layout(layout2);

    let mut input_buff = String::with_capacity(10);

    let args: Args = Args::parse();
    
    let run_configs = args.config.and_then(
        |x| {
            let data = fs::read_to_string(x)
                .expect("Could not find config file.");
            let data: RunConfig = serde_json::from_str(&data).unwrap();
            Some(data)
        }
    );

    let mut graph_file = String::with_capacity(40);

    match run_configs {
        None => manual(&mut plot, &mut plot_loglog, &mut input_buff, &mut graph_file),
        Some(config) => {
            graph_file.insert_str(0, &config.output_file);
            auto(config, &mut plot, &mut plot_loglog);
        }
    };
    
    fs::create_dir_all("plot").expect("Could not create directory for results.");
    plot.write_html(format!("plot/{}.html", graph_file.as_str().trim()));
    plot_loglog.write_html(format!("plot/{}_loglog.html", graph_file.as_str().trim()));
}
