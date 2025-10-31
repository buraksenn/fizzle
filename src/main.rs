use std::io;

use fizzle::repl;

fn main() {
    env_logger::init();
    println!("Hello, this is the Monkey programming language!");
    println!("Feel free to type in commands");

    repl::start(io::stdin().lock(), io::stdout());
}
