// Dhruv Pai helped a lot with this file.

#![allow(dead_code, non_snake_case)]

use std::rc::Rc;

use dace::ast::Node;
use dace::ast::Stmt;
use dace::{
    a_ref, branch_node, create_loops, generate_subscript, insert_at, insert_at_innermost,
    loop_body, loop_node, nested_loops,
};

pub fn lu(n: usize) -> Rc<Node> {
    let ref_a_ij = a_ref("A", vec![n, n], vec!["i", "j"]);
    let ref_a_ik = a_ref("A", vec![n, n], vec!["i", "k"]);
    let ref_a_kj = a_ref("A", vec![n, n], vec!["k", "j"]);
    let ref_a_jj = a_ref("A", vec![n, n], vec!["j", "j"]);

    let mut Aik_Akj_Aij_Aij = [
        ref_a_ik.clone(),
        ref_a_kj.clone(),
        ref_a_ij.clone(),
        ref_a_ij.clone(),
    ];
    let mut Ajj_Aij_Aij = [ref_a_jj.clone(), ref_a_ij.clone(), ref_a_ij.clone()];

    let ubound = n as i32;

    let loop_indices = vec!["i", "j", "k"];
    let bounds = |loop_index| generate_subscript(&loop_indices, loop_index);

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    let mut j_loop_lower_ref = loop_node!("j", 0 => bounds("i"));
    let mut k_loop_ref_j = loop_node!("k", 0 => bounds("k"));
    let mut j_loop_upper_ref = loop_node!("j", bounds("i") => ubound);
    let mut k_loop_ref_i = loop_node!("k", 0 => bounds("i"));

    loop_body(&[&mut i_loop_ref, &mut j_loop_lower_ref, &mut k_loop_ref_j]);

    for node in Aik_Akj_Aij_Aij.iter_mut() {
        insert_at(node, &mut k_loop_ref_j, "k");
    }

    for node in Ajj_Aij_Aij.iter_mut() {
        insert_at(node, &mut j_loop_lower_ref, "j");
    }

    loop_body(&[&mut i_loop_ref, &mut j_loop_upper_ref, &mut k_loop_ref_i]);

    for node in Aik_Akj_Aij_Aij.iter_mut() {
        insert_at(node, &mut k_loop_ref_i, "k");
    }

    i_loop_ref
}

pub fn lu_affine(n: usize) -> Rc<Node> {
    let ref_a_ij = a_ref("A", vec![n, n], vec!["i", "j"]);
    let ref_a_ik = a_ref("A", vec![n, n], vec!["i", "k"]);
    let ref_a_kj = a_ref("A", vec![n, n], vec!["k", "j"]);
    let ref_a_jj = a_ref("A", vec![n, n], vec!["j", "j"]);

    let mut Aik_Akj_Aij_Aij = [
        ref_a_ik.clone(),
        ref_a_kj.clone(),
        ref_a_ij.clone(),
        ref_a_ij.clone(),
    ];
    let mut Ajj_Aij_Aij = [ref_a_jj.clone(), ref_a_ij.clone(), ref_a_ij.clone()];

    let ubound = n as i32;

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    let mut j_loop_lower_ref = loop_node!("j", 0 => (vec![1, 0, 0], 0));
    let mut k_loop_ref_j = loop_node!("k", 0 => (vec![0, 1, 0], 0));
    let mut j_loop_upper_ref = loop_node!("j", (vec![1, 0, 0], 0) => ubound);
    let mut k_loop_ref_i = loop_node!("k", 0 => (vec![1, 0, 0], 0));

    loop_body(&[&mut i_loop_ref, &mut j_loop_lower_ref, &mut k_loop_ref_j]);

    for node in Aik_Akj_Aij_Aij.iter_mut() {
        insert_at(node, &mut k_loop_ref_j, "k");
    }

    for node in Ajj_Aij_Aij.iter_mut() {
        insert_at(node, &mut j_loop_lower_ref, "j");
    }

    loop_body(&[&mut i_loop_ref, &mut j_loop_upper_ref, &mut k_loop_ref_i]);

    for node in Aik_Akj_Aij_Aij.iter_mut() {
        insert_at(node, &mut k_loop_ref_i, "k");
    }

    i_loop_ref
}

pub fn trmm_trace(M: usize, N: usize) -> Rc<Node> {
    let mut i_loop_ref = Node::new_single_loop("i", 0, M as i32);
    let mut j_loop_ref = Node::new_single_loop("j", 0, N as i32);
    let mut k_loop_ref =
        Node::new_single_loop("k", Node::get_lb(&i_loop_ref).unwrap() + 1, M as i32);

    insert_at(&mut j_loop_ref, &mut i_loop_ref, "i");

    // B[i * N + j] = alpha * B[i * N + j];
    let mut B3_ref = a_ref("B", vec![M, N], vec!["i", "j"]);
    insert_at(&mut B3_ref, &mut j_loop_ref, "j");

    insert_at(&mut k_loop_ref, &mut j_loop_ref, "j");

    // B[i * N + j] += A[k * M + i] * B[k * N + j];
    let mut A_B1_B2_B3 = [
        a_ref("A", vec![N, M], vec!["k", "i"]),
        a_ref("B", vec![M, N], vec!["k", "j"]),
        a_ref("B", vec![M, N], vec!["i", "j"]),
        a_ref("B", vec![M, N], vec!["i", "j"]),
    ];
    for node in A_B1_B2_B3.iter_mut() {
        insert_at(node, &mut k_loop_ref, "k");
    }

    i_loop_ref
}

pub fn mvt(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = 10;

    let mut i_loop_ref = nested_loops(&["i", "j"], ubound);
    let mut m_loop_ref = nested_loops(&["m", "k"], ubound);

    // creating x1[i] = x1[i] + a[i][j] * y1[j];
    let s_ref_x1: Rc<Node> = a_ref("x1", vec![n], vec!["i"]);
    let s_ref_a1 = a_ref("a1", vec![n, n], vec!["i", "j"]);
    let s_ref_y1 = a_ref("y1", vec![n], vec!["j"]);
    let mut x1_a1_y1_x1 = [
        s_ref_x1.clone(),
        s_ref_a1.clone(),
        s_ref_y1.clone(),
        s_ref_x1.clone(),
    ];
    for node in x1_a1_y1_x1.iter_mut() {
        insert_at_innermost(node, &mut i_loop_ref);
    }

    //x2[i] = x2[i] + a[j][i] * y2[j];
    let s_ref_x2: Rc<Node> = a_ref("x2", vec![n], vec!["m"]);
    let s_ref_a2 = a_ref("a2", vec![n, n], vec!["k", "m"]);
    let s_ref_y2 = a_ref("y2", vec![n], vec!["k"]);
    let mut x2_a2_y2_x2 = [
        s_ref_x2.clone(),
        s_ref_a2.clone(),
        s_ref_y2.clone(),
        s_ref_x2.clone(),
    ];
    for node in x2_a2_y2_x2.iter_mut() {
        insert_at_innermost(node, &mut m_loop_ref);
    }

    // combine two seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_ref, m_loop_ref]))
}

