#[cfg(test)]
mod tests {
    use dace;
    use dace::arybase::set_arybase;
    use dace::ast::{Node, Stmt};
    use static_ri::{access3addr, tracing_ri};

    #[test]
    fn test_access3addr() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let mut nested_loops_top = dace::nested_loops(&vec!["i", "j", "k"], 0, ubound);

        let ref_c = dace::a_ref("C", vec![n, n], vec!["i", "j"]);
        let ref_a = dace::a_ref("A", vec![n, n], vec!["i", "k"]);
        let ref_b = dace::a_ref("B", vec![n, n], vec!["k", "j"]);

        for refs in &mut [ref_c.clone(), ref_a.clone(), ref_b.clone()] {
            dace::insert_at(refs, &mut nested_loops_top, "k");
        }

        set_arybase(&nested_loops_top);

        println!("{:?}", ref_c.stmt);
        println!("{:?}", ref_a.stmt);
        println!("{:?}", ref_b.stmt);
        let ivec = vec![1, 2, 3]; // Replace with the index vector for your test
        let data_size = 8; // Replace with the data size for your test
        let cache_line_size = 8; // Replace with the cache line size for your test

        for node in [ref_c, ref_a, ref_b].iter() {
            if let Stmt::Ref(ary_ref) = &node.stmt {
                let result = access3addr(ary_ref, &ivec, data_size, cache_line_size);
                println!("Reside in Cache Line: #{}\n", result);
            } else {
                panic!("Expected Stmt::Ref");
            }
        }
        //
        // let a = get_rank_indices(ref_c.clone());
        // println!("a: {:?}", a);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 8);

