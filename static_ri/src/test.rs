#![allow(dead_code)]
#[cfg(test)]
mod tests {
    use dace::ast::Node;
    use dace::construct;
    use dace_tests::polybench_simplify;
    use static_ri::tracing_ri;

    #[test]
    fn test_poly() {
        let mut tri = polybench_simplify::mvt(1024);
        assert_eq!(tri.node_count(), 13);
        tri.print_structure(0);
        tracing_ri(&mut tri, 8, 8);
    }

    #[test]
    fn test_100b0() {
        let n = 5; // array dim

        let mut nested_loops = construct::nested_loops(&["i", "j", "k", "l", "m"], n);
        let mut ref_c = construct::squ_ref("c", n, vec!["i", "l"]);
        construct::insert_at_innermost(&mut ref_c, &mut nested_loops);
        // nested_loops.print_structure(0);

        tracing_ri(&mut nested_loops, 8, 40);
    }

    #[test]
    fn test_10b01() {
        let n = 4; // array dim

        let mut nested_loops = construct::nested_loops(&["i", "j", "k", "l", "m"], n);
        let mut ref_c = construct::squ_ref("c", n, vec!["i", "m", "k"]);
        construct::insert_at_innermost(&mut ref_c, &mut nested_loops);
        nested_loops.print_structure(0);

        tracing_ri(&mut nested_loops, 8, 16);
    }

    // #[test]
    fn test_tracing_ri_precise_ri() {
        let n = 10; // array dim
        let mut nested_loops = construct::nested_loops(&["inf", "i", "j", "k", "l", "m", "n"], n);
        let mut ref_c = construct::squ_ref("c", n, vec!["i", "m", "l"]);
        construct::insert_at_innermost(&mut ref_c, &mut nested_loops);
        nested_loops.print_structure(0);

        tracing_ri(&mut nested_loops, 8, 40);
    }

    #[test]
    fn test_p_b10() {
        let n = 8; // array dim

        let mut nested_loops = construct::nested_loops(&["c", "i", "j", "k"], n);
        let mut ref_c = construct::squ_ref("c", n, vec!["k", "j"]);

        construct::insert_at_innermost(&mut ref_c, &mut nested_loops);

        tracing_ri(&mut nested_loops, 8, 16);
    }

    #[test]
    fn test_1b0() {
        let n = 10; // array dim

        let mut nested_loops = construct::nested_loops(&["c", "i", "j", "k"], n);
        let mut ref_c = construct::squ_ref("c", n, vec!["i", "j"]);

        construct::insert_at_innermost(&mut ref_c, &mut nested_loops);

        tracing_ri(&mut nested_loops, 8, 40);
    }

    #[test]
    fn test_b10() {
        let n = 10; // array dim

        let mut nested_loops = construct::nested_loops(&["j", "k", "i"], n);
        let mut ref_c = construct::squ_ref("c", n, vec!["k", "j"]);

        construct::insert_at_innermost(&mut ref_c, &mut nested_loops);

        tracing_ri(&mut nested_loops, 8, 8);
    }

    #[test]
    fn test_111() {
        let n: usize = 8; // array dim
        let ubound = n as i32; // loop bound
        let mut nested_loops = construct::nested_loops(&["j", "i"], ubound);
        let mut ref_c = construct::a_ref("c", vec![n, n], vec!["j", "i"]);

        construct::insert_at(&mut ref_c, &mut nested_loops, "i");

        tracing_ri(&mut nested_loops, 8, 8);
    }

    #[test]
    fn test_combined() {
        // let n: usize = 10; // array dim
        // let ubound = n as i32; // loop bound
        // // let mut nested_loops = construct::nested_loops(&vec!["j", "i"], ubound);
        // let mut i_loop_ref = Node::new_single_loop("i", 0, ubound as i32);
        // let mut j_loop_ref = Node::new_single_loop("j", 0, ubound as i32);
        // let mut ref_c = Node::new_ref("A", vec![n], |ijk| {
        //     vec![ijk[0] as usize + ijk[1] as usize]
        // });

        // dace::insert_node(&mut j_loop_ref, &mut ref_c);
        // dace::insert_node(&mut i_loop_ref, &mut j_loop_ref);
        // // i_loop_ref.print_structure(0);

        // tracing_ri(&mut i_loop_ref, 8, 16);

        let n: usize = 8; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);

        let ref_a = Node::new_ref("C", vec![n], |ijk| {
            vec![(ijk[0] as usize / 2 + ijk[1] as usize / 2)]
        });

