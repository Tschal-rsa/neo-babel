#[macro_use]
extern crate lazy_static;

#[macro_use]
mod core;
mod cli;

fn main() {
    let mut interface = cli::Cli::new();
    interface.mainloop();
}