pub fn trisolv(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    let mut j_loop_ref = Node::new_single_loop_dyn_ub("j", 0, move |i| i[0]);

    // creating x[i] = b[i];
    let s_ref_x1 = a_ref("x", vec![n], vec!["i"]);
    let s_ref_b = a_ref("b", vec![n], vec!["i"]);
    let mut b_x1 = [s_ref_b.clone(), s_ref_x1.clone()];
    for node in b_x1.iter_mut() {
        insert_at(node, &mut i_loop_ref, "i");
    }

    insert_at(&mut j_loop_ref, &mut i_loop_ref, "i");

    // creating x[i] -= L[i][j] * x[j];
    let s_ref_L1 = a_ref("L", vec![n, n], vec!["i", "j"]);
    let s_ref_x2 = a_ref("x", vec![n], vec!["j"]);
    let s_ref_x3 = a_ref("x", vec![n], vec!["i"]);
    let mut L1_x2_x3_x3 = [
        s_ref_L1.clone(),
        s_ref_x2.clone(),
        s_ref_x3.clone(),
        s_ref_x3.clone(),
    ];
    for node in L1_x2_x3_x3.iter_mut() {
        insert_at(node, &mut j_loop_ref, "j");
    }

    // creating x[i] = x[i] / L[i][i]
    let s_ref_L2 = a_ref("L", vec![n, n], vec!["i", "i"]);
    let mut x1_L2_x1 = [s_ref_x1.clone(), s_ref_L2.clone(), s_ref_x1.clone()];
    for node in x1_L2_x1.iter_mut() {
        insert_at(node, &mut i_loop_ref, "i");
    }

    i_loop_ref
}

pub fn syrk(n: usize, m: usize) -> Rc<Node> {
    // n,m are array dimensions
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    let mut i_loop_ref = nested_loops(&["i", "j"], ubound1);
    let mut k_loop_ref = nested_loops(&["k", "l"], ubound1);
    let mut m_loop_ref = Node::new_single_loop("m", 0, ubound2);

    //creating C[i][j] = C[i][j] * beta
    let mut s_ref_c1 = a_ref("c", vec![n, n], vec!["i", "j"]);
    for _ in 0..2 {
        insert_at_innermost(&mut s_ref_c1, &mut i_loop_ref);
    }

    insert_at_innermost(&mut m_loop_ref, &mut k_loop_ref);

    // creating C[i][j] = C[i][j] + alpha * A[i][k] * A[j][k]
    let s_ref_a1 = a_ref("a1", vec![n, m], vec!["k", "m"]);
    let s_ref_a2 = a_ref("a2", vec![n, m], vec!["l", "m"]);
    let s_ref_c2 = a_ref("c", vec![n, n], vec!["k", "l"]);
    let mut a1_a2_c2_c2 = [
        s_ref_a1.clone(),
        s_ref_a2.clone(),
        s_ref_c2.clone(),
        s_ref_c2.clone(),
    ];
    for node in a1_a2_c2_c2.iter_mut() {
        insert_at(node, &mut m_loop_ref, "m");
    }

    // combine two seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_ref, k_loop_ref]))
}

pub fn syr2d(n: usize, m: usize) -> Rc<Node> {
    // n,m are array dimensions
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound1);
    let mut j_loop_ref = loop_node!("j", 0 => |i : &[i32]| i[0]);
    let mut k_loop_ref = Node::new_single_loop("k", 0, ubound2);
    let mut l_loop_ref = loop_node!("l", 0 => |i : &[i32]| i[0]);

    insert_at(&mut j_loop_ref, &mut i_loop_ref, "i");

    // creating C[i][j] *= beta;
    let mut s_ref_c = a_ref("c", vec![n, n], vec!["i", "j"]);
    for _ in 0..2 {
        insert_at(&mut s_ref_c, &mut j_loop_ref, "j");
    }

    loop_body(&[&mut i_loop_ref, &mut k_loop_ref, &mut l_loop_ref]);

    // creating C[i][j] += A[j][k]*alpha*B[i][k] + B[j][k]*alpha*A[i][k];
    let a1_b1__b2_a2 = [("a1", "b1"), ("b2", "a2")];
    for names in a1_b1__b2_a2.iter() {
        Node::extend_loop_body(
            &mut l_loop_ref,
            &mut Node::new_ref(names.0, vec![n, m], |ijkl| {
                vec![ijkl[3] as usize, ijkl[2] as usize]
            }),
        );
        Node::extend_loop_body(
            &mut l_loop_ref,
            &mut Node::new_ref(names.1, vec![n, m], |ijkl| {
                vec![ijkl[0] as usize, ijkl[2] as usize]
            }),
        );
    }
    let c1_c2 = ["c1", "c2"];
    for name in c1_c2.iter() {
        Node::extend_loop_body(
            &mut l_loop_ref,
            &mut Node::new_ref(name, vec![n, n], |ijkl| {
                vec![ijkl[0] as usize, ijkl[3] as usize]
            }),
        );
    }

    i_loop_ref
}

