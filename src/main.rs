mod city;
mod vec2d;

use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use fastrand::Rng;

fn info_center(msg: impl fmt::Display, width: usize) {
    println!("{:^w$}", msg, w = width);
}

fn main() {
    let (width, height) = (80, 60);
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    info_center("oO0OoO0OoO0Oo CiTY oO0OoO0OoO0Oo", width);
    info_center(format_args!("seed: {}", seed), width);
    println!();

    let rng = Rng::with_seed(seed);

}
