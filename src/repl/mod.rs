use std::io::{BufRead, Write};

use crate::evaluator::evaluate;

const PROMPT: &str = ">> ";

pub fn start<R: BufRead, W: Write>(mut input: R, mut output: W) {
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
            Ok(node) => match evaluate(node) {
                Ok(obj) => writeln!(output, "{}", obj.inspect()).unwrap(),
                Err(e) => writeln!(output, "Error: {}", e).unwrap(),
            },
            Err(e) => writeln!(output, "Error: {}", e).unwrap(),
        }
    }
}
