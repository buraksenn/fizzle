use std::{cell::RefCell, rc::Rc};

use anyhow::{Ok, anyhow};

use crate::{
    ast::{BlockStatement, Expression, Node, Program, Statement},
    object::{
        builtin,
        environment::Environment,
        object::{Function, Object},
    },
    token::Token,
};

type EvaluatorResult = Result<Rc<Object>, anyhow::Error>;

pub fn evaluate(n: Node, env: Rc<RefCell<Environment>>) -> EvaluatorResult {
    match n {
        Node::Expression(exp) => evaluate_expression(&exp, env),
        Node::Program(p) => evaluate_program(&p, env),
        Node::Statement(st) => evaluate_statement(&st, env),
    }
}

fn evaluate_program(prog: &Program, env: Rc<RefCell<Environment>>) -> EvaluatorResult {
    let mut result = Rc::new(Object::Null);
    for st in prog.statements.iter() {
        result = evaluate_statement(st, env.clone())?;
        if let Object::Return(val) = result.as_ref() {
            return Ok(Rc::new((**val).clone()));
        }
    }

    Ok(result)
}

fn evaluate_statement(st: &Statement, env: Rc<RefCell<Environment>>) -> EvaluatorResult {
    match st {
        Statement::Expression { value } => evaluate_expression(value, env),
        Statement::Return { value } => {
            let exp = evaluate_expression(value, env)?;
            Ok(Rc::new(Object::Return(Rc::new((*exp).clone()))))
        }
        Statement::Let { name, value } => evaluate_let_statement(name, value, env),
    }
}

fn evaluate_let_statement(
    name: &String,
    value: &Expression,
    env: Rc<RefCell<Environment>>,
) -> EvaluatorResult {
    let exp = evaluate_expression(value, env.clone())?;
    env.borrow_mut().set(name.clone(), exp.clone());
    Ok(exp)
}

fn evaluate_expression(exp: &Expression, env: Rc<RefCell<Environment>>) -> EvaluatorResult {
    match exp {
        Expression::IntegerLiteral(i) => Ok(Rc::new(Object::Integer(*i))),
        Expression::Boolean(b) => Ok(Rc::new(Object::Boolean(*b))),
        Expression::Identifier(s) => env
            .borrow()
            .get(s)
            .or_else(|| builtin::from_str(&s).map(|b| Rc::new(Object::Builtin(b))))
            .ok_or_else(|| anyhow!("identifier {} does not exist", s)),
        Expression::StringLiteral(s) => Ok(Rc::new(Object::String(s.clone()))),
        Expression::Array(literals) => Ok(Rc::new(Object::Array(Rc::new(evaluate_expressions(
            literals,
            Rc::clone(&env),
        )?)))),
        Expression::Prefix { operator, operand } => {
            let right = evaluate_expression(operand, env)?;
            evaluate_prefix_expression(operator, right)
        }
        Expression::Infix {
            left,
            operator,
            right,
        } => {
            let l = evaluate_expression(left, env.clone())?;
            let r = evaluate_expression(right, env)?;
            evaluate_infix_expression(l, operator, r)
        }
        Expression::If(exp) => {
            let cond = evaluate_expression(&exp.condition, env.clone())?;
            if is_truthy(cond.as_ref()) {
                evaluate_block_statement(&exp.consequence, env)
            } else {
                match exp.alternative {
                    Some(ref alt) => evaluate_block_statement(alt, env),
                    None => Ok(Rc::new(Object::Null)),
                }
            }
        }
        Expression::Function(func) => Ok(Rc::new(Object::Function(Box::new(Function {
            parameters: func.parameters.clone(),
            body: func.body.clone(),
            env: Rc::clone(&env),
        })))),
        Expression::Call(call) => {
            let evaluated_func = evaluate_expression(&call.function, Rc::clone(&env))?;
            let args = evaluate_expressions(&call.arguments, env)?;

            match evaluated_func.as_ref() {
                Object::Function(f) => apply_function(&f, args),
                Object::Builtin(b) => b(args),
                obj => Err(anyhow!("expected function, got {}", obj)),
            }
        }
        Expression::Index(idx) => {
            let left = evaluate_expression(&idx.left, Rc::clone(&env))?;
            let index = evaluate_expression(&idx.index, Rc::clone(&env))?;

            match (&*left, &*index) {
                (Object::Array(a), Object::Integer(i)) => match &a.get(*i as usize) {
                    Some(v) => Ok(Rc::clone(v)),
                    None => Ok(Rc::new(Object::Null)),
                },
                (l, i) => Err(anyhow!(
                    "expected array and integer pair but got {} and {}",
                    l,
                    i
                )),
            }
        }
    }
}

