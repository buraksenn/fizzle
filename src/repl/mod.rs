use std::io::{BufRead, Write};

use crate::lexer::Lexer;
use crate::parser::Parser;

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

        let mut parser = Parser::new(Lexer::new(&line));
        match parser.parse_program() {
            Ok(program) => writeln!(output, "{}", program).unwrap(),
            Err(e) => writeln!(output, "Error: {}", e).unwrap(),
        }
    }
}
