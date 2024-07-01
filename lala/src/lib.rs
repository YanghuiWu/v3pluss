#![allow(internal_features)] // to hide the warning of using unstable features for the line below
#![feature(core_intrinsics)]
#![feature(get_mut_unchecked)]

use std::collections::HashMap;
use std::rc::Rc;

use dace::ast::{AryRef, Node, Stmt};

#[allow(dead_code)]
fn assign_ranks_reverse(node: &mut Node) -> i32 {
    match &mut node.stmt {
        Stmt::Loop(loop_stmt) => unsafe {
            // Recursively assign ranks to the loops in the body
            let max_inner_rank = loop_stmt
                .body
                .iter_mut()
                .map(|child| assign_ranks_reverse(Rc::get_mut_unchecked(child)))
                .max()
                .unwrap_or(0);

            // Assign the current rank to this loop
            loop_stmt.rank = max_inner_rank;

            max_inner_rank + 1
        },
        Stmt::Block(block_stmt) => unsafe {
            // Recursively assign ranks to the loops in the body
            let max_inner_rank = block_stmt
                .iter_mut()
                .map(|child| assign_ranks_reverse(Rc::get_mut_unchecked(child)))
                .max()
                .unwrap_or(0);

            max_inner_rank
        },
        _ => {
            // println!("Not loop {:?}", node.stmt);
            0
        }
    }
}

#[allow(dead_code)]
fn assign_ranks(node: &mut Node, current_rank: i32) -> i32 {
    match &mut node.stmt {
        Stmt::Loop(loop_stmt) => unsafe {
            // Assign the current rank to this loop
            loop_stmt.rank = current_rank;

            // Recursively assign ranks to the loops in the body
            let max_inner_rank = loop_stmt
                .body
                .iter_mut()
                .map(|child| assign_ranks(Rc::get_mut_unchecked(child), current_rank + 1))
                .max()
                .unwrap_or(current_rank);

            max_inner_rank
        },
        Stmt::Block(block_stmt) => unsafe {
            // Recursively assign ranks to the loops in the body
            let max_inner_rank = block_stmt
                .iter_mut()
                .map(|child| assign_ranks(Rc::get_mut_unchecked(child), current_rank))
                .max()
                .unwrap_or(current_rank);

            max_inner_rank
        },
        _ => current_rank,
    }
}

#[allow(dead_code)]
fn count_arr_refs(node: &Node) -> usize {
    let mut count = 0;

    match &node.stmt {
        Stmt::Ref(_) => count += 1,
        Stmt::Loop(loop_stmt) => {
            for child in &loop_stmt.body {
                count += count_arr_refs(child);
            }
        }
        Stmt::Block(block_stmt) => {
            for child in block_stmt {
                count += count_arr_refs(child);
            }
        }
        _ => {}
    }

    count
}

#[allow(dead_code)]
fn print_ri_and_count_arr_refs(node: &Node) -> usize {
    let mut count = 0;

    match &node.stmt {
        Stmt::Ref(arr_ref_stmt) => {
            println!("{}'s ri values: {:?}", arr_ref_stmt.name, arr_ref_stmt.ri);
            count += 1;
        }
        Stmt::Loop(loop_stmt) => {
            for child in &loop_stmt.body {
                count += print_ri_and_count_arr_refs(child);
            }
        }
        Stmt::Block(block_stmt) => {
            for child in block_stmt {
                count += print_ri_and_count_arr_refs(child);
            }
        }
        _ => {}
    }

    count
}

#[allow(dead_code)]
// Calculate reuse intervals
unsafe fn calculate_reuse_intervals(
    node: &mut Rc<Node>,
    loop_ranks: &mut HashMap<String, i32>,
    num_arr_refs: usize,
) {
    let node = Rc::get_mut_unchecked(node);
    match &mut node.stmt {
        Stmt::Ref(arr_ref_stmt) => {
            determine_reuse_intervals(arr_ref_stmt, loop_ranks, num_arr_refs);
        }
        Stmt::Loop(loop_stmt) => unsafe {
            loop_ranks.insert(loop_stmt.iv.clone(), loop_stmt.rank);
            for child in &mut loop_stmt.body {
                calculate_reuse_intervals(child, loop_ranks, num_arr_refs);
            }
        },
        Stmt::Block(block_stmt) => unsafe {
            for child in block_stmt {
                calculate_reuse_intervals(child, loop_ranks, count_arr_refs(child));
            }
        },
        _ => {}
    }
}

