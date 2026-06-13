#![no_std]
#![no_main]
mod walc;
use walc::*;

use cordic::*;
use fixed::types::I40F24 as Fixed;

const WELCOME_MESSAGE: &str = r"Welcome to a 64-bit fixed-point (40.24) RPN calculator!
Type 'help' for a list of available functions and constants.
";

const PROMPT: &str = "> ";

const HELP_MESSAGE: &str = r"Commands:
    help - Show this message
    exit - Exit the calculator

Operators:
    + - * / %

Constants:
    pi e

Functions:
    abs ceil floor round sqrt exp
    ilog2 ilog10                        - integer logarithms (floored)
    sin cos tan asin acos atan atan2    - trigonometric (radians)

Reverse Polish notation (RPN) examples:
    > 3 4 + 5 *     - Calculates (3 + 4) * 5
    35

    > pi 2 / sin    - Calculates sin(pi/2)
    1
";

fn main() {
    print_string(WELCOME_MESSAGE);

    loop {
        print_string(PROMPT);
        let input = read_line();
        let input = input.trim();

        match input {
            "exit" => break,
            "help" => print_string(HELP_MESSAGE),
            input => eval(input),
        }
    }
}

enum Token {
    Number(Fixed),

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Abs,

    Ceil,
    Floor,
    Round,

    Exp,
    Sqrt,

    Ilog2,
    Ilog10,

    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
}

impl Token {
    fn arity(&self) -> usize {
        match self {
            Token::Number(_) => 0,

            Token::Abs
            | Token::Ceil
            | Token::Floor
            | Token::Round
            | Token::Exp
            | Token::Sqrt
            | Token::Ilog2
            | Token::Ilog10
            | Token::Sin
            | Token::Cos
            | Token::Tan
            | Token::Asin
            | Token::Acos
            | Token::Atan => 1,

            Token::Add | Token::Sub | Token::Mul | Token::Div | Token::Mod | Token::Atan2 => 2,
        }
    }
}

fn parse_token(source: &str) -> Option<Token> {
    if let Ok(num) = source.parse::<Fixed>() {
        return Some(Token::Number(num));
    }

    match source {
        "pi" => Some(Token::Number(Fixed::PI)),
        "e" => Some(Token::Number(Fixed::E)),

        "+" => Some(Token::Add),
        "-" => Some(Token::Sub),
        "*" => Some(Token::Mul),
        "/" => Some(Token::Div),
        "%" => Some(Token::Mod),

        "abs" => Some(Token::Abs),

        "ceil" => Some(Token::Ceil),
        "floor" => Some(Token::Floor),
        "round" => Some(Token::Round),

        "exp" => Some(Token::Exp),
        "sqrt" => Some(Token::Sqrt),

        "ilog2" => Some(Token::Ilog2),
        "ilog10" => Some(Token::Ilog10),

        "sin" => Some(Token::Sin),
        "cos" => Some(Token::Cos),
        "tan" => Some(Token::Tan),
        "asin" => Some(Token::Asin),
        "acos" => Some(Token::Acos),
        "atan" => Some(Token::Atan),
        "atan2" => Some(Token::Atan2),

        _ => None,
    }
}

fn eval(input: &str) {
    let tokens = input
        .split_whitespace()
        .map(parse_token)
        .collect::<Option<Vec<_>>>();

    let tokens = match tokens {
        Some(tokens) => tokens,
        None => {
            print_string("Error: Invalid token in input.\n");
            return;
        }
    };

    let mut stack = Vec::<Fixed>::new();

    for token in tokens {
        if let Token::Number(num) = token {
            stack.push(num);
            continue;
        }

        let arity = token.arity();

        if stack.len() < arity {
            print_string("Error: Not enough operands for operator.\n");
            return;
        }

        if arity == 1 {
            let x = stack.pop().unwrap();
            let result = match token {
                Token::Abs => Some(x.abs()),
                Token::Ceil => Some(x.ceil()),
                Token::Floor => Some(x.floor()),
                Token::Round => Some(x.round()),
                Token::Exp => Some(exp(x)),
                Token::Sqrt => Some(sqrt(x)),
                Token::Ilog2 => x.checked_int_log2().map(|log| Fixed::from_num(log)),
                Token::Ilog10 => x.checked_int_log10().map(|log| Fixed::from_num(log)),
                Token::Sin => Some(sin(x)),
                Token::Cos => Some(cos(x)),
                Token::Tan => Some(tan(x)),
                Token::Asin => Some(asin(x)),
                Token::Acos => Some(acos(x)),
                Token::Atan => Some(atan(x)),
                _ => unreachable!(),
            };

            if let Some(result) = result {
                stack.push(result);
            } else {
                print_string("Error: Invalid input for operator.\n");
                return;
            }
        } else {
            let y = stack.pop().unwrap();
            let x = stack.pop().unwrap();
            let result = match token {
                Token::Add => x.checked_add(y),
                Token::Sub => x.checked_sub(y),
                Token::Mul => x.checked_mul(y),
                Token::Div => x.checked_div(y),
                Token::Mod => x.checked_rem(y),
                Token::Atan2 => Some(atan2(y, x)),
                _ => unreachable!(),
            };

            if let Some(result) = result {
                stack.push(result);
            } else {
                print_string("Error: Invalid input for operator (overflow or division by zero).\n");
                return;
            }
        }
    }

    if let Some(result) = stack.pop() {
        println!("{result}");
    }
}
