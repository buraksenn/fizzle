use crate::{
    ast::{Expression, Node, Program, Statement},
    object::Object,
};

type EvaluatorResult<T> = Result<T, anyhow::Error>;

pub fn evaluate(n: Node) -> EvaluatorResult<Object> {
    match n {
        Node::Expression(exp) => evaluate_expression(&exp),
        Node::Program(p) => evaluate_program(&p),
        Node::Statement(st) => evaluate_statement(&st),
    }
}

fn evaluate_program(prog: &Program) -> EvaluatorResult<Object> {
    let mut result = Object::Null;
    for st in prog.statements.iter() {
        result = evaluate_statement(st)?;
    }

    Ok(result)
}

fn evaluate_statement(st: &Statement) -> EvaluatorResult<Object> {
    match st {
        Statement::Expression { value } => evaluate_expression(value),
        _ => todo!(),
    }
}

fn evaluate_expression(exp: &Expression) -> EvaluatorResult<Object> {
    match exp {
        Expression::IntegerLiteral(i) => Ok(Object::Integer(*i)),
        _ => todo!(),
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

        let mut it = tests.iter();
        while let Some(case) = it.next() {
            let obj = eval(case.input);
            assert_integer_object(obj, case.expected);
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
}
