use crate::ast::{Node, Stmt};
use std::{collections::HashMap, rc::Rc};

use crate::ast;
use crate::types;

//Giordan's method for nest_loops that Woody (had already) made below.
pub fn loop_body(stmts: &[&mut Rc<Node>]) {
    stmts.windows(2).rev().for_each(|window| {
        let prev = &mut Rc::clone(window[0]);
        let next = &mut Rc::clone(window[1]);
        Node::extend_loop_body(prev, next);
    });
}

pub fn nested_loops(vars: &[&str], ub: i32) -> Rc<Node> {
    nest_the_loops(create_loops(vars, 0, ub))
}

pub fn nest_the_loops(mut order: Vec<Rc<Node>>) -> Rc<Node> {
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

pub fn generate_subscript(indices: &[&str], loop_index: &str) -> Box<types::DynamicBoundFunction> {
    indices
        .iter()
        .enumerate()
        .filter(|(_, value)| loop_index == **value)
        .map(|(index, _value)| Box::new(move |ivec: &[i32]| ivec[index]))
        .next()
        .expect("loop_index does not belong in indices!")
}

pub fn generate_sub(indices: &[String], loops: &[String]) -> Box<types::DynFunc> {
    //println!("indices: {:?}", indices);
    //println!("loops: {:?}", loops);

    let index_map = loops
        .iter()
        .enumerate()
        .fold(HashMap::new(), |mut acc, (index, value)| {
            acc.insert(value, index);
            acc
        });
    let rank_indices: Vec<usize> = indices
        .iter()
        .map(move |loop_index| *index_map.get(loop_index).unwrap())
        .collect();

    Box::new(move |ivec: &[i32]| rank_indices.iter().map(|&pos| ivec[pos] as usize).collect())
}

pub fn generate_sub_2(indices: &[&str], loops: &[&str]) -> Box<types::DynFunc> {
    println!("indices: {:?}", indices);
    println!("loops: {:?}", loops);

    let index_map = indices
        .iter()
        .enumerate()
        .fold(HashMap::new(), |mut acc, (index, value)| {
            acc.insert(value, index);
            acc
        });
    println!("index_map: {:?}", index_map);
    let rank_indices: Vec<usize> = loops
        .iter()
        .map(move |loop_index| *index_map.get(loop_index).unwrap())
        .collect();
    println!("rank_indices: {:?}", rank_indices);
    Box::new(move |ivec: &[i32]| rank_indices.iter().map(|&pos| ivec[pos] as usize).collect())
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

/// Creates a reference node for an array with specified dimensions and indices.
///
/// This function constructs a reference node to an array, allowing for the specification
/// of each dimension's size and the indices used for referencing. It is suitable for arrays
/// where dimensions may vary in size.
///
/// # Parameters
/// - `nm`: A string slice representing the name of the array.
/// - `dim`: A vector of `usize` where each element represents the size of a dimension in the array.
/// - `ind`: A vector of string slices, each representing an index used for array referencing.
///
/// # Returns
/// Returns a `Rc<Node>` pointing to the newly created reference node.
///
/// # Examples
/// ```rust
/// use dace::construct;
/// let array_ref = construct::a_ref("myArray", vec![10, 20, 30], vec!["i", "j", "k"]);
/// ```
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

/// Creates a square reference node for an array where all dimensions are of the same size.
///
/// This function is a specialized version of `a_ref` for creating references to square (or cubic, etc.)
/// arrays, where each dimension has the same size. It simplifies the process by requiring only a single
/// size parameter for all dimensions.
///
/// # Parameters
/// - `nm`: A string slice representing the name of the array.
/// - `dim`: A `usize` value representing the size of each dimension in the array.
/// - `ind`: A vector of string slices, each representing an index used for array referencing.
///
/// # Returns
/// Returns a `Rc<Node>` pointing to the newly created square reference node.
///
/// # Examples
/// ```rust
/// use dace::construct;
/// let square_array_ref = construct::squ_ref("mySquareArray", 10, vec!["i", "j", "k"]);
/// ```
pub fn squ_ref(nm: &str, dim: i32, ind: Vec<&str>) -> Rc<Node> {
    let dim = dim as usize;
    let dimensions = vec![dim; ind.len()]; // Create a vector with the dimension repeated for each index
    let ref_stmt = ast::AryRef {
        name: nm.to_string(),
        dim: dimensions,
        indices: ind.iter().map(|s| s.to_string()).collect(),
        sub: Box::new(|_i| vec![0]),
        base: None,
        ref_id: None,
        ri: vec![],
    };
    Node::new_node(ast::Stmt::Ref(ref_stmt))
}

pub fn insert_at(node: &mut Rc<Node>, head: &mut Rc<Node>, iv: &str) -> bool {
    let stmt = unsafe { Rc::get_mut_unchecked(head) };
    match &mut stmt.stmt {
        ast::Stmt::Loop(loop_stmt) => {
            if loop_stmt.iv == iv {
                insert_node(head, node);
                true
            } else {
                for child in &mut loop_stmt.body {
                    if insert_at(node, child, iv) {
                        return true;
                    }
                }
                false
            }
        }
        ast::Stmt::Block(children) => {
            for child in children.iter_mut() {
                if insert_at(node, child, iv) {
                    return true;
                }
            }
            false
        }
        _ => {
            panic!("Don't support branching yet!");
        }
    }
}

pub fn insert_at_innermost(node: &mut Rc<Node>, head: &mut Rc<Node>) -> String {
    let target = assign_ranks(head, 0);
    let b = insert_at(node, head, &target.loop_only(|lp| lp.iv.clone()).unwrap());
    // head.print_structure(0);
    if b {
        target.loop_only(|lp| lp.iv.clone()).unwrap()
    } else {
        "None".to_string()
    }
}

#[allow(dead_code)]
pub fn assign_ranks_reverse(node: &mut Rc<Node>) -> i32 {
    let node = unsafe { Rc::get_mut_unchecked(node) };
    match &mut node.stmt {
        Stmt::Loop(loop_stmt) => {
            // Recursively assign ranks to the loops in the body
            let max_inner_rank = loop_stmt
                .body
                .iter_mut()
                .map(assign_ranks_reverse)
                .max()
                .unwrap_or(0);

            // Assign the current rank to this loop
            loop_stmt.rank = max_inner_rank;

            max_inner_rank + 1
        }
        Stmt::Block(block_stmt) => {
            // Recursively assign ranks to the loops in the body
            let max_inner_rank = block_stmt
                .iter_mut()
                .map(assign_ranks_reverse)
                .max()
                .unwrap_or(0);

            max_inner_rank
        }
        _ => {
            // println!("Not loop {:?}", node.stmt);
            0
        }
    }
}

// #[allow(dead_code)]
// fn assign_ranks(node: &mut Rc<Node>, current_rank: i32) -> i32 {
//     let node = unsafe { Rc::get_mut_unchecked(node) };
//     match &mut node.stmt {
//         Stmt::Loop(loop_stmt) => unsafe {
//             // Assign the current rank to this loop
//             loop_stmt.rank = current_rank;
//
//             // Recursively assign ranks to the loops in the body
//             let max_inner_rank = loop_stmt
//                 .body
//                 .iter_mut()
//                 .map(|child| assign_ranks(child, current_rank + 1))
//                 .max()
//                 .unwrap_or(current_rank);
//
//             max_inner_rank
//         },
//         Stmt::Block(block_stmt) => unsafe {
//             // Recursively assign ranks to the loops in the body
//             let max_inner_rank = block_stmt
//                 .iter_mut()
//                 .map(|child| assign_ranks(child, current_rank))
//                 .max()
//                 .unwrap_or(current_rank);
//
//             max_inner_rank
//         },
//         _ => current_rank,
//     }
// }

/// Assign ranks to the loops in the AST. and return the node with the highest rank.
pub fn assign_ranks(node: &mut Rc<Node>, current_rank: i32) -> Rc<Node> {
    let cur = Rc::clone(node);
    let node_ptr = unsafe { Rc::get_mut_unchecked(node) };
    match &mut node_ptr.stmt {
        Stmt::Loop(loop_stmt) => {
            // Assign the current rank to this loop
            loop_stmt.rank = current_rank;

            // Recursively assign ranks to the loops in the body and find the node with the highest rank
            // let mut max_rank_node = Rc::clone(&noe);
            let max_inner_rank = loop_stmt
                .body
                .iter_mut()
                .map(|child| assign_ranks(child, current_rank + 1))
                .max_by_key(|n| n.rank().unwrap_or(-1))
                .unwrap_or(Rc::clone(&cur));

            // Compare the rank of the current node with the highest rank node found in its body
            if max_inner_rank.rank().unwrap_or(-1) > loop_stmt.rank {
                max_inner_rank
            } else {
                Rc::clone(node)
            }
            // max_rank_node
        }
        Stmt::Block(block_stmt) => {
            // Recursively assign ranks to the loops in the body and find the node with the highest rank
            let max_inner_rank = block_stmt
                .iter_mut()
                .map(|child| assign_ranks(child, current_rank))
                .max_by_key(|n| n.rank())
                .unwrap_or(Rc::clone(&cur));

            max_inner_rank
        }
        _ => Rc::clone(node),
    }
}

pub fn insert_node(a_loop: &mut Rc<Node>, node: &mut Rc<Node>) {
    Node::extend_loop_body(a_loop, node);
    let node = unsafe { Rc::get_mut_unchecked(node) };
    if let ast::Stmt::Ref(ref_stmt) = &mut node.stmt {
        ref_stmt.sub = generate_sub(&ref_stmt.indices, &get_loops_indices(Rc::clone(a_loop)));
    }
}
