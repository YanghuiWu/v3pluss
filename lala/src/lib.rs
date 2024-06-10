#![allow(internal_features)] // to hide the warning of using unstable features for the line below
#![feature(core_intrinsics)]
#![feature(get_mut_unchecked)]

use std::rc::Rc;

use dace::ast::{Node, Stmt};

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
        i_loop.print_structure(0);
        assert_eq!(i_loop.node_count(), 6);
    }

    #[test]
    fn test_poly() {
        // let mut bench = polybench::_2mm(1024, 1024, 1024, 1024);
        let mut bench = polybench::trisolv(1024);
        unsafe {
            assign_ranks(Rc::get_mut_unchecked(&mut bench));
        }
        bench.print_structure(0);
    }
}