pub fn gemm(n: usize) -> Rc<Node> {
    let A0 = Node::new_ref("A", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let B0 = Node::new_ref("B", vec![n, n], |ijk| {
        vec![ijk[2] as usize, ijk[1] as usize]
    });
    let C0 = Node::new_ref("C", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    let ubound = n as i32;

    let i_j_k_loops = create_loops(&["i", "j", "k"], 0, ubound);
    let (mut i_loop_ref, mut j_loop_ref, mut k_loop_ref) = (
        i_j_k_loops[0].clone(),
        i_j_k_loops[1].clone(),
        i_j_k_loops[2].clone(),
    );

    insert_at(&mut j_loop_ref, &mut i_loop_ref, "i");

    for _ in 0..2 {
        Node::extend_loop_body(&mut j_loop_ref, &mut C0.clone());
    }

    insert_at(&mut k_loop_ref, &mut j_loop_ref, "j");

    let mut A0_B0_C0_C0 = [A0.clone(), B0.clone(), C0.clone(), C0.clone()];
    for node in A0_B0_C0_C0.iter_mut() {
        Node::extend_loop_body(&mut k_loop_ref, node)
    }

    i_loop_ref
}

pub fn _3mm(NI: usize, NJ: usize, NK: usize, NL: usize, NM: usize) -> Rc<Node> {
    let s_ref_e = a_ref("e", vec![NI, NJ], vec!["i", "j"]);
    let s_ref_a = a_ref("a", vec![NI, NK], vec!["i", "k"]);
    let s_ref_b = a_ref("b", vec![NK, NJ], vec!["k", "j"]);

    let mut ini_loop_ref1 = Node::new_single_loop("i", 0, NI as i32);
    let mut jnj_loop_ref1 = Node::new_single_loop("j", 0, NJ as i32);
    let mut knk_loop_ref = Node::new_single_loop("k", 0, NK as i32);

    insert_at(&mut jnj_loop_ref1, &mut ini_loop_ref1, "i");

    insert_at(&mut s_ref_e.clone(), &mut jnj_loop_ref1, "j");

    insert_at(&mut knk_loop_ref, &mut jnj_loop_ref1, "j");

    let mut a_b_e = [s_ref_a, s_ref_b, s_ref_e];

    for node in a_b_e.iter_mut() {
        insert_at(node, &mut knk_loop_ref, "k");
    }

    let s_ref_f = a_ref("f", vec![NJ, NL], vec!["i", "j"]);
    let s_ref_c = a_ref("c", vec![NJ, NM], vec!["i", "k"]);
    let s_ref_d = a_ref("d", vec![NM, NL], vec!["k", "j"]);

    let mut inj_loop_ref = Node::new_single_loop("i", 0, NJ as i32);
    let mut jnl_loop_ref = Node::new_single_loop("j", 0, NL as i32);
    let mut knm_loop_ref = Node::new_single_loop("k", 0, NM as i32);

    insert_at(&mut jnl_loop_ref, &mut inj_loop_ref, "i");

    insert_at(&mut s_ref_f.clone(), &mut jnl_loop_ref, "j");

    insert_at(&mut knm_loop_ref, &mut jnl_loop_ref, "j");

    let mut c_d_f = [s_ref_c, s_ref_d, s_ref_f];

    for node in c_d_f.iter_mut() {
        insert_at(node, &mut knm_loop_ref, "k");
    }

    let s_ref_g = a_ref("g", vec![NI, NL], vec!["i", "j"]);
    let s_ref_e_2 = a_ref("e", vec![NI, NJ], vec!["i", "k"]);
    let s_ref_f_2 = a_ref("f", vec![NJ, NL], vec!["k", "j"]);

    let mut ini_loop_ref2 = Node::new_single_loop("i", 0, NI as i32);
    let mut jnl_loop_ref2 = Node::new_single_loop("j", 0, NL as i32);
    let mut knj_loop_ref = Node::new_single_loop("k", 0, NJ as i32);

    insert_at(&mut jnl_loop_ref2, &mut ini_loop_ref2, "i");

    insert_at(&mut s_ref_g.clone(), &mut jnl_loop_ref2, "j");

    insert_at(&mut knj_loop_ref, &mut jnl_loop_ref2, "j");

    let mut e2_f2_g = [s_ref_e_2, s_ref_f_2, s_ref_g];

    for node in e2_f2_g.iter_mut() {
        insert_at(node, &mut knj_loop_ref, "k");
    }

    Node::new_node(Stmt::Block(vec![
        ini_loop_ref1,
        inj_loop_ref,
        ini_loop_ref2,
    ]))
}

pub fn _2mm(NI: usize, NJ: usize, NK: usize, NL: usize) -> Rc<Node> {
    let s_ref_tmp = a_ref("tmp", vec![NI, NJ], vec!["i", "j"]);
    let s_ref_a = a_ref("a", vec![NI, NK], vec!["i", "k"]);
    let s_ref_b = a_ref("b", vec![NK, NJ], vec!["k", "j"]);
    let s_ref_c = a_ref("c", vec![NL, NJ], vec!["k", "j"]);
    let s_ref_d = a_ref("d", vec![NI, NL], vec!["i", "j"]);

    let mut ini_loop_ref1 = Node::new_single_loop("i", 0, NI as i32);
    let mut jnj_loop_ref = Node::new_single_loop("j", 0, NJ as i32);
    let knk_loop_ref = Node::new_single_loop("k", 0, NK as i32);
    let mut knk_loop_ref_clone = knk_loop_ref.clone();

    insert_at(&mut jnj_loop_ref, &mut ini_loop_ref1, "i");

    insert_at(&mut s_ref_tmp.clone(), &mut jnj_loop_ref, "j");

    insert_at(&mut knk_loop_ref_clone, &mut jnj_loop_ref, "j");

    let mut a_b_tmp = [s_ref_a.clone(), s_ref_b.clone(), s_ref_tmp.clone()];

    for node in a_b_tmp.iter_mut() {
        insert_at(node, &mut knk_loop_ref_clone, "k");
    }

    let mut ini_loop_ref2 = Node::new_single_loop("i", 0, NI as i32);
    let mut jnl_loop_ref = Node::new_single_loop("j", 0, NL as i32);
    let mut knj_loop_ref = Node::new_single_loop("k", 0, NJ as i32);

    insert_at(&mut jnl_loop_ref, &mut ini_loop_ref2, "i");

    insert_at(&mut s_ref_d.clone(), &mut jnl_loop_ref, "j");

    insert_at(&mut knj_loop_ref, &mut jnl_loop_ref, "j");

    let mut tmp_c_d = [s_ref_tmp.clone(), s_ref_c.clone(), s_ref_d.clone()];

    for node in tmp_c_d.iter_mut() {
        insert_at(node, &mut knj_loop_ref, "k");
    }

    Node::new_node(Stmt::Block(vec![ini_loop_ref1, ini_loop_ref2]))
}

pub fn cholesky(n: usize) -> Rc<Node> {
    let ubound = n as i32;

    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound);
    let mut j_loop_ref = Node::new_single_loop_dyn_ub("j", 0, move |i| i[0]);
    let mut k1_loop_ref = Node::new_single_loop_dyn_ub("k", 0, move |j| j[0]);
    let mut k2_loop_ref = Node::new_single_loop_dyn_ub("k", 0, move |i| i[0]);

    loop_body(&[&mut i_loop_ref, &mut j_loop_ref, &mut k1_loop_ref]);

    //create A[i * N + j] -= A[i * N + k] * A[j * N + k];
    let s_ref_aij1 = a_ref("a", vec![n, n], vec!["i", "j"]);
    let s_ref_aik1 = a_ref("a", vec![n, n], vec!["i", "k"]);
    let s_ref_ajk = a_ref("a", vec![n, n], vec!["j", "k"]);
    let mut aik1_ajk_aij1_aij1 = [
        s_ref_aik1.clone(),
        s_ref_ajk.clone(),
        s_ref_aij1.clone(),
        s_ref_aij1.clone(),
    ];
    for node in aik1_ajk_aij1_aij1.iter_mut() {
        insert_at(node, &mut k1_loop_ref, "k");
    }

    // create A[i * N + j] /= A[j * N + j];
    let s_ref_aij2 = a_ref("a", vec![n, n], vec!["i", "j"]);
    let s_ref_ajj = a_ref("a", vec![n], vec!["j"]);
    let mut ajj_aij2_aij2 = [s_ref_ajj.clone(), s_ref_aij2.clone(), s_ref_aij2.clone()];
    for node in ajj_aij2_aij2.iter_mut() {
        insert_at(node, &mut j_loop_ref, "j");
    }

    insert_at(&mut k2_loop_ref, &mut i_loop_ref, "i");

    //create A[i * N + i] -= A[i * N + k] * A[i * N + k];
    let s_ref_aii1 = Node::new_ref("a", vec![n], |ijk| vec![ijk[0] as usize]);
    let s_ref_aik2 = Node::new_ref("a", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let mut aik2_aik2_aii1_aii1 = [
        s_ref_aik2.clone(),
        s_ref_aik2.clone(),
        s_ref_aii1.clone(),
        s_ref_aii1.clone(),
    ];
    for node in aik2_aik2_aii1_aii1.iter_mut() {
        Node::extend_loop_body(&mut k2_loop_ref, node);
    }

    //create A[i * N + i] = sqrt(A[i * N + i]);
    let mut s_ref_aii2 = a_ref("a", vec![n], vec!["i"]);
    for _ in 0..2 {
        insert_at(&mut s_ref_aii2, &mut i_loop_ref, "i");
    }

    i_loop_ref
}

pub fn gramschmidt_trace(n: usize, m: usize) -> Rc<Node> {
    //nrm += A[i * N + k] * A[i * N + k];
    let s_ref_a1 = Node::new_ref("a1", vec![n], move |ki| {
        vec![ki[1] as usize * n + ki[0] as usize]
    });

    //R[k * N + k] = sqrt(nrm);
    let s_ref_r1 = Node::new_ref("r1", vec![n], move |k| {
        vec![k[0] as usize * n + k[0] as usize]
    });

    //Q[i * N + k] = A[i * N + k] / R[k * N + k];
    let s_ref_a2 = Node::new_ref("a2", vec![n], move |ki| {
        vec![ki[1] as usize * n + ki[0] as usize]
    });
    let s_ref_r1_copy = Node::new_ref("r1", vec![n], move |ki| {
        vec![ki[0] as usize * n + ki[0] as usize]
    });
    let s_ref_q1 = Node::new_ref("q1", vec![n], move |ki| {
        vec![ki[1] as usize * n + ki[0] as usize]
    });

    //R[k * N + j] = 0.0;
    let s_ref_r2 = Node::new_ref("r2", vec![n], move |kj| {
        vec![kj[0] as usize * n + kj[1] as usize]
    });

    //R[k * N + j] += Q[i * N + k] * A[i * N + j];
    let s_ref_q2 = Node::new_ref("q2", vec![n], move |kji| {
        vec![kji[2] as usize * n + kji[0] as usize]
    });
    let s_ref_a3 = Node::new_ref("a3", vec![n], move |kji| {
        vec![kji[2] as usize * n + kji[1] as usize]
    });
    let s_ref_r3 = Node::new_ref("r3", vec![n], move |kji| {
        vec![kji[0] as usize * n + kji[1] as usize]
    });
    //insert r3 clone here in this order

    //A[i * N + j] = A[i * N + j] - Q[i * N + k] * R[k * N + j];
    //insert a3 clone here in this order
    //insert q2 clone here in this order
    //insert r3 clone here in this order
    //insert a3 clone here in this order

    let mut k_loop_ref = Node::new_single_loop("k", 0, n as i32);
    let i_loops = create_loops(&["i", "i", "i", "i"], 0, m as i32);
    let (mut i_loop_ref, mut i_loop_ref2, mut i_loop_ref3, mut i_loop_ref4) = (
        i_loops[0].clone(),
        i_loops[1].clone(),
        i_loops[2].clone(),
        i_loops[3].clone(),
    );
    let mut j_loop_ref = loop_node!("j", |v : &[i32]| v[0] + 1 => n as i32);

    insert_at(&mut i_loop_ref, &mut k_loop_ref, "k");

    for _ in 0..2 {
        Node::extend_loop_body(&mut i_loop_ref, &mut s_ref_a1.clone());
    }

    Node::extend_loop_body(&mut k_loop_ref, &mut s_ref_r1.clone());

    insert_at(&mut i_loop_ref2, &mut k_loop_ref, "k");

    let mut a2_r1copy_q1 = [s_ref_a2.clone(), s_ref_r1_copy.clone(), s_ref_q1.clone()];

    for node in a2_r1copy_q1.iter_mut() {
        Node::extend_loop_body(&mut i_loop_ref2, node);
    }

    insert_at(&mut j_loop_ref, &mut k_loop_ref, "k");

    Node::extend_loop_body(&mut j_loop_ref, &mut s_ref_r2.clone());

    insert_at(&mut i_loop_ref3, &mut j_loop_ref, "j");

    let mut q2_a3_r3_r3 = [
        s_ref_q2.clone(),
        s_ref_a3.clone(),
        s_ref_r3.clone(),
        s_ref_r3.clone(),
    ];

    for node in q2_a3_r3_r3.iter_mut() {
        Node::extend_loop_body(&mut i_loop_ref3, node);
    }

    insert_at(&mut i_loop_ref4, &mut j_loop_ref, "j");

    let mut a3_q2_r3_a3 = [
        s_ref_a3.clone(),
        s_ref_q2.clone(),
        s_ref_r3.clone(),
        s_ref_a3.clone(),
    ];

    for node in a3_q2_r3_a3.iter_mut() {
        Node::extend_loop_body(&mut i_loop_ref4, node);
    }

    k_loop_ref
}

pub fn heat_3d(m: usize, n: usize) -> Rc<Node> {
    // n: usize is array dim
    let ubound = n as i32; // loop bound
    let tsteps = m as i32; // steps bound

    let mut t_loop_ref = Node::new_single_loop("t", 0, tsteps);

    let i1_j1_k1 = create_loops(&["i_1", "j_1", "k_1"], 0, ubound);
    let (mut i_loop_ref_1, mut j_loop_ref_1, mut k_loop_ref_1) = (
        i1_j1_k1[0].clone(),
        i1_j1_k1[1].clone(),
        i1_j1_k1[2].clone(),
    );
    loop_body(&[
        &mut t_loop_ref,
        &mut i_loop_ref_1,
        &mut j_loop_ref_1,
        &mut k_loop_ref_1,
    ]);

    let s_ref_a_1 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize + 1, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_a_2 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_a_3 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize - 1, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_a_4 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize + 1, ijk[2] as usize]
    });
    let s_ref_a_5 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_a_6 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize - 1, ijk[2] as usize]
    });
    let s_ref_a_7 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize + 1]
    });
    let s_ref_a_8 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_a_9 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize - 1]
    });
    let s_ref_a_10 = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });

    let s_ref_b = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });

    let mut a1_a2_a3_a4_a5_a6_a7_a8_a9_a10_b = [
        s_ref_a_1, s_ref_a_2, s_ref_a_3, s_ref_a_4, s_ref_a_5, s_ref_a_6, s_ref_a_7, s_ref_a_8,
        s_ref_a_9, s_ref_a_10, s_ref_b,
    ];
    for node in a1_a2_a3_a4_a5_a6_a7_a8_a9_a10_b.iter_mut() {
        Node::extend_loop_body(&mut k_loop_ref_1, node);
    }

    let i2_j2_k2 = create_loops(&["i_2", "j_2", "k_2"], 0, ubound);
    let (mut i_loop_ref_2, mut j_loop_ref_2, mut k_loop_ref_2) = (
        i2_j2_k2[0].clone(),
        i2_j2_k2[1].clone(),
        i2_j2_k2[2].clone(),
    );
    loop_body(&[
        &mut t_loop_ref,
        &mut i_loop_ref_2,
        &mut j_loop_ref_2,
        &mut k_loop_ref_2,
    ]);

    let s_ref_b_1 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize + 1, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_b_2 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_b_3 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize - 1, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_b_4 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize + 1, ijk[2] as usize]
    });
    let s_ref_b_5 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_b_6 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize - 1, ijk[2] as usize]
    });
    let s_ref_b_7 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize + 1]
    });
    let s_ref_b_8 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_b_9 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize - 1]
    });
    let s_ref_b_10 = Node::new_ref("B", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });
    let s_ref_a = Node::new_ref("A", vec![n, n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize, ijk[2] as usize]
    });

    let mut b1_b2_b3_b4_b5_b6_b7_b8_b9_b10_a = [
        s_ref_b_1, s_ref_b_2, s_ref_b_3, s_ref_b_4, s_ref_b_5, s_ref_b_6, s_ref_b_7, s_ref_b_8,
        s_ref_b_9, s_ref_b_10, s_ref_a,
    ];
    for node in b1_b2_b3_b4_b5_b6_b7_b8_b9_b10_a.iter_mut() {
        Node::extend_loop_body(&mut k_loop_ref_2, node);
    }

    t_loop_ref
}

