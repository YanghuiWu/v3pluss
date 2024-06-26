#![allow(internal_features)]
#![feature(core_intrinsics)]

use std::collections::BTreeSet;
use std::intrinsics::ceilf32;
use std::ops::Range;
use std::{collections::HashMap, time::Instant};

use rand::prelude::Distribution;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use dace::ast::{LoopBound, LoopStmt, Node, Stmt};
use dace_tests::polybench;
use static_ri::tracing_ri;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("LOG_LEVEL"))
        .init();
    let mut wrapping_loop = Vec::new();
    let mut trace = polybench::gemm(128);
    let mut ref_counter = 0;
    let start = Instant::now();
    // let hist = static_rd::trace::trace(&mut trace);
    // let hist = static_rd::trace::tracing_ri(&mut trace);
    let _hist = tracing_ri(&mut trace);
    let mut ans = HashMap::new();
    sample_collect(&trace, &mut wrapping_loop, &mut ans, &mut ref_counter);
    let _samples: HashMap<usize, std::collections::BTreeSet<Vec<usize>>> =
        sample_gen(&mut ans, 0.1);
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
        Stmt::Branch(_) => unimplemented!("branch is not supported yet"),
    }
}

pub fn sample_gen(
    collected: &mut HashMap<usize, Vec<(&str, Range<usize>)>>,
    sampling_rate: f32,
) -> HashMap<usize, BTreeSet<Vec<usize>>> {
    let mut intermidiate = HashMap::<usize, f32>::new();
    collected.iter_mut().for_each(|(ref_id, accesses)| {
        let mut sample_times = 1.;
        accesses.iter_mut().for_each(|(_, range)| {
            let (lb, ub) = (range.start, range.end);
            sample_times *= (ub - lb) as f32 * sampling_rate;
        });
        intermidiate.insert(*ref_id, sample_times);
    });
    debug!("collected: {:#?}", collected);
    debug!("intermidiate: {:#?}", intermidiate);

    let mut ans = HashMap::<usize, BTreeSet<Vec<usize>>>::new();
    collected.iter_mut().for_each(|(ref_id, accesses)| {
        while ans.get(ref_id).unwrap_or(&BTreeSet::new()).len()
            < unsafe { ceilf32(*intermidiate.get(ref_id).unwrap()) as usize }
        // FIXME: no sure if f32 is enough here
        {
            let mut sample_name: Vec<usize> = Vec::new();
            accesses.iter_mut().for_each(|(_, range)| {
                let (lb, ub) = (range.start, range.end);
                let mut rng = rand::thread_rng();
                let dist = rand::distributions::Uniform::new(0, ub - lb);
                let rand_num = dist.sample(&mut rng);
                sample_name.push(rand_num);
            });
            ans.entry(*ref_id)
                .or_insert(BTreeSet::new())
                .insert(sample_name);
            // kinda weird here, feels like wasting a cycle of loop
        }
    });
    // ans.clone().into_iter().for_each(|(ref_id, samples)| {
    //     println!("ref_id: {}", ref_id);
    //     println!("samples: {}", samples.len());
    // });
    print!("ans: {:#?}", ans);
    ans
}