fn evaluate_expressions(
    exps: &Vec<Expression>,
    env: Rc<RefCell<Environment>>,
) -> Result<Vec<Rc<Object>>, anyhow::Error> {
    let mut objs = Vec::with_capacity(exps.len());

    for e in exps {
        let res = evaluate_expression(&e, Rc::clone(&env))?;
        objs.push(res);
    }

    Ok(objs)
}

fn apply_function(f: &Function, args: Vec<Rc<Object>>) -> EvaluatorResult {
    let extended = extend_function_env(f, &args);
    let evaluated = evaluate_block_statement(&f.body, extended)?;

    if let Object::Return(v) = &*evaluated {
        return Ok(Rc::clone(v));
    }
    Ok(evaluated)
}

fn extend_function_env(func: &Function, args: &Vec<Rc<Object>>) -> Rc<RefCell<Environment>> {
    let env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
        &func.env,
    ))));

    let mut args_iter = args.into_iter();

    for param in &func.parameters {
        let arg = args_iter.next().unwrap();

        env.borrow_mut().set(param.clone(), Rc::clone(arg))
    }

    env
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Boolean(false) | Object::Null => false,
        _ => true,
    }
}

fn evaluate_block_statement(
    block: &BlockStatement,
    env: Rc<RefCell<Environment>>,
) -> EvaluatorResult {
    let mut result = Rc::new(Object::Null);
    for st in block.statements.iter() {
        result = evaluate_statement(st, env.clone())?;
        if let Object::Return(_) = result.as_ref() {
            return Ok(result);
        }
    }

    Ok(result)
}

fn evaluate_infix_expression(
    left: Rc<Object>,
    operator: &Token,
    right: Rc<Object>,
) -> EvaluatorResult {
    match (left.as_ref(), right.as_ref()) {
        (Object::Integer(l), Object::Integer(r)) => match operator {
            Token::Minus => Ok(Rc::new(Object::Integer(l - r))),
            Token::Plus => Ok(Rc::new(Object::Integer(l + r))),
            Token::Slash => Ok(Rc::new(Object::Integer(l / r))),
            Token::Asterisk => Ok(Rc::new(Object::Integer(l * r))),
            Token::Gt => Ok(Rc::new(Object::Boolean(l > r))),
            Token::Lt => Ok(Rc::new(Object::Boolean(l < r))),
            Token::Eq => Ok(Rc::new(Object::Boolean(l == r))),
            Token::Neq => Ok(Rc::new(Object::Boolean(l != r))),
            _ => Err(anyhow!(
                "unsupported integer infix expression: {}",
                operator
            ))?,
        },
        (Object::Boolean(l), Object::Boolean(r)) => match operator {
            Token::Eq => Ok(Rc::new(Object::Boolean(l == r))),
            Token::Neq => Ok(Rc::new(Object::Boolean(l != r))),
            _ => Err(anyhow!(
                "unsupported boolean infix expression: {}",
                operator
            ))?,
        },
        (Object::String(l), Object::String(r)) => match operator {
            Token::Plus => Ok(Rc::new(Object::String(l.clone() + &r))),
            _ => Err(anyhow!(
                "unsupported boolean infix expression: {}",
                operator
            ))?,
        },
        (_, _) => Err(anyhow!("unsupported infix expressions")),
    }
}

fn evaluate_prefix_expression(operator: &Token, operand: Rc<Object>) -> EvaluatorResult {
    match operator {
        Token::Bang => evaluate_bang_expression(operand),
        Token::Minus => evaluate_minus_expression(operand),
        tok => Err(anyhow!(
            "expected !/- but got unsupported {} token while evaluation prefix expression",
            tok
        )),
    }
}

fn evaluate_minus_expression(operand: Rc<Object>) -> EvaluatorResult {
    match operand.as_ref() {
        Object::Integer(i) => Ok(Rc::new(Object::Integer(-i))),
        obj => Err(anyhow!(
            "expected integer for minus expression, got: {}",
            obj
        )),
    }
}

