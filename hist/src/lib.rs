use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Hist {
    pub hist: HashMap<Option<usize>, usize>,
    // attrs: HashMap<String,String>
}

impl Hist {
    pub fn new() -> Hist {
        Self::default()
    }

    pub fn add_dist(&mut self, d: Option<usize>) {
        self.hist
            .entry(d)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    pub fn to_vec(&self) -> Vec<(Option<usize>, usize)> {
        let mut hvec: Vec<_> = self.hist.iter().map(|(&k, &v)| (k, v)).collect();
        hvec.sort_by(|a, b| a.0.cmp(&b.0));
        hvec
    }
}

impl fmt::Display for Hist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut hvec = self.to_vec();
        let tot = hvec.iter().fold(0, |acc, x| acc + x.1);
        if !hvec.is_empty() {
            let min = hvec[0].0;
            let max = hvec.last().unwrap().0;
            writeln!(f, "Reuse distance histogram:\n\t{} distance value(s), min {:?}, max {:?}\n\t{} accesses", hvec.len(), min, max, tot)?;
            if let Some((None, count)) = hvec.first() {
                // writeln!(f, "\t({} cold accesses)", count)?;
                hvec.push((None, *count));
                hvec.remove(0);
            }
            writeln!(f, "value, count")?;
            hvec.into_iter().try_fold((), |_, (d, cnt)| {
                writeln!(
                    f,
                    "{}, {}",
                    d.map_or_else(|| "Cold".to_string(), |v| v.to_string()),
                    cnt
                )
            })
        } else {
            writeln!(f, "Reuse distance histogram is empty")?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hist_empty() {
        let h = Hist::new(); // Create an empty Hist

        // Check that to_vec returns an empty vector
        let v = h.to_vec();
        assert!(v.is_empty(), "Expected an empty vector, got {:?}", v);

        // Check that the Display implementation doesn't panic and returns an expected string
        let display = format!("{}", h);
        assert_eq!(
            display, "Reuse distance histogram is empty\n",
            "Unexpected output from Display implementation: {}",
            display
        );
    }

    #[test]
    fn hist_basic_operations() {
        let mut h = Hist::new();
        h.add_dist(None);
        h.add_dist(Some(1));
        h.add_dist(Some(1));
        h.add_dist(Some(100));

        let v = h.to_vec();
        println!("{:?}", v);
        assert_eq!(v[1], (Some(1), 2));
        assert_eq!(v[2], (Some(100), 1));
        assert_eq!(v[0], (None, 1));

        assert_eq!(format!("{}", h), "Reuse distance histogram:\n\t3 distance value(s), min None, max Some(100)\n\t4 accesses\nvalue, count\n1, 2\n100, 1\nCold, 1\n");

        // use cargo test -- --show-output to see the result
        println!("{}", h);
    }
}
