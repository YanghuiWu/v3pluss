#![allow(internal_features)] // to hide the warning of using unstable features for the line below
#![feature(core_intrinsics)]
#![feature(get_mut_unchecked)]

use std::collections::HashMap;
use std::rc::Rc;

use dace::ast::{AryRef, Node, Stmt};

#[allow(dead_code)]
fn assign_ranks(node: &mut Node) -> usize {
    match &mut node.stmt {
        Stmt::Loop(loop_stmt) => unsafe {
            // Recursively assign ranks to the loops in the body
            let max_inner_rank = loop_stmt
                .body
                .iter_mut()
                .map(|child| assign_ranks(Rc::get_mut_unchecked(child)))
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
                .map(|child| assign_ranks(Rc::get_mut_unchecked(child)))
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
    loop_ranks: &mut HashMap<String, usize>,
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
    loop_ranks: &mut HashMap<String, usize>,
    num_arr_refs: usize,
) -> Vec<String> {
    let indices = (arr_ref_stmt.sub)(&[0, 1, 2, 3]);
    let named_indices: Vec<String> = indices
        .iter()
        .enumerate()
        .map(|(_idx, val)| match val {
            0 => "i".to_string(),
            1 => "j".to_string(),
            2 => "k".to_string(),
            3 => "l".to_string(),
            _ => format!("Dimension > 3: {}", val),
        })
        .collect();
    let mut ri_values = vec![];
    // search for the ranks of the indices
    let f_rank = *loop_ranks.get(&named_indices[0]).unwrap_or(&0);
    let p_rank = *loop_ranks.get(&named_indices[1]).unwrap_or(&0);
    // print!("f_rank: {}, p_rank: {}\n", f_rank, p_rank);

    if p_rank < f_rank || f_rank != 0 || p_rank != 0 {
        ri_values.push(format!("{}", num_arr_refs));
    }
    if (f_rank == 0 || p_rank == 0) && !(f_rank == 1 && p_rank == 0) {
        ri_values.push(format!("{}n", num_arr_refs));
    }
    if ((f_rank == 1 && p_rank == 2) || (f_rank as i32 - p_rank as i32).abs() == 1)
        && (f_rank == 0 || p_rank == 0)
    {
        ri_values.push(format!("{}n^2", num_arr_refs));
    }
    if f_rank == 0 || p_rank == 0 {
        ri_values.push("infinite".to_string());
    }

    arr_ref_stmt.ri = ri_values.clone();
    ri_values
}

#[cfg(test)]
mod tests {
    use dace::ast::Node;
    use dace_tests::polybench;

    use super::*;

    #[test]
    fn matmul() {
        let n: usize = 100; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);

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

        [ref_c, ref_a, ref_b]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(&mut k_loop, s));
        Node::extend_loop_body(&mut j_loop, &mut k_loop);
        Node::extend_loop_body(&mut i_loop, &mut j_loop);

        unsafe {
            assign_ranks(Rc::get_mut_unchecked(&mut i_loop));
        }
        println!("{:?}", i_loop.stmt);
        i_loop.print_structure(0);
        let arr_refs = count_arr_refs(&i_loop);
        unsafe {
            calculate_reuse_intervals(&mut i_loop, &mut HashMap::new(), arr_refs);
        }
        assert_eq!(i_loop.node_count(), 6);
        print_ri_and_count_arr_refs(&i_loop);
    }

    #[test]
    fn test_poly() {
        // let mut bench = polybench::mvt(1024); // TODO: fix the issue with the mvt with single array
        let mut bench = polybench::stencil(1024);
        // let mut bench = polybench::seidel_2d(10, 1024); // TODO: fix the issue with self define loop indices
        // let mut bench = polybench::gemver(1024); // TODO: fix the issue with the mvt with single array
        unsafe {
            assign_ranks(Rc::get_mut_unchecked(&mut bench));
        }
        bench.print_structure(0);
        let arr_refs = count_arr_refs(&bench);
        unsafe {
            calculate_reuse_intervals(&mut bench, &mut HashMap::new(), arr_refs);
        }
        print_ri_and_count_arr_refs(&bench);
    }
}