fn evaluate_bang_expression(operand: Rc<Object>) -> EvaluatorResult {
    Ok(Rc::new(Object::Boolean(!is_truthy(operand.as_ref()))))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{object::environment::Environment, parser::parse};

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

    #[test]
    fn test_return_statements() {
        struct Test<'a> {
            input: &'a str,
            expected: i64,
        }
        let tests = vec![
            Test {
                input: "return 10;",
                expected: 10,
            },
            Test {
                input: "return 10; 9;",
                expected: 10,
            },
            Test {
                input: "return 2 * 5; 9;",
                expected: 10,
            },
            Test {
                input: "9; return 2 * 5; 9;",
                expected: 10,
            },
            Test {
                input: "if (10 > 1) {
                           if (10 > 1) {
                             return 10;
                           }
                           return 1;
                         }",
                expected: 10,
            },
        ];

        for t in tests {
            let evaluated = eval(t.input);
            assert_integer_object(evaluated, t.expected)
        }
    }

    #[test]
    fn let_statements() {
        struct Test<'a> {
            input: &'a str,
            expected: i64,
        }
        let tests = vec![
            Test {
                input: "let a = 5; a;",
                expected: 5,
            },
            Test {
                input: "let a = 5 * 5; a;",
                expected: 25,
            },
            Test {
                input: "let a = 5; let b = a; b;",
                expected: 5,
            },
            Test {
                input: "let a = 5; let b = a; let c = a + b + 5; c;",
                expected: 15,
            },
        ];

        for t in tests {
            let evaluated = eval(t.input);
            assert_integer_object(evaluated, t.expected)
        }
    }

    #[test]
    fn test_function_object() {
        let input = "fn(x) { x + 2; };";
        let evaluated = eval(input);

        match &*evaluated {
            Object::Function(f) => {
                assert_eq!(f.parameters.len(), 1);
                assert_eq!(f.parameters.first().unwrap(), "x");
                assert_eq!(f.body.to_string(), "(x + 2)");
            }
            _ => panic!("expected function object but got {:?}", evaluated),
        }
    }

    #[test]
    fn test_function_application() {
        struct Test<'a> {
            input: &'a str,
            expected: i64,
        }
        let tests = vec![
            Test {
                input: "let identity = fn(x) { x; }; identity(5);",
                expected: 5,
            },
            Test {
                input: "let identity = fn(x) { return x; }; identity(5);",
                expected: 5,
            },
            Test {
                input: "let double = fn(x) { x * 2; }; double(5);",
                expected: 10,
            },
            Test {
                input: "let add = fn(x, y) { x + y; }; add(5, 5);",
                expected: 10,
            },
            Test {
                input: "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                expected: 20,
            },
            Test {
                input: "fn(x) { x; }(5)",
                expected: 5,
            },
        ];

        for t in tests {
            assert_integer_object(eval(t.input), t.expected)
        }
    }

    #[test]
    fn closures() {
        let input = "let newAdder = fn(x) {
  fn(y) { x + y };
};

