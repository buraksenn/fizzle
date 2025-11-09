use std::{
    cell::RefCell,
    io::{BufRead, Write},
    rc::Rc,
};

use crate::{evaluator::evaluate, object::environment::Environment};

const PROMPT: &str = ">> ";

pub fn start<R: BufRead, W: Write>(mut input: R, mut output: W) {
    let env = Rc::new(RefCell::new(Environment::new()));

    loop {
        // print prompt
        write!(output, "{PROMPT}").unwrap();
        output.flush().unwrap();

        let mut line = String::new();
        let bytes_read = input.read_line(&mut line).unwrap();
        if bytes_read == 0 {
            // EOF or input closed
            return;
        }

        match crate::parser::parse(&line) {
            Ok(node) => match evaluate(node, env.clone()) {
                Ok(obj) => writeln!(output, "{}", obj.as_ref().inspect()).unwrap(),
                Err(e) => writeln!(output, "Error: {}", e).unwrap(),
            },
            Err(e) => writeln!(output, "Error: {}", e).unwrap(),
        }
    }
}
