use std::io::{BufRead, Write};

use crate::lexer::Lexer;
use crate::token::Token;

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

        let mut lexer = Lexer::new(&line);
        loop {
            let tok = lexer.next_token();
            if matches!(tok, Token::Eof) {
                break;
            }
            writeln!(output, "{:?}", tok).unwrap();
        }
    }
}