pub fn convolution_2d(ni: usize, nj: usize) -> Rc<Node> {
    let mut i_ni_loop_ref = Node::new_single_loop("i", 1, (ni - 1) as i32);
    let mut j_nj_loop_ref = Node::new_single_loop("j", 1, (nj - 1) as i32);

    insert_at(&mut j_nj_loop_ref, &mut i_ni_loop_ref, "i");

    let A_B = ["A", "B"];
    for name in A_B.iter() {
        let mut node = a_ref(name, vec![ni, nj], vec!["i", "j"]);
        insert_at(&mut node, &mut j_nj_loop_ref, "j");
    }

    Node::new_node(Stmt::Block(vec![i_ni_loop_ref]))
}

pub fn symm(n: usize, m: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound1 = n as i32;
    let ubound2 = m as i32;

    // creating loops
    let mut i_loop_ref = Node::new_single_loop("i", 0, ubound2);
    let mut j_loop_ref = Node::new_single_loop("j", 0, ubound1);
    let mut k_loop_ref = loop_node!("k", 0 => move |i : &[i32]| i[0]);

    loop_body(&[&mut i_loop_ref, &mut j_loop_ref, &mut k_loop_ref]);

    // creating c[k][j] += alpha * b[i][j] * a[i][k]
    let mut b1_a1_c1_c2 = [
        a_ref("b1", vec![m, n], vec!["i", "j"]),
        a_ref("a1", vec![m, m], vec!["i", "k"]),
        a_ref("c1", vec![m, n], vec!["k", "j"]),
        a_ref("c2", vec![m, n], vec!["k", "j"]),
    ];
    for node in b1_a1_c1_c2.iter_mut() {
        insert_at(node, &mut k_loop_ref, "k");
    }

    // creating tempt2 += b[k][j] * a[i][k]
    let mut b2_a2 = [
        a_ref("b2", vec![m, n], vec!["k", "j"]),
        a_ref("a2", vec![m, m], vec!["i", "k"]),
    ];
    for node in b2_a2.iter_mut() {
        insert_at(node, &mut k_loop_ref, "k");
    }

    // creating c[i][j] = beta * c[i][j] + alpha* b[i][j] * a[i][i] + alpha * temp2
    let mut c3_b3_a3_c4 = [
        a_ref("c3", vec![m, n], vec!["i", "j"]),
        a_ref("b3", vec![m, n], vec!["i", "j"]),
        a_ref("a3", vec![m, m], vec!["i", "i"]),
        a_ref("c4", vec![m, n], vec!["i", "j"]),
    ];
    for node in c3_b3_a3_c4.iter_mut() {
        insert_at(node, &mut j_loop_ref, "j");
    }

    i_loop_ref
}