        let mut loop_order = vec![i_loop, j_loop];
        [ref_a.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 8);
    }

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
        tracing_ri(&mut nested_loops_top.clone(), 8, 40);

        let mut nested_loops_top = construct::nested_loops(&["j", "k", "i"], ubound);
        for a_ref in &mut refs {
            construct::insert_at(a_ref, &mut nested_loops_top, "i");
        }
        tracing_ri(&mut nested_loops_top.clone(), 8, 40);
    }

    #[test]
    fn test_tracing_ri2() {
        let n: usize = 16; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);

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

        let mut loop_order = vec![i_loop, j_loop, k_loop];
        [ref_a.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 64);
    }

    #[test]
    fn test_tracing_101() {
        let n = 10;
        let mut nest_loops = construct::nested_loops(&["i", "j", "k"], n as i32);
        let mut ref_c = construct::a_ref("ref_c", vec![n, n], vec!["i", "j"]);
        construct::insert_at(&mut ref_c, &mut nest_loops, "k");
        let mut ref_a = construct::a_ref("ref_a", vec![n, n], vec!["i", "k"]);
        construct::insert_at(&mut ref_a, &mut nest_loops, "k");
        let mut ref_b = construct::a_ref("ref_b", vec![n, n], vec!["k", "j"]);
        construct::insert_at(&mut ref_b, &mut nest_loops, "k");

        tracing_ri(&mut nest_loops.clone(), 1, 4);
    }

    #[test]
    fn higer_dim_loop_test() {
        let n = 6;
        let mut nest_loops = construct::nested_loops(&["i", "j", "k", "l", "m", "n"], n as i32);
        let mut ref_c = construct::a_ref("ref_c", vec![n, n], vec!["i", "j"]);
        construct::insert_at(&mut ref_c, &mut nest_loops, "n");
        let mut ref_a = construct::a_ref("ref_a", vec![n, n], vec!["i", "k"]);
        construct::insert_at(&mut ref_a, &mut nest_loops, "n");
        let mut ref_b = construct::a_ref("ref_b", vec![n, n], vec!["k", "j"]);
        construct::insert_at(&mut ref_b, &mut nest_loops, "n");
        let _ref_d = construct::a_ref("ref_d", vec![n, n, n], vec!["j", "l", "n"]);
        construct::insert_at(&mut ref_c, &mut nest_loops, "n");
        let _ref_e = construct::a_ref("ref_e", vec![n, n], vec!["i", "k"]);
        construct::insert_at(&mut ref_a, &mut nest_loops, "n");
        let _ref_f = construct::a_ref("ref_f", vec![n, n], vec!["k", "j"]);
        construct::insert_at(&mut ref_b, &mut nest_loops, "n");

        tracing_ri(&mut nest_loops.clone(), 4, 8);
    }

    #[test]
    fn test_tracing_101_2() {
        let n = 10;
        let mut nest_loops = construct::nested_loops(&["i", "j", "k"], n as i32);
        let ref_c = construct::a_ref("ref_c", vec![n, n], vec!["i", "j"]);
        let ref_a = construct::a_ref("ref_a", vec![n, n], vec!["i", "k"]);
        let ref_b = construct::a_ref("ref_b", vec![n, n], vec!["k", "j"]);

        for s in &mut [ref_c, ref_a, ref_b] {
            construct::insert_at(s, &mut nest_loops, "k");
        }

        tracing_ri(&mut nest_loops.clone(), 8, 40);
    }

    #[test]
    fn test_tracing_ri_101_imp() {
        let n: usize = 8; // array dim
        let ubound = n as i32; // loop bound
        let h_loop = Node::new_single_loop("h", 0, ubound);
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);

        let ref_a = Node::new_ref("C", vec![n, n], |ijk| {
            vec![(ijk[1] as usize + ijk[2] as usize) / 2, ijk[3] as usize]
        });

        let mut loop_order = vec![h_loop, i_loop, j_loop, k_loop];
        [ref_a.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 8);
    }

    #[test]
    fn test_tracing_ri_1010b() {
        let n: usize = 5; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let m_loop = Node::new_single_loop("m", 0, ubound);

        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[2] as usize, ijklm[4] as usize]
        });

        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 40);
    }

    #[test]
    fn test_tracing_ri_1010b_imperfect() {
        let n: usize = 2; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let mut m_loop = Node::new_single_loop("m", 0, ubound);

        let mut ref_o = Node::new_ref("Refo", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[2] as usize, ijklm[3] as usize]
        });
        Node::extend_loop_body(&mut m_loop, &mut ref_o);

        let ref_i = Node::new_ref("Refi", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[2] as usize, ijklm[4] as usize]
        });

        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop];
        [ref_i.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 8);
    }

    #[test]
    fn test_tracing_ri_10b01() {
        let n: usize = 5; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let m_loop = Node::new_single_loop("m", 0, ubound);

        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[4] as usize, ijklm[2] as usize]
        });

        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 8);
    }

    #[test]
    fn test_tracing_ri_b0101() {
        let n: usize = 5; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let m_loop = Node::new_single_loop("m", 0, ubound);

        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[2] as usize, ijklm[4] as usize, ijklm[0] as usize]
        });

        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 40);
    }

    #[test]
    fn test_tracing_ri4() {
        let n: usize = 3; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let m_loop = Node::new_single_loop("m", 0, ubound);
        let n_loop = Node::new_single_loop("n", 0, ubound);
        let o_loop = Node::new_single_loop("o", 0, ubound);

        let ref_3 = Node::new_ref("Ref3", vec![n, n, n, n], |ijklm| {
            vec![
                ijklm[0] as usize,
                ijklm[2] as usize,
                ijklm[4] as usize,
                ijklm[6] as usize,
            ]
        });

        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop, n_loop, o_loop];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 24);
    }

    #[test]
    fn test_tracing_ri_100101() {
        let n: usize = 8; // array dim
        let ubound = n as i32; // loop bound
        let i_loop = Node::new_single_loop("i", 0, ubound);
        let j_loop = Node::new_single_loop("j", 0, ubound);
        let k_loop = Node::new_single_loop("k", 0, ubound);
        let l_loop = Node::new_single_loop("l", 0, ubound);
        let m_loop = Node::new_single_loop("m", 0, ubound);
        let n_loop = Node::new_single_loop("n", 0, ubound);

        let ref_3 = Node::new_ref("Ref3", vec![n, n, n], |ijklm| {
            vec![ijklm[0] as usize, ijklm[3] as usize, ijklm[5] as usize]
        });

        let mut loop_order = vec![i_loop, j_loop, k_loop, l_loop, m_loop, n_loop];
        [ref_3.clone()]
            .iter_mut()
            .for_each(|s| Node::extend_loop_body(loop_order.last_mut().unwrap(), s));

        let nested_loops_top = construct::nest_the_loops(loop_order);

        tracing_ri(&mut nested_loops_top.clone(), 8, 16);
    }
}
