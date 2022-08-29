
# About
`hp` or `HashParse` is a fast, simple and completely dependency free command line argument parsing library. Parsing is done in `O(n)` where `n` is the number of supplied arguments and the implementation is based on **HashMaps**, all lookups should be in `O(1)` time complexity.

The most prominent features are:
- automatic help message generation.
- colorful and verbose error reporting.
- hierarchical commands and subcommands.
- easy to read and understand documentation.

# Rationale
For the last few months all of my projects in Rust were command line tools. I'm still a student with a ton of time on my hands, so my approach generally was, and still is for the most part, to use as little dependencies as possible and try to implement most of the stuff I need myself so I can gain a better understanding of the component in question while also keeping the project's size and complexity at a minimum. When writing these command line tools, I realized that I needed a simple and efficient way to parse command line arguments. In every project I worked on I mostly used a variation of the same complex parser which was always a hassle to write and then scale and add more arguments to, as the project grew. I wanted something simple, small and fast with little to no dependencies, something that would provided everything that a simple CLI app might need. I knew of other libraries such as `clap` or `structopt` (both of which you should probably use instead of this), but I wanted to write something simple which would provide some of the same functionality, such as automatic `--help` message generation and a good way to report errors and most of all, a way to quickly add new commands and subcommands. So over the span of a few days, I wrote and documented most of `hp` the high performance, no dependency command line parsing library. There are still some things I want to change and improve, but it should be in a good and working condition now.

```rust
extern crate hp;
use hp::{Parser, Template};
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new parser and set the necessary fields for the `--help` command.
    let mut parser = Parser::new()
        .with_description("An example program.")
        .with_author("Me")
        .with_program_name("myprog");

    // Add a top level `command template` that will look for `--say` in the command line arguments.
    // The command takes 1 value that is not optional, it is required.
    // The third function argument is the help string which will be printed by the `--help`
    // command.
    parser.add("--say", 1, "Print the command line argument value supplied.");

    // For finer control, a user might want to use the `Template` structure for a better
    // control over the added Template.
    parser.add_template(Template::new()
                        .matches("--new")
                        .matches("-n")
                        .number_of_values(2)
                        .optional_values(true)
                        .with_help("Do some amazing thing here please!"));

    // Each added `Template` returns an ID.
    // This ID can than be used to create subcommands for that given command.
    let id = parser.add("-c", 0, "Count something based on the subcommands supplied.");

    parser.add_subcommand(id, "--add", 2, "Perform addition on two numbers.");


    // $ myprog --add
    // ERROR: Out of context argument, because '--add' is a subcommand of
    // '-c' and '-c' is not present in the command.
    //
    // $ myprog -c --add 2 2
    // 4

    // Call closures instantly, when an argument is parsed.
    //
    // Normally when using the `has()` and `get()` interface on the `ParsedAruguments` struct,
    // upon parsing multiple arguments matching the same `Template` only the last instance of
    // the argument is stored in the parsed result.
    //
    // The closure passed to `on_parse()` is called during parsing on every instance of the
    // argument matching the same template. This means that the code meant for that template
    // can be run on every value of every occurrence of the parsed template.
    parser.add_subcommand_template(id, Template::new()
        .matches("--sub")
        .number_of_values(2)
        .on_parse(|values| {
            let (a, b): (i32, i32) = (values[0].parse().unwrap(), values[1].parse().unwrap());
            println!("{}", a - b);
        }));

    let presult = parser.parse(None);

    if let Some(err) = presult.as_ref().err() {
        println!("{err}");
        exit(1);
    }

    let presult = presult.unwrap();

    if presult.has("--say") {
        println!("Saying: {}", presult.get("--say").unwrap().values()[0]);
    }

    Ok(())
}
```

# Examples
Most of the functions have some simple code examples. There's also an `examples` directory which currently has a working calculator CLI app.

# Contributing & Fixing
If you feel like there's something missing from `hp` feel free to open an issue or submit a pull request. All contributions and discussions are very welcome!