pub fn stencil(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    // creating loops
    let mut i_loop_ref = nested_loops(&["i", "j"], ubound + 1);

    // creating b[i][j] =  a[i][j] + a[i][j] + a[i][j] + a[i][j] + a[i][j]
    let a1_a2_a3_a4_a5_b = ["a1", "a2", "a3", "a4", "a5", "b"];
    for name in a1_a2_a3_a4_a5_b.iter() {
        let mut node = a_ref(name, vec![n, n], vec!["i", "j"]);
        insert_at_innermost(&mut node, &mut i_loop_ref);
    }

    i_loop_ref
}

pub fn seidel_2d(m: usize, n: usize) -> Rc<Node> {
    let ubound = n as i32;
    let tsteps = m as i32;

    // creating loops
    let mut i_loop_ref = Node::new_single_loop("i", 0, tsteps - 1);
    let mut j_loop_ref = nested_loops(&["j", "k"], ubound - 2);

    insert_at(&mut j_loop_ref, &mut i_loop_ref, "i");

    // creating A[i][j] = A[i][j] + A[i][j] + A[i][j] + A[i][j] + A[i][j] + A[i][j] + A[i][j] + A[i][j] + A[i][j]
    let a1_a2_a3_a4_a5_a6_a7_a8_a9_a0 =
        ["a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "a0"];
    for name in a1_a2_a3_a4_a5_a6_a7_a8_a9_a0.iter() {
        let mut node = a_ref(name, vec![n, n], vec!["j", "k"]);
        insert_at_innermost(&mut node, &mut i_loop_ref);
    }

    i_loop_ref
}

pub fn ludcmp(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    let mut i_loop_upper = Node::new_single_loop("i", 0, ubound);
    let mut j_loop_upper = loop_node!("j", 0 => move |i : &[i32]| i[0]);
    let mut k_loop_upper = loop_node!("k", 0 => move |j : &[i32]| j[0]);
    let mut j_loop_lower = loop_node!("j", |i : &[i32]| i[0] => move |_: &_| ubound);
    let mut k_loop_lower = loop_node!("k", 0 => move |i : &[i32]| i[0]);

    insert_at(&mut j_loop_upper, &mut i_loop_upper, "i");

    // creating w = A[i][j]
    let mut s_ref_a1: Rc<Node> = a_ref("a1", vec![n, n], vec!["i", "j"]);
    insert_at(&mut s_ref_a1, &mut j_loop_upper, "j");

    insert_at(&mut k_loop_upper, &mut j_loop_upper, "j");

    // creating w -= A[i][k] * A[k][j]
    let mut a2_a3 = [
        a_ref("a2", vec![n, n], vec!["i", "k"]),
        a_ref("a3", vec![n, n], vec!["k", "j"]),
    ];
    for node in a2_a3.iter_mut() {
        insert_at(node, &mut k_loop_upper, "k");
    }

    // creating A[i][j] = w / A[j][j]
    let mut a4_a5 = [
        a_ref("a4", vec![n, n], vec!["j", "j"]),
        a_ref("a5", vec![n, n], vec!["i", "j"]),
    ];
    for node in a4_a5.iter_mut() {
        insert_at(node, &mut j_loop_upper, "j");
    }

    insert_at(&mut j_loop_lower, &mut i_loop_upper, "i");

    // creating w = A[i][j]
    let mut s_ref_a6 = a_ref("a6", vec![n, n], vec!["i", "j"]);
    insert_at(&mut s_ref_a6, &mut j_loop_lower, "j");

    insert_at(&mut k_loop_lower, &mut j_loop_lower, "j");

    // creating w -= A[i][k] * A[k][j]
    let mut a7_a8 = [
        a_ref("a7", vec![n, n], vec!["i", "k"]),
        a_ref("a8", vec![n, n], vec!["k", "j"]),
    ];
    for node in a7_a8.iter_mut() {
        insert_at(node, &mut k_loop_lower, "k");
    }

    // creating A[i][j] = w
    let mut s_ref_a9 = a_ref("a9", vec![n, n], vec!["i", "j"]);
    insert_at(&mut s_ref_a9, &mut j_loop_lower, "j");

    let mut i_loop_middle = Node::new_single_loop("i", 0, ubound);
    let mut j_loop2 = loop_node!("j", 0 => move |i : &[i32]| i[0]);

    // creating w = b[i]
    let mut s_ref_b1 = a_ref("b1", vec![n], vec!["i"]);
    insert_at(&mut s_ref_b1, &mut i_loop_middle, "i");

    insert_at(&mut j_loop2, &mut i_loop_middle, "i");

    // creating w -= A[i][j] * y[j]
    let mut a10_y1 = [
        a_ref("a10", vec![n, n], vec!["i", "j"]),
        a_ref("y1", vec![n], vec!["j"]),
    ];
    for node in a10_y1.iter_mut() {
        insert_at(node, &mut j_loop2, "j");
    }

    // creating y[i] = w
    let mut s_ref_y2 = a_ref("y2", vec![n], vec!["i"]);
    insert_at(&mut s_ref_y2, &mut i_loop_middle, "i");

    let mut i_loop_lower = loop_node!("i", ubound-1 => 0, step: |x| x - 1);
    let mut j_loop3 = loop_node!("j", |i : &[i32]| i[0] + 1 => move |_: &_| ubound);

    // creating w = y[i]
    let mut s_ref_y3 = a_ref("y3", vec![n], vec!["i"]);
    insert_at(&mut s_ref_y3, &mut i_loop_lower, "i");

    insert_at(&mut j_loop3, &mut i_loop_lower, "i");

    // creating w -= A[i][j] * x[j]
    let mut a11_x1 = [
        a_ref("a11", vec![n, n], vec!["i", "j"]),
        a_ref("x1", vec![n], vec!["j"]),
    ];
    for node in a11_x1.iter_mut() {
        insert_at(node, &mut j_loop3, "j");
    }

    // creating x[i] = w / A[i][i]
    let mut a12_x2 = [
        a_ref("a12", vec![n, n], vec!["i", "i"]),
        a_ref("x2", vec![n], vec!["i"]),
    ];
    for node in a12_x2.iter_mut() {
        insert_at(node, &mut i_loop_lower, "i");
    }

    // combine three seperate loops
    Node::new_node(Stmt::Block(vec![i_loop_upper, i_loop_middle, i_loop_lower]))
}

pub fn nussinov(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    // creating loops
    let mut i_loop = loop_node!("i", ubound-1 => 0, step: |x| x - 1);
    let mut j_loop = loop_node!("j", |i : &[i32]| i[0] + 1 => move |_: &_| ubound);
    let mut k_loop = loop_node!("k", |i : &[i32]| i[0] + 1 => move |j : &[i32]| j[0]);

    insert_at(&mut j_loop, &mut i_loop, "i");

    // creating table[i][j] = max_score(table[i][j], table[i][j-1])
    let s_ref_if1_t1 = Node::new_ref("if1_t1", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let s_ref_if1_t2 = Node::new_ref("if1_t2", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize - 1]
    });
    let s_ref_if1_t3 = Node::new_ref("if1_t3", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    // creating table[i][j] = max_score(table[i][j], table[i+1][j])
    let s_ref_if2_t1 = Node::new_ref("if2_t1", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let s_ref_if2_t2 = Node::new_ref("if2_t2", vec![n, n], |ijk| {
        vec![ijk[0] as usize + 1, ijk[1] as usize]
    });
    let s_ref_if2_t3 = Node::new_ref("if2_t3", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    // creating table[i][j] = max_score(table[i][j], table[i+1][j-1]+match(seq[i], seq[j]))
    let s_ref_if3_t1 = Node::new_ref("if3_t1", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let s_ref_if3_t2 = Node::new_ref("if3_t2", vec![n, n], |ijk| {
        vec![ijk[0] as usize + 1, ijk[1] as usize - 1]
    });
    let s_ref_if3_s1 = Node::new_ref("if3_s1", vec![n], |ijk| vec![ijk[0] as usize]);
    let s_ref_if3_s2 = Node::new_ref("if3_s2", vec![n], |ijk| vec![ijk[1] as usize]);
    let s_ref_if3_t3 = Node::new_ref("if3_t3", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    // creating table[i][j] = max_score(table[i][j], table[i+1][j-1])
    let s_ref_else3_t1 = Node::new_ref("else3_t1", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let s_ref_else3_t2 = Node::new_ref("else3_t2", vec![n, n], |ijk| {
        vec![ijk[0] as usize + 1, ijk[1] as usize - 1]
    });
    let s_ref_else3_t3 = Node::new_ref("else3_t3", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    // creating if else branches
    let q1 = Node::new_node(Stmt::Block(vec![s_ref_if1_t1, s_ref_if1_t2, s_ref_if1_t3]));
    let branch1 = branch_node! {
        if (|ijk| ijk[1] > 0) {
            //[s_ref_if1_t1, s_ref_if1_t2, s_ref_if1_t3]
            q1
        }
    };

    let q2 = Node::new_node(Stmt::Block(vec![s_ref_if2_t1, s_ref_if2_t2, s_ref_if2_t3]));
    let branch2 = branch_node! {
        if (move |ijk| ijk[0] + 1 < ubound) {
            //[s_ref_if2_t1, s_ref_if2_t2, s_ref_if2_t3]
            q2
        }
    };

    let q3 = Node::new_node(Stmt::Block(vec![
        s_ref_if3_t1,
        s_ref_if3_t2,
        s_ref_if3_s1,
        s_ref_if3_s2,
        s_ref_if3_t3,
    ]));
    let q4 = Node::new_node(Stmt::Block(vec![
        s_ref_else3_t1,
        s_ref_else3_t2,
        s_ref_else3_t3,
    ]));
    let branch3 = branch_node! {
        if (|ijk| ijk[0] < ijk[1] - 1) {
            //[s_ref_if3_t1, s_ref_if3_t2, s_ref_if3_s1, s_ref_if3_s2, s_ref_if3_t3]
            q3
        } else {
            //[s_ref_else3_t1, s_ref_else3_t2, s_ref_else3_t3]
            q4
        }
    };

    let branch4 = branch_node! {
        if (move |ijk| ijk[1] > 0 && ijk[0] + 1 < ubound) {
            branch3
        }
    };

    let mut branch1_branch2_branch4 = [branch1, branch2, branch4];
    for node in branch1_branch2_branch4.iter_mut() {
        Node::extend_loop_body(&mut j_loop, node);
    }

    insert_at(&mut k_loop, &mut j_loop, "j");

    // creating table[i][j] = max_score(table[i][j], table[i][k] + table[k+1][j])
    let s_ref_t1 = Node::new_ref("t1", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });
    let s_ref_t2 = Node::new_ref("t2", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[2] as usize]
    });
    let s_ref_t3 = Node::new_ref("t3", vec![n, n], |ijk| {
        vec![ijk[2] as usize + 1, ijk[1] as usize]
    });
    let s_ref_t4 = Node::new_ref("t4", vec![n, n], |ijk| {
        vec![ijk[0] as usize, ijk[1] as usize]
    });

    let mut t1_t2_t3_t4 = [s_ref_t1, s_ref_t2, s_ref_t3, s_ref_t4];
    for node in t1_t2_t3_t4.iter_mut() {
        Node::extend_loop_body(&mut k_loop, node);
    }

    i_loop
}

pub fn jacobi_1d(m: usize, n: usize) -> Rc<Node> {
    let ubound = n as i32;
    let tsteps = m as i32;

    let mut t_loop = Node::new_single_loop("t", 0, tsteps);
    let mut i_loop1 = Node::new_single_loop("i", 1, ubound - 1);
    let mut i_loop2 = Node::new_single_loop("i", 1, ubound - 1);

    Node::extend_loop_body(&mut t_loop, &mut i_loop1);

    // creating B[i] = 0.33333 * (A[i-1] + A[i] + A[i + 1]);
    let s_ref_a1 = Node::new_ref("a1", vec![n], |ti| vec![ti[1] as usize - 1]);
    let s_ref_a2 = Node::new_ref("a2", vec![n], |ti| vec![ti[1] as usize]);
    let s_ref_a3 = Node::new_ref("a3", vec![n], |ti| vec![ti[1] as usize + 1]);
    let s_ref_b1 = Node::new_ref("b1", vec![n], |ti| vec![ti[1] as usize]);
    let mut a1_a2_a3_b1 = [s_ref_a1, s_ref_a2, s_ref_a3, s_ref_b1];
    for node in a1_a2_a3_b1.iter_mut() {
        Node::extend_loop_body(&mut i_loop1, node);
    }

    Node::extend_loop_body(&mut t_loop, &mut i_loop2);

    // creating A[i] = 0.33333 * (B[i-1] + B[i] + B[i + 1]);
    let s_ref_b2 = Node::new_ref("b2", vec![n], |ti| vec![ti[1] as usize - 1]);
    let s_ref_b3 = Node::new_ref("b3", vec![n], |ti| vec![ti[1] as usize]);
    let s_ref_b4 = Node::new_ref("b4", vec![n], |ti| vec![ti[1] as usize + 1]);
    let s_ref_a4 = Node::new_ref("a4", vec![n], |ti| vec![ti[1] as usize]);
    let mut b2_b3_b4_a4 = [s_ref_b2, s_ref_b3, s_ref_b4, s_ref_a4];
    for node in b2_b3_b4_a4.iter_mut() {
        Node::extend_loop_body(&mut i_loop2, node);
    }

    t_loop
}

pub fn jacobi_2d(m: usize, n: usize) -> Rc<Node> {
    let ubound = n as i32;
    let tsteps = m as i32;

    let mut t_loop = Node::new_single_loop("t", 0, tsteps);
    let mut i_loop1 = Node::new_single_loop("j", 1, ubound - 1);
    let mut j_loop1 = Node::new_single_loop("j", 1, ubound - 1);
    let mut i_loop2 = Node::new_single_loop("j", 1, ubound - 1);
    let mut j_loop2 = Node::new_single_loop("j", 1, ubound - 1);

    loop_body(&[&mut t_loop, &mut i_loop1, &mut j_loop1]);

    // creating B[i][j] = 0.2 * (A[i][j] + A[i][j-1] + A[i][1+j] + A[(1+i)][j] + A[(i-1)][j]);
    let s_ref_a1 = Node::new_ref("a1", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize]
    });
    let s_ref_a2 = Node::new_ref("a2", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize - 1]
    });
    let s_ref_a3 = Node::new_ref("a3", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize + 1]
    });
    let s_ref_a4 = Node::new_ref("a4", vec![n, n], |tij| {
        vec![tij[1] as usize + 1, tij[2] as usize]
    });
    let s_ref_a5 = Node::new_ref("a5", vec![n, n], |tij| {
        vec![tij[1] as usize - 1, tij[2] as usize]
    });
    let s_ref_b1 = Node::new_ref("b1", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize]
    });
    let mut a1_a2_a3_a4_a5_b1 = [s_ref_a1, s_ref_a2, s_ref_a3, s_ref_a4, s_ref_a5, s_ref_b1];
    for node in a1_a2_a3_a4_a5_b1.iter_mut() {
        Node::extend_loop_body(&mut j_loop1, node);
    }

    loop_body(&[&mut t_loop, &mut i_loop2, &mut j_loop2]);

    // creating A[i][j] = 0.2 * (B[i][j] + B[i][j-1] + B[i][1+j] + B[(1+i)][j] + B[(i-1)][j]);
    let s_ref_b2 = Node::new_ref("b2", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize]
    });
    let s_ref_b3 = Node::new_ref("b3", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize - 1]
    });
    let s_ref_b4 = Node::new_ref("b4", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize + 1]
    });
    let s_ref_b5 = Node::new_ref("b5", vec![n, n], |tij| {
        vec![tij[1] as usize + 1, tij[2] as usize]
    });
    let s_ref_b6 = Node::new_ref("b6", vec![n, n], |tij| {
        vec![tij[1] as usize - 1, tij[2] as usize]
    });
    let s_ref_a6 = Node::new_ref("a6", vec![n, n], |tij| {
        vec![tij[1] as usize, tij[2] as usize]
    });
    let mut b2_b3_b4_b5_b6_a6 = [s_ref_b2, s_ref_b3, s_ref_b4, s_ref_b5, s_ref_b6, s_ref_a6];
    for node in b2_b3_b4_b5_b6_a6.iter_mut() {
        Node::extend_loop_body(&mut j_loop2, node);
    }

    t_loop
}

