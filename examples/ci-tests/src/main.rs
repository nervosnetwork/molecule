use molecule::prelude::*;

pub mod types {
    include!(concat!(env!("OUT_DIR"), "/", "ci_tests", ".rs"));
}

fn main() {
    let data = types::AllInOne::default();
    println!("Data Display:\n{}\n", data);
    println!("Data Debug:\n{:?}\n", data);
    println!("Data Length = {}", data.as_slice().len());
}