        println!("{}", hist);
    }

    #[test]
    fn test_tracing_ri2() {
        let n: usize = 32; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_c = Node::new_ref("C", vec![n, n], |ijk| {
            vec![ijk[0] as usize, ijk[1] as usize]
        });
        #[allow(unused_variables)]
        let ref_a = Node::new_ref("A", vec![n, n], |ijk| {
            vec![ijk[0] as usize, ijk[2] as usize]
        });
        #[allow(unused_variables)]
        let ref_b = Node::new_ref("B", vec![n, n], |ijk| {
            vec![ijk[2] as usize, ijk[1] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let mut loop_order = vec![i_loop, j_loop, k_loop];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        // let loop_order = &mut [&mut j_loop, &mut k_loop, &mut i_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_a.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = dace::nest_loops(loop_order);

        // let n: usize = 100; // array dim
        // let node = Node::new_ref("C", vec![n, n], |ijk| {
        //     vec![ijk[0] as usize, ijk[1] as usize]
        // });
        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 64);

        // Assert the expected result
        // Replace with your own expected result
        println!("{}", hist);
        // assert_eq!(hist, Hist::new());
    }

    #[test]
    fn test_tracing_101() {
        let n = 8;
        let mut nest_loops = dace::nested_loops(&vec!["i", "j", "k"], 0, n as i32);
        let mut ref_c = dace::a_ref("ref_c", vec![n, n], vec!["i", "j"]);
        dace::insert_at(&mut ref_c, &mut nest_loops, "k");
        let mut ref_a = dace::a_ref("ref_a", vec![n, n], vec!["i", "k"]);
        dace::insert_at(&mut ref_a, &mut nest_loops, "k");
        let mut ref_b = dace::a_ref("ref_b", vec![n, n], vec!["k", "j"]);
        dace::insert_at(&mut ref_b, &mut nest_loops, "k");

        set_arybase(&nest_loops);
        let hist = tracing_ri(&mut nest_loops.clone(), 8, 64);
        println!("{}", hist);
    }

    #[test]
    fn test_tracing_101_2() {
        let n = 20;
        let mut nest_loops = dace::nested_loops(&vec!["i", "j", "k"], 0, n as i32);
        let ref_c = dace::a_ref("ref_c", vec![n, n], vec!["i", "j"]);
        let ref_a = dace::a_ref("ref_a", vec![n, n], vec!["i", "k"]);
        let ref_b = dace::a_ref("ref_b", vec![n, n], vec!["k", "j"]);

        for s in &mut [ref_c, ref_a, ref_b] {
            dace::insert_at(s, &mut nest_loops, "k");
        }

        set_arybase(&nest_loops);
        println!("{}", tracing_ri(&mut nest_loops.clone(), 8, 40));
    }

    #[test]
    fn test_tracing_ri_101_imp() {
        let n: usize = 8; // array dim
        let ubound = n as i32; // loop bound
        let h_loop = Node::new_single_loop("h", 0, ubound);
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);

        // let mut ref_o = Node::new_ref("Refo", vec![n, n], |ij| {
        //     vec![ij[1] as usize, ij[2] as usize]
        // });
        // Node::extend_loop_body(&mut j_loop, &mut ref_o);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_a = Node::new_ref("C", vec![n, n], |ijk| {
            vec![(ijk[1] as usize + ijk[2] as usize) / 2, ijk[3] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let mut loop_order = vec![h_loop, i_loop, j_loop, k_loop];
        [ref_a.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = dace::nest_loops(loop_order);
        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 8);

        println!("{}", hist);
    }
    /*
    #[test]
    fn test_tracing_ri_1010b() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);
        let mut l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[2] as usize, ijklm[4] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let loop_order = &mut [
            &mut i_loop,
            &mut j_loop,
            &mut k_loop,
            &mut l_loop,
            &mut m_loop,
        ];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = nest_loops(loop_order);

        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 64);

        println!("{}", hist);
    }
    #[test]
    fn test_tracing_ri_1010b_imperfect() {
        let n: usize = 2; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);
        let mut l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);


        let mut ref_o = Node::new_ref("Refo", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[2] as usize, ijklm[3] as usize]
        });
        Node::extend_loop_body(&mut m_loop, &mut ref_o);

        #[allow(unused_variables)]
        let ref_i = Node::new_ref("Refi", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[2] as usize, ijklm[4] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let loop_order = &mut [
            &mut i_loop,
            &mut j_loop,
            &mut k_loop,
            &mut l_loop,
            &mut m_loop,
        ];
        [ref_i.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));


        let nested_loops_top = nest_loops(loop_order);

        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 8);

        println!("{}", hist);
    }
    #[test]
    fn test_tracing_ri_10b01() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);
        let mut l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[4] as usize, ijklm[2] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let loop_order = &mut [
            &mut i_loop,
            &mut j_loop,
            &mut k_loop,
            &mut l_loop,
            &mut m_loop,
        ];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        // let loop_order = &mut [&mut j_loop, &mut k_loop, &mut i_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = nest_loops(loop_order);

        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 64);

        println!("{}", hist);
    }
    #[test]
    fn test_tracing_ri_b0101() {
        let n: usize = 5; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);
        let mut l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[2] as usize, ijklm[4] as usize, ijklm[0] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let loop_order = &mut [
            &mut i_loop,
            &mut j_loop,
            &mut k_loop,
            &mut l_loop,
            &mut m_loop,
        ];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = nest_loops(loop_order);

        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 64);

        println!("{}", hist);
    }
    #[test]
    fn test_tracing_ri4() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);
        let mut l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);
        let mut n_loop = Node::new_single_loop("n", 0, ubound);
        let mut o_loop = Node::new_single_loop("o", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_3 = Node::new_ref("Ref3", vec![n, n, n, n], |ijklm| {
            vec![
                ijklm[0] as usize,
                ijklm[2] as usize,
                ijklm[4] as usize,
                ijklm[6] as usize,
            ]
        });

        // Choose the loop order here by specifying the order of the loops
        let loop_order = &mut [
            &mut i_loop,
            &mut j_loop,
            &mut k_loop,
            &mut l_loop,
            &mut m_loop,
            &mut n_loop,
            &mut o_loop,
        ];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        // let loop_order = &mut [&mut j_loop, &mut k_loop, &mut i_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = nest_loops(loop_order);

        // let n: usize = 100; // array dim
        // let node = Node::new_ref("C", vec![n, n], |ijk| {
        //     vec![ijk[0] as usize, ijk[1] as usize]
        // });
        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 64);

        // Assert the expected result
        // Replace with your own expected result
        println!("{}", hist);
        // assert_eq!(hist, Hist::new());
    }

    #[test]
    fn test_tracing_ri_100101() {
        let n: usize = 10; // array dim
        let ubound = n as i32; // loop bound
        let mut i_loop = Node::new_single_loop("i", 0, ubound);
        let mut j_loop = Node::new_single_loop("j", 0, ubound);
        let mut k_loop = Node::new_single_loop("k", 0, ubound);
        let mut l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);
        let mut n_loop = Node::new_single_loop("n", 0, ubound);

        // creating C[i,j] += A[i,k] * B[k,j]\
        #[allow(unused_variables)]
        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[3] as usize, ijklm[5] as usize]
        });

        // Choose the loop order here by specifying the order of the loops
        let loop_order = &mut [
            &mut i_loop,
            &mut j_loop,
            &mut k_loop,
            &mut l_loop,
            &mut m_loop,
            &mut n_loop,
        ];
        // let loop_order = &mut [&mut i_loop, &mut k_loop, &mut j_loop];
        // let loop_order = &mut [&mut j_loop, &mut k_loop, &mut i_loop];
        // Add array references to the innermost loop after nesting the loops
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = nest_loops(loop_order);

        // let n: usize = 100; // array dim
        // let node = Node::new_ref("C", vec![n, n], |ijk| {
        //     vec![ijk[0] as usize, ijk[1] as usize]
        // });
        set_arybase(&nested_loops_top);

        let hist = tracing_ri(&mut nested_loops_top.clone(), 8, 64);

        // Assert the expected result
        // Replace with your own expected result
        println!("{}", hist);
        // assert_eq!(hist, Hist::new());
    }


     */
}
