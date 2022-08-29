use std::{cell::RefCell, rc::Rc, process::exit};

use hp::{Parser, Template};

fn main() {
    let result = Rc::new(RefCell::new(0.));

    let mut parser = Parser::new()
        .with_author("Example")
        .with_description("hp example calculator program.")
        .with_program_name("calc");

    let res = result.clone();
    parser.add_template(
        Template::new()
            .matches("add")
            .matches("+")
            .with_help("Add two or more numbers supplied.")
            .number_of_values(99)
            .optional_values(false)
            .on_parse(move |values| {
                for value in values {
                    if let Ok(v) = value.parse::<f64>() {
                        *res.borrow_mut() += v;
                    }
                }
            }),
    );
    let res = result.clone();
    parser.add_template(
        Template::new()
            .matches("sub")
            .matches("-")
            .with_help("Substitute two or more numbers supplied.")
            .number_of_values(99)
            .optional_values(false)
            .on_parse(move |values| {
                for value in values {
                    if let Ok(v) = value.parse::<f64>() {
                        *res.borrow_mut() -= v
                    }
                }
            }),
    );
    let res = result.clone();
    parser.add_template(
        Template::new()
            .matches("mul")
            .matches("*")
            .with_help("Multiply two or more numbers supplied.")
            .number_of_values(99)
            .optional_values(false)
            .on_parse(move |values| {
                for value in values {
                    if let Ok(v) = value.parse::<f64>() {
                        *res.borrow_mut() *= v
                    }
                }
            }),
    );
    let res = result.clone();
    parser.add_template(
        Template::new()
            .matches("div")
            .matches("/")
            .with_help("Divide two or more numbers supplied.")
            .number_of_values(99)
            .optional_values(false)
            .on_parse(move |values| {
                for value in values {
                    if let Ok(v) = value.parse::<f64>() {
                        *res.borrow_mut() /= v
                    };
                }
            }),
    );

    if let Err(e) = parser.parse(None) {
        println!("{e}");
        exit(1);
    }

    println!("{}", result.borrow())
}
