#![allow(internal_features)] // to hide the warning of using unstable features for the line below
#![feature(core_intrinsics)]
#![feature(get_mut_unchecked)]
#![allow(dead_code)]

use std::rc::Rc;

use dace::ast::{AryRef, Node, Stmt};

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

fn access_matrix(arr_ref_stmt: &mut AryRef, loops: Vec<String>) -> Vec<Vec<usize>> {
    let mut matrix: Vec<Vec<usize>> = Vec::new();
    for i in loops {
        let mut dim: Vec<usize> = Vec::with_capacity(arr_ref_stmt.indices.len());
        for j in arr_ref_stmt.indices.clone() {
            if j == i {
                dim.push(1);
                //print!("\n{j} and {i}\n");
            } else {
                dim.push(0);
            }
        }
        matrix.push(dim);
    }
    print!("{:?}\n\n", matrix);
    matrix
}

fn matrix_production(node: &mut Rc<Node>, loops: &mut Vec<String>) -> Vec<Vec<Vec<usize>>> {
    let node = unsafe { Rc::get_mut_unchecked(node) };
    let mut matrix: Vec<Vec<Vec<usize>>> = Vec::new();

    match &mut node.stmt {
        Stmt::Ref(arr_ref_stmt) => {
            // call some other function which returns Vec<Vec<usize>> which is then pushed to matrix
            let access_matrix = access_matrix(arr_ref_stmt, loops.to_vec());
            matrix.push(access_matrix.clone());
            arr_ref_stmt.access_matrix.push(access_matrix);
            //return matrix;
            //print!("MATRIX TEST {:?}\n\n", matrix);
        }
        Stmt::Loop(loop_stmt) => {
            loops.push(loop_stmt.iv.clone());
            for child in &mut loop_stmt.body {
                matrix.extend(matrix_production(child, loops));
            }
            // loops.pop();
        }
        Stmt::Block(block_stmt) => {
            for child in block_stmt {
                matrix.extend(matrix_production(child, loops));
            }
        }
        _ => {}
    }

    // print the node and the matrix
    // println!("Node: {:?}", node);
    // println!("Matrix: {:?}\n", matrix);
    matrix
}

fn find_locality_position(matrix: Vec<Vec<usize>>) -> i32 {
    let mut locality_position: i32 = -1;
    for (i, vec) in matrix.iter().enumerate() {
        if let Some(&last) = vec.last() {
            if last == 1 {
                locality_position = i as i32;
            }
        }
    }

    locality_position
}

fn generalized_determine_reuse_intervals(matrixes: Vec<Vec<Vec<usize>>>, references: Vec<&str>) {
    for (ref_index, reference) in references.iter().enumerate() {
        let access_vector: Vec<usize> = matrixes[ref_index]
            .iter()
            .map(|dim| if dim.iter().any(|&x| x == 1) { 1 } else { 0 })
            .collect();

        let locality_position: i32 = find_locality_position(matrixes[ref_index].clone());

        println!("{reference} ri probability distribution:");
        println!("access_vector: {:?}", access_vector);

        let constant: usize = references.len();
        let mut prior: String = String::from("");
        let mut zero_count: usize = access_vector.iter().filter(|&&x| x == 0).count();

        for (loop_index, &value) in access_vector.iter().enumerate() {
            if loop_index + 1 == access_vector.len() || access_vector[loop_index + 1] == 1 {
                let curr = match loop_index as i32 {
                    x if x == locality_position => prior.replace('b', ""),
                    x if x > locality_position && value == 0 => {
                        zero_count -= 1;
                        format!("1/n^{}", zero_count)
                    }
                    x if x < locality_position && value == 0 => {
                        zero_count -= 1;
                        format!("1/b*n^{}", zero_count)
                    }
                    _ => {
                        continue;
                    }
                };

                let power: usize = access_vector.len() - loop_index - 1;
                if prior.is_empty() {
                    println!("{}n^{}: {}", constant, power, curr);
                } else {
                    println!("{}n^{}: {} - {}", constant, power, curr, prior);
                }

                prior = curr;
            }
        }

        println!();
    }
}

#[cfg(test)]
mod tests {
    use dace::ast::Node;
    use dace::construct;
    use dace_tests::polybench_simplify;

    use super::*;

    #[test]
    fn simple_test() {
        let n = 10; // array dim
        let i_loop = Node::new_single_loop("i", 0, n);
        let j_loop = Node::new_single_loop("j", 0, n);

        let mut ref_c = construct::squ_ref("C", n, vec!["j"]);

        let mut nested_loops_top = construct::nest_the_loops(vec![i_loop, j_loop]);
        dace::construct::insert_at_innermost(&mut ref_c, &mut nested_loops_top);

        nested_loops_top.print_structure(0);

        construct::assign_ranks(&mut nested_loops_top, 0);
        let references: Vec<&str> = vec!["C"];

        let loop_matrixes: Vec<Vec<Vec<usize>>> =
            matrix_production(&mut nested_loops_top, &mut Vec::new());
        generalized_determine_reuse_intervals(loop_matrixes, references);
    }

