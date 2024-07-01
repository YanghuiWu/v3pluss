#![feature(get_mut_unchecked)]
#![allow(internal_features)] // to hide the warning of using unstable features for the line below
#![feature(core_intrinsics)]

use std::rc::Rc;

use crate::ast::Node;

pub mod arybase;
pub mod ast;
pub mod iter;

//Giordan's method for nest_loops that Woody (had already) made below.
pub fn loop_body(stmts: &[&mut Rc<Node>]) {
    stmts.windows(2).rev().for_each(|window| {
        let prev = &mut Rc::clone(window[0]);
        let next = &mut Rc::clone(window[1]);
        Node::extend_loop_body(prev, next);
    });
}

pub fn nested_loops(loop_names: &[&str], lb: i32, ub: i32) -> Rc<Node> {
    nest_loops(create_loops(loop_names, lb, ub))
}

pub fn nest_loops(mut order: Vec<Rc<Node>>) -> Rc<Node> {
    let mut outer_loop = Rc::clone(&order[0]);
    for loop_node in &mut order[1..] {
        Node::extend_loop_body(&mut outer_loop, loop_node);
        outer_loop = Rc::clone(loop_node);
    }
    Rc::clone(&order[0])
}

pub fn create_loops(loop_names: &[&str], lb: i32, ub: i32) -> Vec<Rc<Node>> {
    loop_names
        .iter()
        .map(|&name| Node::new_single_loop(name, lb, ub))
        .collect()
}

fn generate_sub(
    indices: Vec<String>,
    loops: Vec<String>,
) -> Box<dyn for<'a> Fn(&'a [i32]) -> Vec<usize>> {
    Box::new(move |ivec: &[i32]| {
        // println!("indices: {:?}", indices);
        // println!("loops: {:?}", loops);
        let mut rank_indices = vec![];
        for idx in indices.iter() {
            if let Some(pos) = loops.iter().position(|i| *i == *idx) {
                rank_indices.push(pos);
                // println!("pos: {:?}", pos);
            }
        }
        rank_indices.iter().map(|&pos| ivec[pos] as usize).collect()
    })
}

// Get the loop indices for a given node.
pub fn get_loops_indices(node: Rc<Node>) -> Vec<String> {
    let mut loops = Vec::new();
    let mut current_node = Some(node);
    while let Some(node) = current_node {
        if let ast::Stmt::Loop(loop_stmt) = &node.stmt {
            loops.push(loop_stmt.iv.clone());
        }
        current_node = node.parent.upgrade();
    }
    loops.reverse();
    loops
}

pub fn a_ref(nm: &str, dim: Vec<usize>, ind: Vec<&str>) -> Rc<Node> {
    let ref_stmt = ast::AryRef {
        name: nm.to_string(),
        dim,
        indices: ind.iter().map(|s| s.to_string()).collect(),
        sub: Box::new(|_i| vec![0]),
        base: None,
        ref_id: None,
        ri: vec![],
    };
    Node::new_node(ast::Stmt::Ref(ref_stmt))
}

pub fn insert_at(node: &mut Rc<Node>, mut head: &mut Rc<Node>, iv: &str) {
    let stmt = unsafe { Rc::get_mut_unchecked(&mut head) };
    match &mut stmt.stmt {
        ast::Stmt::Loop(loop_stmt) => {
            if loop_stmt.iv == iv {
                insert_node(head, node);
            } else {
                for child in &mut loop_stmt.body {
                    insert_at(node, child, iv);
                    return;
                }
            }
        }
        ast::Stmt::Block(children) => {
            for child in children.iter_mut() {
                insert_at(node, child, iv);
                return;
            }
        }
        _ => {
            panic!("Don't support branching yet!");
        }
    }
}

fn insert_node(a_loop: &mut Rc<Node>, node: &mut Rc<Node>) {
    Node::extend_loop_body(a_loop, node);
    let node = unsafe { Rc::get_mut_unchecked(node) };
    if let ast::Stmt::Ref(ref_stmt) = &mut node.stmt {
        ref_stmt.sub = generate_sub(
            ref_stmt.indices.clone(),
            get_loops_indices(Rc::clone(a_loop)),
        );
    }
}
