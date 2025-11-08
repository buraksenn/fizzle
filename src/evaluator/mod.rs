use anyhow::anyhow;

use crate::{
    ast::{Expression, Node, Program, Statement},
    object::Object,
    token::{self, Token},
};

type EvaluatorResult = Result<Object, anyhow::Error>;

pub fn evaluate(n: Node) -> EvaluatorResult {
    match n {
        Node::Expression(exp) => evaluate_expression(&exp),
        Node::Program(p) => evaluate_program(&p),
        Node::Statement(st) => evaluate_statement(&st),
    }
}

fn evaluate_program(prog: &Program) -> EvaluatorResult {
    let mut result = Object::Null;
    for st in prog.statements.iter() {
        result = evaluate_statement(st)?;
    }

    Ok(result)
}

fn evaluate_statement(st: &Statement) -> EvaluatorResult {
    match st {
        Statement::Expression { value } => evaluate_expression(value),
        _ => todo!(),
    }
}

fn evaluate_expression(exp: &Expression) -> EvaluatorResult {
    match exp {
        Expression::IntegerLiteral(i) => Ok(Object::Integer(*i)),
        Expression::Boolean(b) => Ok(Object::Boolean(*b)),
        Expression::Prefix { operator, operand } => {
            let right = evaluate_expression(operand)?;
            evaluate_prefix_expression(operator, right)
        }
        _ => todo!(),
    }
}

fn evaluate_prefix_expression(operator: &Token, operand: Object) -> EvaluatorResult {
    match operator {
        Token::Bang => evaluate_bang_expression(operand),
        tok => Err(anyhow!(
            "expected ! but got unsupported {} token while evaluation prefix expression",
            tok
        )),
    }
}

fn evaluate_bang_expression(operand: Object) -> EvaluatorResult {
    match operand {
        Object::Boolean(false) | Object::Null => Ok(Object::Boolean(true)),
        _ => Ok(Object::Boolean(false)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{object::Object, parser::parse};

    #[test]
    fn eval_integer_expression() {
        struct Test<'a> {
            input: &'a str,
            expected: i64,
        }
        let tests = [
            Test {
                input: "5",
                expected: 5,
            },
            Test {
                input: "10",
                expected: 10,
            },
        ];

        for case in tests.iter() {
            let obj = eval(case.input);
            assert_integer_object(obj, case.expected);
        }
    }

    #[test]
    fn eval_boolean_expression() {
        struct Test<'a> {
            input: &'a str,
            expected: bool,
        }
        let tests = vec![
            Test {
                input: "true",
                expected: true,
            },
            Test {
                input: "false",
                expected: false,
            },
        ];

        for case in tests.iter() {
            let obj = eval(case.input);
            assert_boolean_object(obj, case.expected);
        }
    }

    #[test]
    fn test_bang_operator() {
        struct Test<'a> {
            input: &'a str,
            expected: bool,
        }
        let tests = vec![
            Test {
                input: "!true",
                expected: false,
            },
            Test {
                input: "!false",
                expected: true,
            },
            Test {
                input: "!5",
                expected: false,
            },
            Test {
                input: "!!true",
                expected: true,
            },
            Test {
                input: "!!false",
                expected: false,
            },
            Test {
                input: "!!5",
                expected: true,
            },
        ];

        for t in tests {
            let evaluated = eval(t.input);
            assert_boolean_object(evaluated, t.expected);
        }
    }

    fn eval(input: &str) -> Object {
        let node = parse(input).unwrap();

        evaluate(node).unwrap()
    }

    fn assert_integer_object(left: Object, r: i64) {
        match left {
            Object::Integer(i) => assert_eq!(i, r),
            x => panic!("expected integer but got {}", x),
        }
    }

    fn assert_boolean_object(left: Object, r: bool) {
        match left {
            Object::Boolean(b) => assert_eq!(b, r),
            x => panic!("expected integer but got {}", x),
        }
    }
}