let addTwo = newAdder(2);
addTwo(2);";
        assert_integer_object(eval(input), 4)
    }

    #[test]
    fn string_literal() {
        let input = r#""Hello World!"#;

        match &*eval(input) {
            Object::String(s) => assert_eq!(s, "Hello World!"),
            obj => panic!("expected string but got {}", obj),
        }
    }

    #[test]
    fn string_concatenation() {
        let input = r#""Hello" + " " + "World!""#;

        match eval(input).as_ref() {
            Object::String(s) => assert_eq!(s, "Hello World!"),
            obj => panic!("expected string but got {}", obj),
        }
    }

    #[test]
    fn test_builtin_functions() {
        struct Test<'a> {
            input: &'a str,
            expected: Object,
        }
        let tests = vec![
            Test {
                input: r#"len("")"#,
                expected: Object::Integer(0),
            },
            Test {
                input: r#"len("four")"#,
                expected: Object::Integer(4),
            },
            Test {
                input: r#"len("hello world")"#,
                expected: Object::Integer(11),
            },
            Test {
                input: "first([1, 2, 3])",
                expected: Object::Integer(1),
            },
            Test {
                input: "first([])",
                expected: Object::Null,
            },
            Test {
                input: "last([1, 2, 3])",
                expected: Object::Integer(3),
            },
            Test {
                input: "last([])",
                expected: Object::Null,
            },
            Test {
                input: "rest([1, 2, 3])",
                expected: Object::Array(Rc::new(vec![
                    Rc::new(Object::Integer(2)),
                    Rc::new(Object::Integer(3)),
                ])),
            },
            Test {
                input: "rest([])",
                expected: Object::Array(Rc::new(vec![])),
            },
            Test {
                input: "push([], 1)",
                expected: Object::Array(Rc::new(vec![Rc::new(Object::Integer(1))])),
            },
        ];

        for t in tests {
            let obj = eval(t.input);

            match (&t.expected, &*obj) {
                (Object::Integer(exp), Object::Integer(got)) => assert_eq!(
                    *exp, *got,
                    "on input {} expected {} but got {}",
                    t.input, exp, got
                ),
                (Object::Null, Object::Null) => {
                    // Both are null, test passes
                }
                (Object::Array(exp), Object::Array(got)) => {
                    assert_eq!(
                        exp.len(),
                        got.len(),
                        "on input {} array lengths differ",
                        t.input
                    );
                    for (i, (e, g)) in exp.iter().zip(got.iter()).enumerate() {
                        match (&**e, &**g) {
                            (Object::Integer(exp_val), Object::Integer(got_val)) => {
                                assert_eq!(
                                    exp_val, got_val,
                                    "on input {} element {} differs",
                                    t.input, i
                                );
                            }
                            _ => panic!(
                                "on input {} element {} expected {:?} but got {:?}",
                                t.input, i, e, g
                            ),
                        }
                    }
                }
                _ => panic!(
                    "on input {} expected {:?} but got {:?}",
                    t.input, t.expected, obj
                ),
            }
        }
    }

    #[test]
    fn test_array_literals() {
        let input = "[1, 2 * 2, 3 + 3]";
        let obj = eval(input);
        match &*obj {
            Object::Array(a) => {
                assert_integer_object(a.get(0).unwrap().clone(), 1);
                assert_integer_object(a.get(1).unwrap().clone(), 4);
                assert_integer_object(a.get(2).unwrap().clone(), 6);
            }
            _ => panic!("expected array but got {:?}", obj),
        }
    }

    #[test]
    fn test_array_index_expressions() {
        struct Test<'a> {
            input: &'a str,
            expected: i64,
        }
        let tests = vec![
            Test {
                input: "[1, 2, 3][0]",
                expected: 1,
            },
            Test {
                input: "[1, 2, 3][1]",
                expected: 2,
            },
            Test {
                input: "[1, 2, 3][2]",
                expected: 3,
            },
            Test {
                input: "let i = 0; [1][i];",
                expected: 1,
            },
            Test {
                input: "[1, 2, 3][1 + 1];",
                expected: 3,
            },
            Test {
                input: "let myArray = [1, 2, 3]; myArray[2];",
                expected: 3,
            },
            Test {
                input: "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
                expected: 6,
            },
            Test {
                input: "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
                expected: 2,
            },
        ];

        for t in tests {
            let obj = eval(t.input);
            assert_integer_object(obj, t.expected);
        }
    }

    #[test]
    fn test_invalid_array_index() {
        let inputs = vec!["[1, 2, 3][3]", "[1, 2, 3][-1]"];

        for input in inputs {
            let obj = eval(input);
            assert_null_object(obj);
        }
    }

    fn eval(input: &str) -> Rc<Object> {
        let _ = env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        let node = parse(input).unwrap();
        let env = Rc::new(RefCell::new(Environment::new()));

        evaluate(node, env).unwrap()
    }

    fn assert_null_object(obj: Rc<Object>) {
        match obj.as_ref() {
            Object::Null => {}
            x => panic!("expected null but got {}", x),
        }
    }

    fn assert_integer_object(left: Rc<Object>, r: i64) {
        match left.as_ref() {
            Object::Integer(i) => assert_eq!(i, &r),
            x => panic!("expected integer but got {}", x),
        }
    }

    fn assert_boolean_object(left: Rc<Object>, r: bool) {
        match left.as_ref() {
            Object::Boolean(b) => assert_eq!(b, &r),
            x => panic!("expected integer but got {}", x),
        }
    }
}
