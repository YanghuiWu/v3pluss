#![allow(internal_features)] // to hide the warning of using unstable features for the line below
#![feature(core_intrinsics)]
#![feature(get_mut_unchecked)]

use std::collections::HashMap;
use std::rc::Rc;

use dace::ast::{AryRef, Node, Stmt};

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
fn calculate_reuse_intervals(
    node: &mut Rc<Node>,
    loop_ranks: &mut HashMap<String, i32>,
    num_arr_refs: usize,
) {
    let node = unsafe { Rc::get_mut_unchecked(node) };
    match &mut node.stmt {
        Stmt::Ref(arr_ref_stmt) => {
            determine_reuse_intervals(arr_ref_stmt, loop_ranks, num_arr_refs);
        }
        Stmt::Loop(loop_stmt) => {
            loop_ranks.insert(loop_stmt.iv.clone(), loop_stmt.rank);
            for child in &mut loop_stmt.body {
                calculate_reuse_intervals(child, loop_ranks, num_arr_refs);
            }
        }
        Stmt::Block(block_stmt) => {
            for child in block_stmt {
                calculate_reuse_intervals(child, loop_ranks, count_arr_refs(child));
            }
        }
        _ => {}
    }
}

#[allow(dead_code)]
fn access_matrix(arr_ref_stmt: &mut AryRef, loops: Vec<String>) -> Vec<Vec<u8>> {
    let mut matrix: Vec<Vec<u8>> = Vec::new();
    for i in loops {
        let mut dim: Vec<u8> = Vec::with_capacity(arr_ref_stmt.indices.len());
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
    //print!("{:?}\n\n", matrix);
    matrix
}

#[allow(dead_code)]
fn matrix_production(node: &mut Rc<Node>, loops: &mut Vec<String>) -> Vec<Vec<Vec<u8>>> {
    let node = unsafe { Rc::get_mut_unchecked(node) };
    let mut matrix: Vec<Vec<Vec<u8>>> = Vec::new();
    match &mut node.stmt {
        Stmt::Ref(arr_ref_stmt) => {
            // call some other function which returns Vec<Vec<u8>> which is then pushed to matrix
            matrix.push(access_matrix(arr_ref_stmt, loops.to_vec()));
            //return matrix;
            //print!("MATRIX TEST {:?}\n\n", matrix);
        }
        Stmt::Loop(loop_stmt) => {
            loops.push(loop_stmt.iv.clone());
            for child in &mut loop_stmt.body {
                matrix.extend(matrix_production(child, loops));
            }
        }
        Stmt::Block(block_stmt) => {
            for child in block_stmt {
                matrix.extend(matrix_production(child, loops));
            }
        }
        _ => {}
    }

    matrix
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

#[allow(dead_code)]
fn find_locality_position(matrix: Vec<Vec<u8>>) -> i32 {
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

#[allow(dead_code)]
fn generalized_determine_reuse_intervals(matrixes: Vec<Vec<Vec<u8>>>, references: Vec<&str>) {
    for (ref_index, reference) in references.iter().enumerate() {
        let access_vector: Vec<u8> = matrixes[ref_index]
            .iter()
            .map(|dim| if dim.iter().any(|&x| x == 1) { 1 } else { 0 })
            .collect();

        let locality_position: i32 = find_locality_position(matrixes[ref_index].clone());

        println!("{reference} ri probability distribution:");
        println!("{:?}", access_vector);
        let constant: usize = references.len();
        let mut _curr: String;
        let mut prior: String = String::from("");
        let mut zero_count: usize = access_vector.iter().filter(|&&x| x == 0).count();
        for loop_index in 0..access_vector.len() {
            if (access_vector[loop_index] == 0 || loop_index as i32 == locality_position)
                & ((access_vector.len() == loop_index + 1)
                    || ((access_vector.len() != loop_index + 1)
                        & (access_vector[loop_index + 1] == 1)))
            {
                let curr = match loop_index as i32 {
                    x if x == locality_position => prior.replace('b', ""),
                    x if x > locality_position => {
                        zero_count -= 1;
                        let mut prob = String::from("1/n^");
                        prob.push_str(&zero_count.to_string());
                        prob
                    }
                    _ => {
                        zero_count -= 1;
                        let mut prob = String::from("1/b*n^");
                        prob.push_str(&zero_count.to_string());
                        prob
                    }
                };

                let power: usize = access_vector.len() - loop_index - 1;

                if prior.is_empty() {
                    println!("{constant}n^{power}: {curr}");
                } else {
                    println!("{constant}n^{power}: {curr} - {prior}");
                }
                prior = curr;
            }
        }

        println!();
    }
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
        let ref_c = dace::a_ref("C", vec![n, n], vec!["i", "j"]);
        let ref_a = dace::a_ref("A", vec![n, n], vec!["i", "k"]);
        let ref_b = dace::a_ref("B", vec![n, n], vec!["k", "j"]);

        let ref_d = dace::a_ref("D", vec![n, n, n], vec!["j", "l", "n"]);
        let ref_e = dace::a_ref("E", vec![n, n, n], vec!["j", "l", "l"]);
        let ref_f = dace::a_ref("F", vec![n, n, n], vec!["i", "j", "m"]);

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
        let mut nested_loops_top: Rc<Node> = dace::nest_loops(loop_order);
        let references: Vec<&str> = vec!["C", "A", "B", "D", "E", "F"];

        // rank assignment for loops
        dace::assign_ranks(&mut nested_loops_top, 0);
        // loop matrix is found where for each array access we store essentially a 2d
        // array by dimension and if a given loop has an influnce on a respective dimension
        let loop_matrixes: Vec<Vec<Vec<u8>>> =
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
        let ref_c = dace::a_ref("C", vec![n, n], vec!["i", "j"]);
        let ref_a = dace::a_ref("A", vec![n, n], vec!["i", "k"]);
        let ref_b = dace::a_ref("B", vec![n, n], vec!["k", "j"]);

        // Choose the loop order here by specifying the order of the loops
        //let mut loop_order = &mut [&mut i_loop, &mut j_loop, &mut k_loop];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        let mut loop_order = vec![i_loop, j_loop, k_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_c, ref_a, ref_b]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let mut nested_loops_top = dace::nest_loops(loop_order);

        //let arr_refs = count_arr_refs(&nested_loops_top);

        // let (tbl, _size) = set_arybase(&mut nested_loops_top);
        // println!("{:?}", tbl);
        dace::assign_ranks(&mut nested_loops_top, 0);
        let references: Vec<&str> = vec!["C", "A", "B"];

        // loop matrix is found where for each array access we store essentially a 2d
        // array by dimension and if a given loop has an influnce on a respective dimension
        let loop_matrixes: Vec<Vec<Vec<u8>>> =
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
        // let mut bench = polybench::mvt(1024); // fixed the issue with the mvt with single array
        let _bench = polybench::stencil(1024);
        // let mut bench = polybench::seidel_2d(10, 10); // fixed the issue with self define loop indices
        // let mut bench = polybench::gemver(1024); // fixed the issue with the mvt with single array
        // let mut bench = polybench::syrk(256, 256);

        // let mut bench = polybench::trmm_trace(1024, 1024);
        let mut bench = polybench::symm(1024, 1024);
        dace::assign_ranks(&mut bench, 0);
        bench.print_structure(0);
        let arr_refs = count_arr_refs(&bench);
        calculate_reuse_intervals(&mut bench, &mut HashMap::new(), arr_refs);
        print_ri_and_count_arr_refs(&bench);

        // let result = trace(&mut bench, LRUStack::new());
        // println!("{}", result.0);
    }
}
