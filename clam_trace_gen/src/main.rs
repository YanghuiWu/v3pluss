#![feature(get_mut_unchecked)]

mod trace;

fn main() {
    trace::trace_polybench("mvt", 512, 64, &[]);
}
