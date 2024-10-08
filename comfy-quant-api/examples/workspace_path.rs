use std::env;

fn main() {
    let path = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("path: {}", path);
}
