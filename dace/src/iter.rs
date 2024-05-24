use std::iter::Iterator;
use std::rc::Rc;

use crate::ast::*;
use crate::ast::*;

///This struct is used to implement a custom iterator. It contains a stack field, which is a vector of tuples. Each tuple contains a Node and a usize representing the number of children of the node that have been visited.
pub struct Walk {
    // usize is the current body statement index, if there is any
    stack: Vec<(Rc<Node>, usize)>,
}

impl Walk {
    /// It takes a reference to a Node and initializes the stack with a tuple containing the Node and 0 (indicating no children have been visited yet).
    pub fn new(root: &Rc<Node>) -> Self {
        Walk {
            stack: vec![(root.clone(), 0)],
        }
        // Iter{ stack: vec![(root, root.loop_only( |lp| {
        //     if lp.body.borrow().len() > 0 { Some(0) } else { None } }))] }
    }

    /// This checks the last node in the stack and its visited count. Depending on the type of the Node (Loop, Ref, Block, or Branch), it updates the stack and returns the Node if it's being visited for the first time. Steps through the AST nodes, returning the next node.
    fn step(&mut self) -> Option<Rc<Node>> {
        let (node, visited) = self.stack.pop()?;
        let result = if visited == 0 {
            Some(node.clone())
        } else {
            None
        };
        // Visit the node for the first time

        match &node.as_ref().stmt {
            Stmt::Loop(loop_stmt) => {
                if visited < loop_stmt.body.len() {
                    self.stack.push((node.clone(), visited + 1));
                    self.stack.push((loop_stmt.body[visited].clone(), 0));
                }
            }
            Stmt::Branch(branch) => {
                if visited == 1 && branch.else_body.is_some() {
                    self.stack.push((node.clone(), visited + 1));
                    self.stack
                        .push((branch.else_body.as_ref().unwrap().clone(), 0));
                } else if visited == 0 {
                    self.stack.push((node.clone(), visited + 1));
                    self.stack.push((branch.then_body.clone(), 0));
                }
            }
            _ => {} // AryRef (pop) or Block (handled in the same way as loop but without .body)
        }
        result
    }
}

impl Iterator for Walk {
    type Item = Rc<Node>;

    /// The next method uses the step function to advance the iterator until a Node is found or the stack is empty.
    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            if let Some(x) = self.step() {
                return Some(x);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn loop_a_0() {
        // i = 0, n { a[0] }
        let mut aref = Node::new_ref("A", vec![1], |_| vec![0]);
        let mut aloop = Node::new_single_loop("i", 0, 10);
        Node::extend_loop_body(&mut aloop, &mut aref);

        let awalk = Walk::new(&aloop);
        // for stmt in awalk {
        //     println!("Thia is a - {:?}\n", stmt.as_ref().stmt.stmt_type());
        // }
        assert_eq!(awalk.fold(0, |cnt, _stmt| cnt + 1), 2);
    }

    #[test]
    fn loop_ij() {
        // i = 0, 1, {j = 0, 0 n { a[0] }; b[0]
        let mut aref = Node::new_ref("A", vec![1], |_| vec![0]);
        let mut jloop = Node::new_single_loop("j", 0, 10);
        Node::extend_loop_body(&mut jloop, &mut aref);
        let mut bref = Node::new_ref("B", vec![1], |_| vec![0]);
        let mut iloop = Node::new_single_loop("i", 0, 1);
        Node::extend_loop_body(&mut iloop, &mut jloop);
        Node::extend_loop_body(&mut iloop, &mut bref);
        let awalk = Walk::new(&iloop);
        // for stmt in awalk {
        //     println!("Thia is a - {:?}\n", stmt.as_ref().stmt.stmt_type());
        // }
        assert_eq!(awalk.fold(0, |cnt, _stmt| cnt + 1), 4);
    }

    #[test]
    fn simple_node() {
        // Simple single node
        let node = Node::new_ref("A", vec![1], |_| vec![0]);
        let awalk = Walk::new(&node);
        assert_eq!(awalk.fold(0, |cnt, _stmt| cnt + 1), 1);
    }

    #[test]
    fn nested_loops() {
        // Nested loops: i { j { k { a[0] } } }
        let mut aref = Node::new_ref("A", vec![1], |_| vec![0]);
        let mut kloop = Node::new_single_loop("k", 0, 1);
        Node::extend_loop_body(&mut kloop, &mut aref);
        let mut jloop = Node::new_single_loop("j", 0, 1);
        Node::extend_loop_body(&mut jloop, &mut kloop);
        let mut iloop = Node::new_single_loop("i", 0, 1);
        Node::extend_loop_body(&mut iloop, &mut jloop);

        let awalk = Walk::new(&iloop);
        assert_eq!(awalk.fold(0, |cnt, _stmt| cnt + 1), 4);
    }
}