pub fn gesummv(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    let mut i_loop = Node::new_single_loop("i", 0, ubound);
    let mut j_loop = Node::new_single_loop("j", 0, ubound);

    // creating tmp[i] = 0 and y[i] = 0;
    let tmp1_y1 = ["tmp1", "y1"];
    for name in tmp1_y1.iter() {
        let mut node = a_ref(name, vec![n], vec!["i"]);
        insert_at(&mut node, &mut i_loop, "i");
    }

    insert_at(&mut j_loop, &mut i_loop, "i");

    // creating tmp[i] = A[i][j] * x[j] + tmp[i];
    // creating y[i] = B[i][j] * x[j] + y[i];
    let a_x1_tmp2_tmp3__b_x2_y2_y3 = [("a", "x1", "tmp2", "tmp3"), ("b", "x2", "y2", "y3")];
    for names in a_x1_tmp2_tmp3__b_x2_y2_y3.iter() {
        let mut nodes = [
            a_ref(names.0, vec![n, n], vec!["i", "j"]),
            a_ref(names.1, vec![n], vec!["j"]),
            a_ref(names.2, vec![n], vec!["i"]),
            a_ref(names.3, vec![n], vec!["i"]),
        ];
        for node in nodes.iter_mut() {
            insert_at(node, &mut j_loop, "j");
        }
    }

    // creating y[i] = alpha * tmp[i] + beta * y[i];
    let tmp4_y4_y5 = ["tmp4", "y4", "y5"];
    for name in tmp4_y4_y5.iter() {
        let mut node = a_ref(name, vec![n], vec!["i"]);
        insert_at(&mut node, &mut i_loop, "i");
    }

    i_loop
}

