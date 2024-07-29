#![feature(get_mut_unchecked)]

use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;

use fxhash::FxHashMap;
use tracing::debug;

use dace::arybase::set_arybase;
use dace::ast::{AryRef, BranchStmt, LoopBound, LoopStmt, Node, Stmt};
use dace::iter::Walk;
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
    // println!("ary_ref: {:?}", ary_ref);
    // println!("ivec: {:?}", ivec);
    // println!("ary_index: {:?}", ary_index);
    if ary_index.len() != ary_ref.dim.len() {
        panic!("Array index and dimension do not match");
    }

    let offset = ary_index
        .iter()
        .zip(ary_ref.dim.iter())
        .fold(0, |acc, (&i, &d)| acc * d + i);

    (ary_ref.base.unwrap() + offset) * data_size / cache_line_size
}

pub fn assign_ref_id(node: &Rc<Node>) {
    println!("Assigning ID...");
    let mut counter = 0;
    Walk::new(node)
        .filter(|node| matches!(&node.stmt, Stmt::Ref(_)))
        .for_each(|mut node| {
            let mutable = unsafe { Rc::get_mut_unchecked(&mut node) };
            let my_ref_id = mutable.ref_only_mut_ref(|aref| &mut aref.ref_id).unwrap();
            if my_ref_id.is_none() {
                *my_ref_id = Some(counter);
                counter += 1;
            }
        });
    println!("number of ID assigned: {}", counter);
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
    fn new(code: &'a Rc<Node>, ds: usize, cls: usize) -> Self {
        TracingContext {
            lat_hash: Default::default(),
            hist: Hist::new(),
            ivec: vec![],
            code,
            counter: 0,
            ds,
            cls, //64
        }
    }

    fn trace_ri(&mut self) -> Hist {
        // set_arybase(self.code);
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

    fn record_access_trace(&self, ary_ref: &AryRef, addr: u64) {
        let file_path = "access_trace.csv";
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        if file.metadata().unwrap().len() == 0 {
            let header = "Array Name,Ref ID,Address\n";
            file.write_all(header.as_bytes()).unwrap();
        }

        let trace_info = format!("{},{},{}\n", ary_ref.name, ary_ref.ref_id.unwrap(), addr);
        file.write_all(trace_info.as_bytes()).unwrap();
    }

    fn handle_ref_stmt(&mut self, ary_ref: &AryRef) {
        debug!("trace_ri arr ref: {:#?}", ary_ref);
        let addr = access3addr(ary_ref, &self.ivec, self.ds, self.cls) as u64;
        debug!("addr: {}", addr);

        self.record_access_trace(ary_ref, addr);

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
                if samples.contains_key(counter_ref) {
                    *counter_ref += 1;
                }
            }
            _ => todo!(),
        }
    }
}

pub fn tracing_ri(code: &mut Rc<Node>, data_size: usize, cache_line_size: usize) -> Hist {
    set_arybase(code);
    let mut context = TracingContext::new(code, data_size, cache_line_size);
    assign_ref_id(code);
    let h = context.trace_ri().clone();
    println!("{}", h);
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use dace::construct;

    #[test]
    fn test_access3addr_and_tracing() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let mut nested_loops_top = construct::nested_loops(&["i", "j", "k"], ubound);

        let ref_c = construct::a_ref("C", vec![n, n], vec!["i", "j"]);
        let ref_a = construct::a_ref("A", vec![n, n], vec!["i", "k"]);
        let ref_b = construct::a_ref("B", vec![n, n], vec!["k", "j"]);

        let mut refs = [ref_c.clone(), ref_a.clone(), ref_b.clone()];

        for a_ref in &mut refs {
            construct::insert_at(a_ref, &mut nested_loops_top, "k");
        }

        set_arybase(&nested_loops_top);

        println!("{:?}", ref_c.stmt);
        println!("{:?}", ref_a.stmt);
        println!("{:?}", ref_b.stmt);

        let ivec = vec![1, 2, 3]; // Replace with the index vector for your test

        let data_size = 8; // Replace with the data size for your test
        let cache_line_size = 8; // Replace with the cache line size for your test

        for (index, node) in refs.iter().enumerate() {
            if let Stmt::Ref(ary_ref) = &node.stmt {
                let result = access3addr(ary_ref, &ivec, data_size, cache_line_size);
                print!(
                    "{}{:?}={:?}",
                    ary_ref.name,
                    ary_ref.indices,
                    (ary_ref.sub)(&ivec)
                );
                println!(" reside in Cache Line: #{}", result);

                match index {
                    0 => assert_eq!(result, 12),  // Assertion for ref_c
                    1 => assert_eq!(result, 113), // Assertion for ref_a
                    2 => assert_eq!(result, 232), // Assertion for ref_b
                    _ => (),
                }
            }
        }

        print!("\nb = 1\t");
        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 8);
        assert_eq!(hist.hist.get(&Some(3)), Some(&900));
        assert_eq!(hist.hist.get(&Some(30)), Some(&900));
        assert_eq!(hist.hist.get(&Some(300)), Some(&900));
        assert_eq!(hist.hist.get(&None), Some(&300));

        print!("b = 5\t");
        let hist2 = tracing_ri(&mut nested_loops_top.clone(), 8, 40);
        assert_eq!(hist2.hist.get(&Some(3)), Some(&1780));
        assert_eq!(hist2.hist.get(&Some(18)), Some(&180));
        assert_eq!(hist2.hist.get(&Some(30)), Some(&800));
        assert_eq!(hist2.hist.get(&Some(180)), Some(&180));
        assert_eq!(hist2.hist.get(&None), Some(&60));

        print!("b = 10\t");
        let hist3 = tracing_ri(&mut nested_loops_top.clone(), 8, 80);
        assert_eq!(hist3.hist.get(&Some(3)), Some(&1980));
        assert_eq!(hist3.hist.get(&Some(30)), Some(&990));
        assert_eq!(hist3.hist.get(&None), Some(&30));
    }

    #[test]
    fn test_10010001() {
        let n: usize = 4; // array dim
        let ubound = n as i32; // loop bound
        let mut nested_loops =
            construct::nested_loops(&["i", "j", "k", "l", "m", "n", "o", "p"], ubound);
        let mut ref_c = construct::a_ref("c", vec![n, n, n], vec!["i", "l", "p"]);

        construct::insert_at(&mut ref_c, &mut nested_loops, "p");

        let hist = tracing_ri(&mut nested_loops, 8, 8);
        assert_eq!(hist.hist.get(&Some(4)), Some(&64512));
        assert_eq!(hist.hist.get(&Some(772)), Some(&960));
        assert_eq!(hist.hist.get(&None), Some(&64));
    }
}
