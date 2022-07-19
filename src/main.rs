#[macro_use]
extern crate lazy_static;

mod core;
mod cli;

fn main() {
    cli::mainloop().unwrap();
}