pub fn gemver(n: usize) -> Rc<Node> {
    // n : usize is size of array
    let ubound = n as i32;

    let mut i_loop1 = nested_loops(&["i", "j"], ubound);

    // creating A[i][j] = A[i][j] + u1[i] * v1[j] + u2[i] * v2[j]
    let mut a1_u1_v1_u2_v2_a2 = [
        a_ref("a1", vec![n, n], vec!["i", "j"]),
        a_ref("u1", vec![n], vec!["i"]),
        a_ref("v1", vec![n], vec!["j"]),
        a_ref("u2", vec![n], vec!["i"]),
        a_ref("v2", vec![n, n], vec!["j"]),
        a_ref("a2", vec![n, n], vec!["i", "j"]),
    ];
    for node in a1_u1_v1_u2_v2_a2.iter_mut() {
        insert_at_innermost(node, &mut i_loop1);
    }

    let mut i_loop2 = nested_loops(&["i", "j"], ubound);

    // creating x[i] = x[i] + beta * A[j][i] * y[j]
    let mut x1_a3_y_x2 = [
        a_ref("x1", vec![n], vec!["i"]),
        a_ref("a3", vec![n, n], vec!["j", "i"]),
        a_ref("y", vec![n], vec!["j"]),
        a_ref("x2", vec![n], vec!["i"]),
    ];
    for node in x1_a3_y_x2.iter_mut() {
        insert_at_innermost(node, &mut i_loop2);
    }

    let mut i_loop3 = Node::new_single_loop("i", 0, ubound);

    // creating x[i] = x[i] + z[i]
    let x3_z_x4 = ["x3", "z", "x4"];
    for name in x3_z_x4.iter() {
        let mut node = a_ref(name, vec![n], vec!["i"]);
        insert_at(&mut node, &mut i_loop3, "i");
    }

    let mut i_loop4 = nested_loops(&["i", "j"], ubound);

    // creating w[i] = w[i] + alpha * A[i][j] * x[j]
    let mut w1_a4_x5_w2 = [
        a_ref("w1", vec![n], vec!["i"]),
        a_ref("a4", vec![n, n], vec!["i", "j"]),
        a_ref("x5", vec![n], vec!["j"]),
        a_ref("w2", vec![n, n], vec!["i"]),
    ];
    for node in w1_a4_x5_w2.iter_mut() {
        insert_at_innermost(node, &mut i_loop4);
    }

    Node::new_node(Stmt::Block(vec![i_loop1, i_loop2, i_loop3, i_loop4]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trmm_trace_test() {
        assert_eq!(trmm_trace(1024, 1024).node_count(), 8);
        trmm_trace(1024, 1024).print_structure(0);
    }

    #[test]
    fn test_mvt() {
        assert_eq!(mvt(1024).node_count(), 13);
        mvt(1024).print_structure(0);
    }

    #[test]
    fn test_trisolv() {
        assert_eq!(trisolv(1024).node_count(), 11);
        trisolv(1024).print_structure(0);
    }

    #[test]
    fn test_syrk() {
        assert_eq!(syrk(256, 256).node_count(), 12);
        syrk(256, 256).print_structure(0);
    }

    #[test]
    fn test_syr2d() {
        assert_eq!(syr2d(1024, 1024).node_count(), 12);
        syr2d(1024, 1024).print_structure(0);
    }

    #[test]
    fn test_gemm_rd_olken() {
        use std::time::Instant;
        let mut trace = gemm(128);
        let start = Instant::now();
        // let hist = static_rd::trace::trace(&mut trace);
        let result = static_rd::trace::trace(&mut trace, static_rd::LRUSplay::new());
        let hist = result.0;
        let end = Instant::now();
        println!("gemm trace time: {:?}", end - start);
        println!("hist: {}", hist);
    }

    #[test]
    fn test_gemm_rd_scale_tree() {
        use std::time::Instant;
        let mut trace = gemm(128);
        let start = Instant::now();
        // let hist = static_rd::trace::trace(&mut trace);
        let hist = static_rd::trace::trace(&mut trace, static_rd::LRUScaleTree::new(0.1, 10000)).0;
        let end = Instant::now();
        println!("gemm trace time: {:?}", end - start);
        println!("hist: {}", hist);
    }

    #[test]
    fn _2mm_test() {
        assert_eq!(_2mm(1024, 1024, 1024, 1024).node_count(), 15);
        _2mm(1024, 1024, 1024, 1024).print_structure(0);
    }

    #[test]
    fn lu_test() {
        let mm = lu(100);
        assert_eq!(mm.node_count(), 16);
        mm.print_structure(0);
    }

    #[test]
    fn test_cholesky() {
        assert_eq!(cholesky(1024).node_count(), 17);
        cholesky(1024).print_structure(0);
    }

    #[test]
    fn gram_schmidt_test() {
        assert_eq!(gramschmidt_trace(1024, 1024).node_count(), 21);
        gramschmidt_trace(1024, 1024).print_structure(0);
    }

    #[test]
    fn heat_3d_test() {
        let mm = heat_3d(5, 100);
        assert_eq!(mm.node_count(), 29);
        // mm.print_structure(0);
    }

    #[test]
    fn test_symm() {
        assert_eq!(symm(1024, 1024).node_count(), 13);
        symm(1024, 1024).print_structure(0);
    }

    #[test]
    fn test_stencil() {
        assert_eq!(stencil(1024).node_count(), 8);
        stencil(1024).print_structure(0);
    }

    #[test]
    fn test_seidel_2d() {
        assert_eq!(seidel_2d(10, 1024).node_count(), 13);
        seidel_2d(10, 1024).print_structure(0);
    }

    #[test]
    fn test_ludcmp() {
        assert_eq!(ludcmp(1024).node_count(), 28);
    }

    #[test]
    fn test_nussinov() {
        assert_eq!(nussinov(1024).node_count(), 29)
    }

    #[test]
    fn test_jacobi_1d() {
        assert_eq!(jacobi_1d(10, 1024).node_count(), 11)
    }

    #[test]
    fn test_jacobi_2d() {
        assert_eq!(jacobi_2d(10, 1024).node_count(), 17)
    }

    #[test]
    fn test_gesummv() {
        assert_eq!(gesummv(1024).node_count(), 15)
    }

    #[test]
    fn test_gemver() {
        gemver(1024).print_structure(0);
        assert_eq!(gemver(1024).node_count(), 25)
    }
}
