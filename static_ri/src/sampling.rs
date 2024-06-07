use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};

use fxhash::FxHashMap;
use tracing::debug;

use dace::arybase::set_arybase;
use dace::ast::{AryRef, BranchStmt, LoopBound, LoopStmt, Node, Stmt};
use hist::Hist;

use crate::ri_utils::access3addr;

static COUNTER: AtomicI64 = AtomicI64::new(0);

pub fn tracing_ri(code: &mut Rc<Node>) -> Hist {
    let mut hist = Hist::new();
    #[allow(non_snake_case)]
    let mut LAT_hash: FxHashMap<String, FxHashMap<u64, i64>> = Default::default();
    set_arybase(code);
    trace_ri(code, &mut LAT_hash, &[], &mut hist);
    hist
}

#[allow(non_snake_case)]
fn trace_ri(
    code: &Rc<Node>,
    LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ivec: &[i32],
    hist: &mut Hist,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            handle_ref_stmt(ary_ref, LAT_hash, ivec, hist);
        }
        Stmt::Loop(aloop) => {
            handle_loop_stmt(aloop, LAT_hash, ivec, hist);
        }
        Stmt::Block(blk) => {
            blk.iter().for_each(|s| trace_ri(s, LAT_hash, ivec, hist));
        }
        Stmt::Branch(stmt) => {
            handle_branch_stmt(stmt, LAT_hash, ivec, hist);
        }
    }
}

#[allow(non_snake_case)]
fn handle_ref_stmt(
    ary_ref: &AryRef,
    LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ivec: &[i32],
    hist: &mut Hist,
) {
    debug!("trace_ri arr ref: {:#?}", ary_ref);
    let addr = access3addr(ary_ref, ivec) as u64;
    debug!("addr: {}", addr);
    let str_name = ary_ref.name.clone();
    let mut prev_counter: Option<i64> = None;
    let local_counter = COUNTER.load(Ordering::Relaxed);

    match LAT_hash.entry(str_name) {
        Entry::Occupied(mut entry) => match entry.get_mut().entry(addr) {
            Entry::Occupied(mut inner) => {
                prev_counter = Some(inner.insert(local_counter));
            }
            Entry::Vacant(entry) => {
                entry.insert(local_counter);
            }
        },
        Entry::Vacant(entry) => {
            let mut inner_hash: FxHashMap<u64, i64> = Default::default();
            inner_hash.insert(addr, local_counter);
            entry.insert(inner_hash);
        }
    }

    if let Some(prev_counter) = prev_counter {
        // Update reuse interval
        let ri = Some((local_counter - prev_counter) as usize);
        // FIXME: hist seems weird, how to deal with -1(the ri of never accessed again elements)
        hist.add_dist(ri);
    } else {
        hist.add_dist(None);
    }

    COUNTER.fetch_add(1, Ordering::Relaxed);
    debug!("counter: {}", COUNTER.load(Ordering::Relaxed));
    debug!("LAT_hash:{:#?}", LAT_hash);
    debug!("hist: {}", hist);
}

#[allow(non_snake_case)]
fn handle_loop_stmt(
    aloop: &LoopStmt,
    LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ivec: &[i32],
    hist: &mut Hist,
) {
    if let LoopBound::Fixed(lb) = aloop.lb {
        if let LoopBound::Fixed(ub) = aloop.ub {
            for i in lb..ub {
                let mut myvec = ivec.to_owned();
                myvec.push(i);
                for stmt in &aloop.body {
                    trace_ri(stmt, LAT_hash, &myvec, hist);
                }
            }
        } else {
            panic!("Dynamic loop upper bound is not supported");
        }
    } else {
        panic!("Dynamic loop lower bound is not supported");
    }
}

#[allow(non_snake_case)]
fn handle_branch_stmt(
    stmt: &BranchStmt,
    LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ivec: &[i32],
    hist: &mut Hist,
) {
    if (stmt.cond)(ivec) {
        trace_ri(&stmt.then_body, LAT_hash, ivec, hist);
    } else if let Some(else_body) = &stmt.else_body {
        trace_ri(else_body, LAT_hash, ivec, hist);
    }
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn sample_ri(
    code: &Rc<Node>,
    samples: HashMap<usize, BTreeSet<Vec<usize>>>,
    _LAT_hash: &mut FxHashMap<String, FxHashMap<u64, i64>>,
    ivec: &[i32],
    _hist: &mut Hist,
    counter_ref: &mut usize,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            debug!("sample_ri arr ref: {:#?}", code);
            let _addr = access3addr(ary_ref, ivec) as u64;
            let Some(_ary_samples) = samples.get(counter_ref) else {
                return;
            };
            *counter_ref += 1;
        }
        Stmt::Loop(_loop_ref) => todo!(),
        Stmt::Block(_block_ref) => todo!(),
        Stmt::Branch(_) => todo!(),
    }
}