#[allow(dead_code)]
/// Determine reuse intervals based on the prompt's criteria
fn determine_reuse_intervals(
    arr_ref_stmt: &mut AryRef,
    loop_ranks: &mut HashMap<String, i32>,
    num_arr_refs: usize,
) -> Vec<String> {
    let max_rank = *loop_ranks.values().max().unwrap_or(&0);
    println!("{} is under {} loops", arr_ref_stmt.name, max_rank + 1);
    let indices = (arr_ref_stmt.sub)(&[0, 1, 2, 3]);
    let named_indices: Vec<String> = indices
        .iter()
        .map(|val| match val {
            0 => "i".to_string(),
            1 => "j".to_string(),
            2 => "k".to_string(),
            3 => "l".to_string(),
            _ => format!("Dimension > 3: {}", val),
        })
        .collect();
    let mut ri_values = vec![];
    // search for the ranks of the indices
    let mut f_rank = *loop_ranks.get(&named_indices[0]).unwrap_or(&0);
    let p_rank;
    if named_indices.len() > 1 {
        p_rank = *loop_ranks.get(&named_indices[1]).unwrap_or(&0);
    } else {
        p_rank = f_rank;
        f_rank = i32::MAX;
    }
    // print!("f_rank: {}, p_rank: {}\n", f_rank, p_rank);

    if f_rank > p_rank || (f_rank != 0 && p_rank != 0) {
        ri_values.push(format!("{}", num_arr_refs));
    }

    if (f_rank == 0 || p_rank == 0) && !(f_rank == 1 && p_rank == 0) && max_rank > 0 {
        ri_values.push(format!("{}n", num_arr_refs));
    }

    if ((f_rank == 1 && p_rank == 2)
        || ((f_rank - p_rank).abs() == 1 && (f_rank == 0 || p_rank == 0)))
        && max_rank > 1
    {
        ri_values.push(format!("{}n^2", num_arr_refs));
    }

    if f_rank == max_rank || p_rank == max_rank {
        ri_values.push("inf".to_string()); // Meaning Infinite
    }

    arr_ref_stmt.ri = ri_values.clone();
    ri_values
}

#[cfg(test)]
mod tests {
    use dace::arybase::set_arybase;
    use dace::ast::Node;
    use dace_tests::polybench;
    use static_rd::trace::trace;
    use static_rd::LRUStack;

    use super::*;

    #[test]
    fn matmul() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]
        let ref_c = Node::new_ref("C", vec![n, n], |ijk| {
            vec![ijk[0] as usize, ijk[1] as usize]
        });
        let ref_a = Node::new_ref("A", vec![n, n], |ijk| {
            vec![ijk[0] as usize, ijk[2] as usize]
        });
        let ref_b = Node::new_ref("B", vec![n, n], |ijk| {
            vec![ijk[2] as usize, ijk[1] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        // let loop_order = &mut [&mut i_loop, &mut j_loop, &mut k_loop];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        let mut loop_order = vec![j_loop, k_loop, i_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_c, ref_a, ref_b]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let mut nested_loops_top = dace::nest_loops(loop_order);

        let arr_refs = count_arr_refs(&nested_loops_top);

        let (tbl, _size) = set_arybase(&mut nested_loops_top);
        println!("{:?}", tbl);
        unsafe {
            assign_ranks(Rc::get_mut_unchecked(&mut nested_loops_top), 0);
            calculate_reuse_intervals(&mut nested_loops_top, &mut HashMap::new(), arr_refs);
        }
        nested_loops_top.print_structure(0);
        let result = trace(&mut nested_loops_top, LRUStack::new());
        println!("{}", result.0);
        print_ri_and_count_arr_refs(&nested_loops_top);
    }

    #[test]
    fn test_poly() {
        // let mut bench = polybench::mvt(1024); // fixed the issue with the mvt with single array
        let _bench = polybench::stencil(1024);
        // let mut bench = polybench::seidel_2d(10, 10); // fixed the issue with self define loop indices
        // let mut bench = polybench::gemver(1024); // fixed the issue with the mvt with single array
        // let mut bench = polybench::syrk(256, 256);

        // let mut bench = polybench::trmm_trace(1024, 1024);
        let mut bench = polybench::symm(1024, 1024);
        unsafe {
            assign_ranks(Rc::get_mut_unchecked(&mut bench), 0);
        }
        bench.print_structure(0);
        let arr_refs = count_arr_refs(&bench);
        unsafe {
            calculate_reuse_intervals(&mut bench, &mut HashMap::new(), arr_refs);
        }
        print_ri_and_count_arr_refs(&bench);

        // let result = trace(&mut bench, LRUStack::new());
        // println!("{}", result.0);
    }
}
