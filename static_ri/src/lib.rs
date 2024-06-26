use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use fxhash::FxHashMap;
use tracing::debug;

use dace::arybase::set_arybase;
use dace::ast::{AryRef, BranchStmt, LoopBound, LoopStmt, Node, Stmt};
use hist::Hist;

/// Calculate the memory address based on the array reference and index vector.
///
/// # Parameters
/// - `ary_ref`: Reference to the array metadata.
/// - `ivec`: Index vector representing the access pattern.
/// - `data_size`: Size of the data element in bytes.
/// - `cache_line_size`: Size of the cache line in bytes.
///
/// # Returns
/// The computed memory address.
pub fn access3addr(
    ary_ref: &AryRef,
    ivec: &[i32],
    data_size: usize,
    cache_line_size: usize,
) -> usize {
    let ary_index = (ary_ref.sub)(ivec);
    if ary_index.len() != ary_ref.dim.len() {
        panic!("Array index and dimension do not match");
    }

    let offset = ary_index
        .iter()
        .zip(ary_ref.dim.iter())
        .fold(0, |acc, (&i, &d)| acc * d + i);

    (ary_ref.base.unwrap() + offset) * data_size / cache_line_size
}

struct TracingContext<'a> {
    lat_hash: FxHashMap<String, FxHashMap<u64, i64>>,
    hist: Hist,
    ivec: Vec<i32>,
    code: &'a Rc<Node>,
    counter: i64,
    ds: usize,
    cls: usize,
}

impl<'a> TracingContext<'a> {
    fn new(code: &'a Rc<Node>) -> Self {
        TracingContext {
            lat_hash: Default::default(),
            hist: Hist::new(),
            ivec: vec![],
            code,
            counter: 0,
            ds: 8,
            cls: 64,
        }
    }

    fn trace_ri(&mut self) -> Hist {
        set_arybase(self.code);
        self.trace_node(self.code);
        self.hist.clone()
    }

    fn trace_node(&mut self, node: &Rc<Node>) {
        match &node.stmt {
            Stmt::Ref(ary_ref) => self.handle_ref_stmt(ary_ref),
            Stmt::Loop(aloop) => self.handle_loop_stmt(aloop),
            Stmt::Block(blk) => blk.iter().for_each(|s| self.trace_node(s)),
            Stmt::Branch(stmt) => self.handle_branch_stmt(stmt),
        }
    }

    fn handle_ref_stmt(&mut self, ary_ref: &AryRef) {
        debug!("trace_ri arr ref: {:#?}", ary_ref);
        let addr = access3addr(ary_ref, &self.ivec, self.ds, self.cls) as u64;
        debug!("addr: {}", addr);
        let str_name = ary_ref.name.clone();
        let mut prev_counter: Option<i64> = None;
        let local_counter = self.counter;

        match self.lat_hash.entry(str_name) {
            Entry::Occupied(mut entry) => match entry.get_mut().entry(addr) {
                Entry::Occupied(mut inner) => {
                    prev_counter = Some(inner.insert(local_counter));
                }
                Entry::Vacant(entry) => {
                    entry.insert(local_counter);
                }
            },
            Entry::Vacant(entry) => {
                let mut inner_hash = FxHashMap::default();
                inner_hash.insert(addr, local_counter);
                entry.insert(inner_hash);
            }
        }

        let ri = prev_counter.map(|prev| (local_counter - prev) as usize);
        self.hist.add_dist(ri);
        // FIXME: hist seems weird, how to deal with -1(the ri of never accessed again elements)

        self.counter += 1;

        debug!("counter: {}", self.counter);
        debug!("LAT_hash:{:#?}", self.lat_hash);
        debug!("hist: {}", self.hist);
    }

    fn handle_loop_stmt(&mut self, aloop: &LoopStmt) {
        if let (LoopBound::Fixed(lb), LoopBound::Fixed(ub)) = (&aloop.lb, &aloop.ub) {
            for i in *lb..*ub {
                self.ivec.push(i);
                aloop.body.iter().for_each(|stmt| self.trace_node(stmt));
                self.ivec.pop(); // TODO: check if this is correct. Added.
            }
        } else {
            panic!("Dynamic loop bounds are not supported");
        }
    }

    fn handle_branch_stmt(&mut self, stmt: &BranchStmt) {
        if (stmt.cond)(&self.ivec) {
            self.trace_node(&stmt.then_body);
        } else if let Some(else_body) = &stmt.else_body {
            self.trace_node(else_body);
        }
    }

    #[allow(dead_code)]
    fn sample_ri(
        &mut self,
        samples: HashMap<usize, BTreeSet<Vec<usize>>>,
        counter_ref: &mut usize,
    ) {
        match &self.code.stmt {
            Stmt::Ref(ary_ref) => {
                debug!("sample_ri arr ref: {:#?}", self.code);
                let _addr = access3addr(ary_ref, &self.ivec, self.ds, self.cls) as u64;
                if samples.get(counter_ref).is_some() {
                    *counter_ref += 1;
                }
            }
            _ => todo!(),
        }
    }
}

pub fn tracing_ri(code: &mut Rc<Node>) -> Hist {
    let mut context = TracingContext::new(code);
    context.trace_ri().clone()
}
