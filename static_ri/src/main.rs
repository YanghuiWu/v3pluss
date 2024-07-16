#![allow(internal_features)]
#![feature(core_intrinsics)]

use std::collections::BTreeSet;
use std::intrinsics::ceilf32;
use std::ops::Range;
use std::{collections::HashMap, time::Instant};

use dace_tests::polybench_simplify;
use rand::prelude::Distribution;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use dace::ast::{LoopBound, LoopStmt, Node, Stmt};
use static_ri::tracing_ri;

mod test;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
        .init();
    let mut wrapping_loop = Vec::new();
    let mut trace = polybench_simplify::gemm(128);
    let mut ref_counter = 0;
    let start = Instant::now();
    let _hist = tracing_ri(&mut trace, 8, 64);
    let mut ans = HashMap::new();
    sample_collect(&trace, &mut wrapping_loop, &mut ans, &mut ref_counter);
    let _samples = sample_gen(&mut ans, 0.1);

    let end = Instant::now();
    println!("gemm trace time: {:?}", end - start);
    // println!("collected: {ans:#?}");
    //println!("hist: {}", hist);
}

pub fn sample_collect<'a>(
    code_tree: &'a Node,
    wrapping_loops: &mut Vec<&'a LoopStmt>,
    ans: &mut HashMap<usize, Vec<(&'a str, Range<usize>)>>,
    // access_name, (loop_name + sample_times)* + -
    ref_counter: &mut usize,
) {
    // let init = (HashMap::<String, usize>::new(), 0);
    match &code_tree.stmt {
        Stmt::Loop(stmt) => {
            wrapping_loops.push(stmt);
            for i in stmt.body.iter() {
                sample_collect(i, wrapping_loops, ans, ref_counter);
            }
            wrapping_loops.pop();
        }
        Stmt::Ref(_) => {
            let accesses: Vec<_> = wrapping_loops
                .iter()
                .filter_map(|x| {
                    let LoopBound::Fixed(lb) = x.lb else {
                        return None;
                    };
                    let LoopBound::Fixed(ub) = x.ub else {
                        return None;
                    };
                    Some((x.iv.as_str(), lb as usize..ub as usize))
                })
                .collect();
            // let ary_name = x.name;
            // we could use this to provide more information...
            ans.insert(*ref_counter, accesses);
            *ref_counter += 1;
        }
        Stmt::Block(x) => {
            for i in x.iter() {
                sample_collect(i, wrapping_loops, ans, ref_counter);
            }
        }
        Stmt::Branch(_) => unimplemented!("Branch statements are not supported yet"),
    }
}

pub fn sample_gen(
    collected: &mut HashMap<usize, Vec<(&str, Range<usize>)>>,
    sampling_rate: f32,
) -> HashMap<usize, BTreeSet<Vec<usize>>> {
    let mut sampling_counts = HashMap::<usize, f32>::new();
    for (ref_id, accesses) in collected.iter() {
        let sample_count = accesses.iter().fold(1.0, |acc, (_, range)| {
            acc * (range.end - range.start) as f32 * sampling_rate
        });
        sampling_counts.insert(*ref_id, sample_count);
    }

    debug!("collected: {:#?}", collected);
    debug!("Sampling counts: {:#?}", sampling_counts);

    let mut samples = HashMap::<usize, BTreeSet<Vec<usize>>>::new();
    let mut rng = rand::thread_rng();

    for (ref_id, accesses) in collected.iter() {
        let required_samples = unsafe { ceilf32(*sampling_counts.get(ref_id).unwrap()) as usize };
        while samples.get(ref_id).unwrap_or(&BTreeSet::new()).len() < required_samples {
            let sample: Vec<usize> = accesses
                .iter()
                .map(|(_, range)| {
                    let dist = rand::distributions::Uniform::new(range.start, range.end);
                    dist.sample(&mut rng)
                })
                .collect();
            samples.entry(*ref_id).or_default().insert(sample);
        }
    }
    // ans.clone().into_iter().for_each(|(ref_id, samples)| {
    //     println!("ref_id: {}", ref_id);
    //     println!("samples: {}", samples.len());
    // });
    print!("ans: {:#?}", samples);
    samples
}
