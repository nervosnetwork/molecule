use molecule::prelude::*;

use molecule_ci_tests::types;

fn main() {
    let data = types::AllInOne::default();
    println!("Data Display:\n{}\n", data);
    println!("Data Debug:\n{:?}\n", data);
    println!("Data Length = {}", data.as_slice().len());
}