    #[test]
    fn higer_dim_loop_test() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
                               //let inf_loop = Node::new_single_loop("inf", 0, ubound);
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let m_loop = Node::new_single_loop("m", 0, ubound);
        let n_loop = Node::new_single_loop("n", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]
        // also D[j,l,n] += E[j,l,l] * F[i,j,m]
        let ref_c = construct::a_ref("C", vec![n, n], vec!["i", "j"]);
        let ref_a = construct::a_ref("A", vec![n, n], vec!["i", "k"]);
        let ref_b = construct::a_ref("B", vec![n, n], vec!["k", "j"]);

        let ref_d = construct::a_ref("D", vec![n, n, n], vec!["j", "l", "n"]);
        let ref_e = construct::a_ref("E", vec![n, n, n], vec!["j", "l", "l"]);
        let ref_f = construct::a_ref("F", vec![n, n, n], vec!["i", "j", "m"]);

        // Choose the loop order here by specifying the order of the loops
        // let loop_order = &mut [&mut i_loop, &mut j_loop, &mut k_loop];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        // let mut loop_order = vec![inf_loop, i_loop, j_loop, k_loop, l_loop, m_loop, n_loop];
        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop, n_loop];

        // Add array references to the innermost loop after nesting the loops
        [ref_c, ref_a, ref_b, ref_d, ref_e, ref_f]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        //the loops which were orignally seperate are not coalessed into eachother so that they are acutally nested
        let mut nested_loops_top: Rc<Node> = construct::nest_the_loops(loop_order);
        let references: Vec<&str> = vec!["C", "A", "B", "D", "E", "F"];

        nested_loops_top.print_structure(0);
        // rank assignment for loops
        construct::assign_ranks(&mut nested_loops_top, 0);
        // loop matrix is found where for each array access we store essentially a 2d
        // array by dimension and if a given loop has an influnce on a respective dimension
        let loop_matrixes: Vec<Vec<Vec<usize>>> =
            matrix_production(&mut nested_loops_top, &mut Vec::new());
        print!("{:?}\n\n", loop_matrixes);
        generalized_determine_reuse_intervals(loop_matrixes, references);
        //ri output
    }

    #[test]
    fn matmul() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]
        let ref_c = construct::a_ref("C", vec![n, n], vec!["i", "j"]);
        let ref_a = construct::a_ref("A", vec![n, n], vec!["i", "k"]);
        let ref_b = construct::a_ref("B", vec![n, n], vec!["k", "j"]);

        // Choose the loop order here by specifying the order of the loops
        //let mut loop_order = &mut [&mut i_loop, &mut j_loop, &mut k_loop];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        let mut loop_order = vec![i_loop, j_loop, k_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_c, ref_a, ref_b]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let mut nested_loops_top = construct::nest_the_loops(loop_order);

        //let arr_refs = count_arr_refs(&nested_loops_top);

        // let (tbl, _size) = set_arybase(&mut nested_loops_top);
        // println!("{:?}", tbl);
        construct::assign_ranks(&mut nested_loops_top, 0);
        let references: Vec<&str> = vec!["C", "A", "B"];

        // loop matrix is found where for each array access we store essentially a 2d
        // array by dimension and if a given loop has an influnce on a respective dimension
        let loop_matrixes: Vec<Vec<Vec<usize>>> =
            matrix_production(&mut nested_loops_top, &mut Vec::new());
        //print!("{:?}\n\n", loop_matrixes);
        generalized_determine_reuse_intervals(loop_matrixes, references);
        //ri output

        // calculate_reuse_intervals(&mut nested_loops_top, &mut HashMap::new(), arr_refs);
        // nested_loops_top.print_structure(0);
        // let result = trace(&mut nested_loops_top, LRUStack::new());
        // println!("{}", result.0);
        // print_ri_and_count_arr_refs(&nested_loops_top);
    }

    #[test]
    fn test_poly() {
        // let mut bench = polybench_simplify::mvt(1024); // fixed the issue with the mvt with single array
        let _bench = polybench_simplify::stencil(1024);
        // let mut bench = polybench_simplify::seidel_2d(10, 10); // fixed the issue with self define loop indices
        // let mut bench = polybench_simplify::gemver(1024); // fixed the issue with the mvt with single array
        // let mut bench = polybench_simplify::syrk(256, 256);

        // let mut bench = polybench_simplify::trmm_trace(1024, 1024);
        let mut bench = polybench_simplify::symm(1024, 1024);
        construct::assign_ranks(&mut bench, 0);
        bench.print_structure(0);
        // let arr_refs = count_arr_refs(&bench);
        // calculate_reuse_intervals(&mut bench, &mut HashMap::new(), arr_refs);
        print_ri_and_count_arr_refs(&bench);

        // let result = trace(&mut bench, LRUStack::new());
        // println!("{}", result.0);
    }
}
