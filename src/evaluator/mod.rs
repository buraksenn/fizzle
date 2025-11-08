use anyhow::anyhow;

use crate::{
    ast::{BlockStatement, Expression, Node, Program, Statement},
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
        Expression::Infix {
            left,
            operator,
            right,
        } => {
            let l = evaluate_expression(left)?;
            let r = evaluate_expression(right)?;
            evaluate_infix_expression(l, operator, r)
        }
        Expression::If(exp) => {
            let cond = evaluate_expression(&exp.condition)?;
            if is_truthy(&cond) {
                return evaluate_block_statement(&exp.consequence);
            } else {
                match exp.alternative {
                    Some(ref alt) => evaluate_block_statement(alt),
                    None => Ok(Object::Null),
                }
            }
        }
        _ => todo!(),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Boolean(false) | Object::Null => false,
        _ => true,
    }
}

fn evaluate_block_statement(block: &BlockStatement) -> EvaluatorResult {
    let mut result = Object::Null;
    for st in block.statements.iter() {
        result = evaluate_statement(st)?
    }

    Ok(result)
}

fn evaluate_infix_expression(left: Object, operator: &Token, right: Object) -> EvaluatorResult {
    match (left, right) {
        (Object::Integer(l), Object::Integer(r)) => match operator {
            Token::Minus => Ok(Object::Integer(l - r)),
            Token::Plus => Ok(Object::Integer(l + r)),
            Token::Slash => Ok(Object::Integer(l / r)),
            Token::Asterisk => Ok(Object::Integer(l * r)),
            Token::Gt => Ok(Object::Boolean(l > r)),
            Token::Lt => Ok(Object::Boolean(l < r)),
            Token::Eq => Ok(Object::Boolean(l == r)),
            Token::Neq => Ok(Object::Boolean(l != r)),
            _ => Err(anyhow!(
                "unsupported integer infix expression: {}",
                operator
            ))?,
        },
        (Object::Boolean(l), Object::Boolean(r)) => match operator {
            Token::Eq => Ok(Object::Boolean(l == r)),
            Token::Neq => Ok(Object::Boolean(l != r)),
            _ => Err(anyhow!(
                "unsupported boolean infix expression: {}",
                operator
            ))?,
        },
        (_, _) => Err(anyhow!("unsupported infix expressions")),
    }
}

fn evaluate_prefix_expression(operator: &Token, operand: Object) -> EvaluatorResult {
    match operator {
        Token::Bang => evaluate_bang_expression(operand),
        Token::Minus => evaluate_minus_expression(operand),
        tok => Err(anyhow!(
            "expected !/- but got unsupported {} token while evaluation prefix expression",
            tok
        )),
    }
}

fn evaluate_minus_expression(operand: Object) -> EvaluatorResult {
    match operand {
        Object::Integer(i) => Ok(Object::Integer(-i)),
        obj => Err(anyhow!(
            "expected integer for minus expression, got: {}",
            obj
        )),
    }
}

fn evaluate_bang_expression(operand: Object) -> EvaluatorResult {
    Ok(Object::Boolean(!is_truthy(&operand)))
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
            Test {
                input: "-5",
                expected: -5,
            },
            Test {
                input: "-10",
                expected: -10,
            },
            Test {
                input: "5 + 5 + 5 + 5 - 10",
                expected: 10,
            },
            Test {
                input: "2 * 2 * 2 * 2 * 2",
                expected: 32,
            },
            Test {
                input: "-50 + 100 + -50",
                expected: 0,
            },
            Test {
                input: "5 * 2 + 10",
                expected: 20,
            },
            Test {
                input: "5 + 2 * 10",
                expected: 25,
            },
            Test {
                input: "20 + 2 * -10",
                expected: 0,
            },
            Test {
                input: "50 / 2 * 2 + 10",
                expected: 60,
            },
            Test {
                input: "2 * (5 + 10)",
                expected: 30,
            },
            Test {
                input: "3 * 3 * 3 + 10",
                expected: 37,
            },
            Test {
                input: "3 * (3 * 3) + 10",
                expected: 37,
            },
            Test {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
                expected: 50,
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
            Test {
                input: "1 < 2",
                expected: true,
            },
            Test {
                input: "1 > 2",
                expected: false,
            },
            Test {
                input: "1 < 1",
                expected: false,
            },
            Test {
                input: "1 > 1",
                expected: false,
            },
            Test {
                input: "1 == 1",
                expected: true,
            },
            Test {
                input: "1 != 1",
                expected: false,
            },
            Test {
                input: "1 == 2",
                expected: false,
            },
            Test {
                input: "1 != 2",
                expected: true,
            },
            Test {
                input: "true == true",
                expected: true,
            },
            Test {
                input: "false == false",
                expected: true,
            },
            Test {
                input: "true == false",
                expected: false,
            },
            Test {
                input: "true != false",
                expected: true,
            },
            Test {
                input: "false != true",
                expected: true,
            },
            Test {
                input: "(1 < 2) == true",
                expected: true,
            },
            Test {
                input: "(1 < 2) == false",
                expected: false,
            },
            Test {
                input: "(1 > 2) == true",
                expected: false,
            },
            Test {
                input: "(1 > 2) == false",
                expected: true,
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

    #[test]
    fn if_else_expressions() {
        struct Test<'a> {
            input: &'a str,
            expected: Object,
        }
        let tests = vec![
            Test {
                input: "if (true) { 10 }",
                expected: Object::Integer(10),
            },
            Test {
                input: "if (false) { 10 }",
                expected: Object::Null,
            },
            Test {
                input: "if (1) { 10 }",
                expected: Object::Integer(10),
            },
            Test {
                input: "if (1 < 2) { 10 }",
                expected: Object::Integer(10),
            },
            Test {
                input: "if (1 > 2) { 10 }",
                expected: Object::Null,
            },
            Test {
                input: "if (1 > 2) { 10 } else { 20 }",
                expected: Object::Integer(20),
            },
            Test {
                input: "if (1 < 2) { 10 } else { 20 }",
                expected: Object::Integer(10),
            },
        ];

        for t in tests {
            let evaluated = eval(t.input);

            match t.expected {
                Object::Integer(i) => assert_integer_object(evaluated, i),
                _ => assert_null_object(evaluated),
            }
        }
    }

    fn eval(input: &str) -> Object {
        let node = parse(input).unwrap();

        evaluate(node).unwrap()
    }

    fn assert_null_object(obj: Object) {
        match obj {
            Object::Null => {}
            x => panic!("expected null but got {}", x),
        }
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
