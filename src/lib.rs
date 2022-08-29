//! # About
//! `hp` or `HashParse` is a fast, simple and completely dependency free command line argument parsing library.
//! Parsing is done in `O(n)` where `n` is the number of supplied arguments and the implementation
//! is based on **HashMaps**, all lookups should be in `O(1)` time complexity.
//!
//! The most prominent features are:
//! - automatic help message generation.
//! - colorful and verbose error reporting.
//! - hierarchical commands and subcommands.
//! - easy to read and understand documentation.
//!
//! # Rationale
//! For the last few months all of my projects in Rust were command line tools. I'm still a student
//! with a ton of time on my hands, so my approach generally was, and still is for the most part, to
//! use as little dependencies as possible and try to implement most of the stuff I need myself so
//! I can gain a better understanding of the component in question while also keeping the project's size
//! and complexity at a minimum. When writing these command line tools, I realized that I needed a simple and
//! efficient way to parse command line arguments. In every project I worked on I mostly used a variation of the
//! same complex parser which was always a hassle to write and then scale and add more arguments to, as the
//! project grew. I wanted something simple, small and fast with little to no dependencies, something
//! that would provided everything that a simple CLI app might need. I knew of other libraries such as `clap` or
//! `structopt` **(both of which you should probably use instead of this)**, but I wanted to write something simple which would provide some of
//! the same functionality, such as automatic `--help` message generation and a good way to report errors and most of all,
//! a way to quickly add new commands and subcommands. So over the span of a few days, I wrote and documented most of
//! `hp` the high performance, no dependency command line parsing library. There are still some
//! things I want to change and improve, but it should be in a good and working condition now.
//!
//! ```
//! extern crate hp;
//! use hp::{Parser, Template};
//! use std::process::exit;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new parser and set the necessary fields for the `--help` command.
//!     let mut parser = Parser::new()
//!         .with_description("An example program.")
//!         .with_author("Me")
//!         .with_program_name("myprog");
//!
//!     // Add a top level `command template` that will look for `--say` in the command line arguments.
//!     // The command takes 1 value that is not optional, it is required.
//!     // The third function argument is the help string which will be printed by the `--help`
//!     // command.
//!     parser.add("--say", 1, "Print the command line argument value supplied.");
//!
//!     // For finer control, a user might want to use the `Template` structure for a better
//!     // control over the added Template.
//!     parser.add_template(Template::new()
//!                         .matches("--new")
//!                         .matches("-n")
//!                         .number_of_values(2)
//!                         .optional_values(true)
//!                         .with_help("Do some amazing thing here please!"));
//!
//!     // Each added `Template` returns an ID.
//!     // This ID can than be used to create subcommands for that given command.
//!     let id = parser.add("-c", 0, "Count something based on the subcommands supplied.");
//!
//!     parser.add_subcommand(id, "--add", 2, "Perform addition on two numbers.");
//!
//!
//!     // $ myprog --add
//!     // ERROR: Out of context argument, because '--add' is a subcommand of
//!     // '-c' and '-c' is not present in the command.
//!     //
//!     // $ myprog -c --add 2 2
//!     // 4
//!
//!     // Call closures instantly, when an argument is parsed.
//!     //
//!     // Normally when using the `has()` and `get()` interface on the `ParsedAruguments` struct,
//!     // upon parsing multiple arguments matching the same `Template` only the last instance of
//!     // the argument is stored in the parsed result.
//!     //
//!     // The closure passed to `on_parse()` is called during parsing on every instance of the
//!     // argument matching the same template. This means that the code meant for that template
//!     // can be run on every value of every occurrence of the parsed template.
//!     parser.add_subcommand_template(id, Template::new()
//!         .matches("--sub")
//!         .number_of_values(2)
//!         .on_parse(|values| {
//!             let (a, b): (i32, i32) = (values[0].parse().unwrap(), values[1].parse().unwrap());
//!             println!("{}", a - b);
//!         }));
//!
//!     let presult = parser.parse(None);
//!
//!     if let Some(err) = presult.as_ref().err() {
//!         println!("{err}");
//!         exit(1);
//!     }
//!
//!     let presult = presult.unwrap();
//!
//!     if presult.has("--say") {
//!         println!("Saying: {}", presult.get("--say").unwrap().values()[0]);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Examples
//! Most of the functions have some simple code examples. There's also an `examples` directory
//! which currently shows the two main ways in which `hp` could be used.

use std::cell::RefCell;
use std::collections::HashMap;
use std::env::{self, current_exe};
use std::fmt::Write;
use std::process::exit;
use std::rc::Rc;

pub use errors::HpError;

pub mod errors;

type Action = Rc<RefCell<dyn FnMut(Vec<String>)>>;
pub type TemplateId = usize;

#[derive(Clone, Debug)]
/// A parsed and verified .
pub struct ParsedArgument {
    id: TemplateId,
    values: Vec<String>,
}

impl ParsedArgument {
    fn new(id: usize, values: Vec<String>) -> Self {
        Self { id, values }
    }

    /// Return the parsed argument values.
    pub fn values(&self) -> &Vec<String> {
        &self.values
    }

    /// Return the unique identifier of this parsed argument.
    pub fn id(&self) -> TemplateId {
        self.id
    }

    /// Get the number of values this parsed argument has.
    pub fn number_of_values(&self) -> usize {
        self.values.len()
    }
}

#[derive(Clone, Debug)]
/// This is the output of the parsing.
///
/// `ParsedAruguments` contains a `HashMap` of arugment names and `ParsedArgument` structures. It
/// provides an interface for the user to quickly and simply verify the presence of a
/// `ParsedArgument` and additionally retrieve its values.
pub struct ParsedArguments {
    hm: HashMap<String, ParsedArgument>,
    ids: HashMap<usize, ParsedArgument>,
}

impl ParsedArguments {
    /// Try to get a **top-level** parsed argument, given its name.
    ///
    /// ```ignore
    /// let mut parser = Parser::new();
    /// parser.add("--some-arg", 0, "Some help.");
    /// let result = parser.parse()?;
    ///
    /// result.get("--some-arg").is_some().then(|| println!("--some-arg in arguments!"))
    /// ```
    pub fn get(&self, key: impl AsRef<str>) -> Option<&ParsedArgument> {
        let key = format!("0#{}", key.as_ref());
        self.hm.get(&key)
    }

    /// Try to get a parsed argument, given its ID.
    ///
    /// ```ignore
    /// let mut parser = Parser::new();
    /// let id = parser.add("--some-other-arg", 0, "Some help.");
    /// let result = parser.parse()?;
    ///
    /// result.get_with_id(id).is_some().then(|| println!("--some-other-arg in arguments!"))
    /// ```
    pub fn get_with_id(&self, id: TemplateId) -> Option<&ParsedArgument> {
        self.ids.get(&id)
    }

    /// Assert, whether a **top-level** argument that mathces `key` has been parsed.
    ///
    /// This function is an alias for `parsed_args.get("some-key").is_some()`.
    ///
    /// ```ignore
    /// let mut parser = Parser::new();
    /// parser.add("--hello", 0, "Some help.");
    /// let result = parser.parse()?;
    ///
    /// result.has("--hello").then(|| println!("Hello!"))
    /// ```
    pub fn has(&self, key: impl AsRef<str>) -> bool {
        self.get(key).is_some()
    }

    /// Assert, whether argument whith `id` has been parsed.
    ///
    /// This function is an alias for `parsed_args.get_with_id(id).is_some()`.
    /// ```ignore
    /// let mut parser = Parser::new();
    /// let world = parser.add("--world", 0, "Some help.");
    /// let result = parser.parse()?;
    ///
    /// result.has(world).then(|| println!("World!"))
    /// ```
    pub fn has_with_id(&self, id: TemplateId) -> bool {
        self.get_with_id(id).is_some()
    }

    /// Try to get a parsed subargument of an argument, given the arguemnt ID and the subargument
    /// name.
    ///
    /// ```ignore
    /// let mut parser = Parser::new();
    /// let world = parser.add("--world", 0, "World command.");
    /// parser.add_subcommand(world, "--new", 1, "Add a new world.")
    /// let result = parser.parse()?;
    ///
    /// if let Some(new_world) = result.get_with_context(world, "--new") {
    ///     println!("created a new world with name {}", new_world.values()[0]);
    /// }
    /// ```
    pub fn get_with_context(
        &self,
        context: usize,
        key: impl AsRef<str>,
    ) -> Option<&ParsedArgument> {
        let key = format!("{context}#{}", key.as_ref());
        self.hm.get(&key)
    }

    /// Asssert, whether a subargument of an argument is present in the `ParsedArguments`.
    ///
    /// Alias for `parsed_args.get_with_context(context, "something").is_some()`
    ///
    /// ```ignore
    /// let mut parser = Parser::new();
    /// let world = parser.add("--world", 0, "World command.");
    /// parser.add_subcommand(world, "--list", 0, "Add a new world.")
    /// let result = parser.parse()?;
    ///
    /// if result.has_with_context(world, "--list") {
    ///     println!("Listing worlds: {worlds}");
    /// }
    /// ```
    pub fn has_with_context(&self, context: usize, key: impl AsRef<str>) -> bool {
        self.get_with_context(context, key).is_some()
    }
}

#[derive(Default, Clone)]
/// `Template` contains all the necessary information for matching and parsing a command line
/// argument.
pub struct Template {
    matches: Vec<String>,
    num_values: usize,
    optional_vals: bool,
    help: String,
    subargument_of: Option<usize>,
    id: TemplateId,
    action: Option<Action>,
}

impl Template {
    /// Creates a new `Template`.
    ///
    /// Create an empty, uninitialized `Template`.
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            num_values: 0,
            optional_vals: false,
            help: "".into(),
            subargument_of: None,
            id: 0,
            action: None,
        }
    }

    /// Add a value which identifies this template when parsing the command line arguments.
    ///
    /// ```ignore
    /// parser.add_template(Template::new().matches("--hi"));
    ///
    /// let ret = parser.parse()?;
    /// assert!(ret.has("--hi"))
    /// ```
    pub fn matches<S: AsRef<str>>(mut self, name: S) -> Self {
        let name = name.as_ref().to_string();
        if !self.matches.contains(&name) {
            self.matches.push(name)
        }

        self
    }

    /// Set the number of values this template takes.
    ///
    /// If 0, the argument takes no values, but it can have subarguments.
    ///
    /// ```ignore
    /// parser.add_template(Template::new().matches("--say").number_of_values(1));
    ///
    /// let ret = parser.parse()?;
    /// if let Ok(vals) = ret.get("--say") {
    ///     println!("{}", vals[0]);
    /// }
    ///
    /// // $ myprog --say hello
    /// // hello
    /// //
    /// // $ myprog --say hello world
    /// // hello
    /// //
    /// // $ myprog --say
    /// // ERROR: missing argument for '--say'.
    /// ```
    pub fn number_of_values(mut self, nv: usize) -> Self {
        self.num_values = nv;
        self
    }

    /// Ignores missing values when parsing command line arguments.
    ///
    /// ```ignore
    /// parser.add_template(Template::new()
    ///                         .matches("--say")
    ///                         .number_of_values(1)
    ///                         .optional_values(true));
    ///
    /// let ret = parser.parse()?;
    /// if let Ok(vals) = ret.get("--say") {
    ///     if let Some(val) = vals.get(0) {
    ///         println!("{val}");
    ///     };
    /// }
    ///
    /// // $ myprog --say hello
    /// // hello
    /// // $ myprog --say # nothing happens, would cause a missing argument error
    /// ```
    pub fn optional_values(mut self, ov: bool) -> Self {
        self.optional_vals = ov;
        self
    }

    /// Set a help string for this template.
    ///
    /// ```ignore
    /// parser.with_description("My awesome program!")
    /// parser.add_template(Template::new()
    ///                         .matches("--say")
    ///                         .number_of_values(1)
    ///                         .optional_values(true)
    ///                         .with_help("Print a given argument."));
    ///
    /// // ...
    /// parser.parse()?;
    ///
    ///
    /// // $ myprog --say hello
    /// // hello
    /// // $ myprog --help
    /// // myprog: My awesome program!
    /// // usage:
    /// //     $ myprog -[-arguments] [value/s]
    /// // arguments:
    /// //     --say [1 optional value\s]        Print a given argument.
    /// //     --help                          Print this help message.
    /// ```
    pub fn with_help<S: AsRef<str>>(mut self, help_string: S) -> Self {
        self.help = help_string.as_ref().into();
        self
    }

    /// Set an action that will be executed immediately when a command is parsed.
    ///
    /// This action is a function with the following signature: `fn(Vec<String>) -> ()`.
    ///
    /// ```ignore
    /// let mut parser = Parser::new();
    ///
    /// parser.add_subcommand_template(id, Template::new()
    ///    .matches("--add")
    ///    .number_of_values(2)
    ///    .on_parse(|values| {
    ///        let (a, b): (i32, i32) = (values[0].parse().unwrap(), values[1].parse().unwrap());
    ///        println!("{}", a + b);
    ///    }));
    ///
    /// ```
    pub fn on_parse<F: FnMut(Vec<String>) + 'static>(mut self, action: F) -> Self {
        self.action = Some(Rc::new(RefCell::new(action)));
        self
    }

    pub(crate) fn set_id(&mut self, id: usize) {
        self.id = id
    }

    pub(crate) fn subarg(&mut self, id: usize) {
        let _ = self.subargument_of.insert(id);
    }
}

#[derive(Default, Clone)]
/// Command line argument parser.
///
/// ```ignore
/// let parser = Parser::new()
///                 .exit_on_help(false)
///                 .with_description("My amazing program!")
///                 .with_usage("$ myprog [commmand] --[subcommand/s] [value/s]")
///                 .with_author("[REDACTED]");
/// ```
pub struct Parser {
    stored: HashMap<String, Template>,
    order: Vec<String>,
    last_id: usize,
    exit_on_help: bool,
    author: String,
    description: String,
    usage: String,
    program_name: String,
    help: Option<String>,
}

impl Parser {
    /// Create a new `Parser` with default configuration, no author and description.
    pub fn new() -> Self {
        let exe_name = match current_exe() {
            Ok(pb) => {
                if let Some(name) = pb.file_name() {
                    name.to_str().unwrap_or("").to_string()
                } else {
                    "".to_string()
                }
            }
            Err(_) => "".to_string(),
        };
        Self {
            stored: HashMap::new(),
            order: Vec::new(),
            last_id: 0,
            exit_on_help: true,
            author: "".to_string(),
            description: "".to_string(),
            usage: "".to_string(),
            program_name: exe_name,
            help: None,
        }
    }

    /// Specifies, whether the program should exit after printing the help message when the
    /// '--help' or '-h' command line arguments are specified.
    pub fn exit_on_help(mut self, v: bool) -> Self {
        self.exit_on_help = v;
        self
    }

    /// Specifies the author of the program, will be used when printing the help message.
    pub fn with_author<S: AsRef<str>>(mut self, v: S) -> Self {
        self.author = v.as_ref().to_string();
        self
    }

    /// Specifies the description of the program, will be used when printing the help message.
    pub fn with_description<S: AsRef<str>>(mut self, v: S) -> Self {
        self.description = v.as_ref().to_string();
        self
    }

    /// Specifies the usage of the program, will be used when printing the help message.
    ///
    /// If nothing is specified, `hp` will try to interpret the given templates and create a custom
    /// usage string.
    pub fn with_usage<S: AsRef<str>>(mut self, v: S) -> Self {
        self.usage = v.as_ref().to_string();
        self
    }

    /// Specifies the program name, will be used when printing the help message.
    ///
    /// If none is specified the name of the binary will be be used.
    pub fn with_program_name<S: AsRef<str>>(mut self, v: S) -> Self {
        self.program_name = v.as_ref().to_string();
        self
    }

    /// Set a completely custom help string, which will be used when printing the `--help`
    /// string.
    pub fn set_help<S: AsRef<str>>(mut self, v: S) -> Self {
        self.help = Some(v.as_ref().to_string());
        self
    }

    fn generate_id(&mut self) -> usize {
        self.last_id += 1;
        self.last_id
    }

    fn add_to_map(&mut self, mut template: Template) -> TemplateId {
        let template_id = self.generate_id();
        template.set_id(template_id);
        let matches = template.matches.clone();
        for name in matches.iter() {
            let subarg = template.subargument_of.unwrap_or(0);
            let new_name = format!("{}#{}", subarg, name.clone());
            let _ = self.stored.insert(new_name.clone(), template.clone());
            self.order.push(name.clone())
        }

        template_id
    }

    /// Add a new `Template` to the parser. Return the ID of the `Template`.
    ///
    /// This method is an abstraction, it creates the `Template` for you, but it takes away some of the options.
    pub fn add<S: AsRef<str>>(
        &mut self,
        matches: S,
        num_values: usize,
        help_message: S,
    ) -> TemplateId {
        let template = Template::new()
            .matches(matches)
            .number_of_values(num_values)
            .with_help(help_message);
        self.add_to_map(template)
    }

    /// A a new `Template` to the parser. Return the ID of the `Template`.
    ///
    /// Given a template, add it to the parser.
    pub fn add_template(&mut self, template: Template) -> TemplateId {
        self.add_to_map(template)
    }

    /// Add a `Template` that is a subcommand of an already existing template to the parser. You
    /// need to provide a `Template` ID of the `Template` this `Template` will be the subcommand of.
    ///
    /// This method creates the `Template` for you, but it takes away some of the options.
    pub fn add_subcommand<S: AsRef<str>>(
        &mut self,
        subargument_of: usize,
        matches: S,
        num_values: usize,
        help_message: S,
    ) -> TemplateId {
        let id = self.generate_id();
        let mut template = Template::new()
            .matches(matches.as_ref())
            .number_of_values(num_values)
            .with_help(help_message.as_ref());
        template.set_id(id);
        template.subarg(subargument_of);

        self.add_to_map(template)
    }

    /// Add a `Template` that is a subcommand of an already existing template to the parser. You
    /// need to provide a `Template` ID of the `Template` this `Template` will be the subcommand of.
    pub fn add_subcommand_template(
        &mut self,
        subargument_of: usize,
        mut template: Template,
    ) -> TemplateId {
        let id = self.generate_id();
        template.set_id(id);
        template.subarg(subargument_of);

        self.add_to_map(template)
    }

    fn create_help(&self) -> String {
        let mut result_string = String::new();

        let longest_value_len = self
            .stored
            .values()
            .into_iter()
            .map(|t| {
                let mut temp = t.matches.join(" | ");
                if t.num_values > 0 {
                    let optional = match t.optional_vals {
                        true => " optional ",
                        false => " ",
                    };
                    write!(temp, " [{}{optional}values]", t.num_values).unwrap();
                }

                temp.len()
            })
            .max();

        if !self.program_name.is_empty() {
            write!(result_string, "{}", self.program_name).unwrap_or(());
        }
        if !self.description.is_empty() {
            writeln!(result_string, ": {}", self.description).unwrap_or(());
        }
        if !self.author.is_empty() {
            writeln!(result_string, "Author: {}", self.author).unwrap_or(());
        }
        if !self.usage.is_empty() {
            writeln!(result_string, "Usage:\n    {}", self.usage).unwrap_or(());
        } else {
            writeln!(
                result_string,
                "Usage:\n    $ {} -[-command] [value/s...]",
                self.program_name
            )
            .unwrap_or(());
        }

        let longest_value_len = match longest_value_len {
            Some(l) => l + 4,
            None => 4,
        };
        let mut max_level = 0;

        writeln!(result_string, "Arguments:").unwrap_or(());

        let mut template_vec: Vec<(&Template, usize)> = Vec::new();
        for name in self.order.iter() {
            let each = self
                .stored
                .values()
                .find(|temp| temp.matches.contains(name))
                .unwrap();
            if !template_vec
                .iter()
                .any(|(template, _)| template.id == each.id)
            {
                if let Some(sub_arg_of) = each.subargument_of {
                    if let Some((index, (_, level))) = template_vec
                        .iter()
                        .enumerate()
                        .find(|(_, (t, _))| t.id == sub_arg_of)
                    {
                        if level + 1 > max_level {
                            max_level = level + 1;
                        }
                        template_vec.insert(index + 1, (each, level + 1));
                    }
                } else {
                    template_vec.push((each, 0))
                }
            }
        }

        for (template, level) in template_vec.iter() {
            let mut lvl = String::new();
            (0..(level * 4)).for_each(|_| lvl.push(' '));

            let mut matches = template.matches.join(" | ");
            if template.num_values > 0 {
                let optional = match template.optional_vals {
                    true => " optional ",
                    false => " ",
                };
                write!(matches, " [{}{optional}values]", template.num_values).unwrap();
            }

            while matches.len() != longest_value_len + (max_level * 4) - lvl.len() {
                matches.push(' ');
            }

            writeln!(result_string, "    {lvl}{matches} {}", template.help).unwrap_or(());
        }

        let mut help = String::from("-h, --help");
        while help.len() != longest_value_len + max_level * 4 {
            help.push(' ');
        }

        write!(result_string, "    {help} Print this help message!").unwrap_or(());

        result_string
    }

    fn help_and_exit(&self) {
        if let Some(help) = &self.help {
            println!("{help}");
        } else {
            let help_string = self.create_help();

            println!("{help_string}");
        }

        if self.exit_on_help {
            exit(0);
        }
    }

    /// Parse the command line arguments, or a list of strings, if provided, and return a
    /// `ParsedArguments` structure.
    pub fn parse(&mut self, from: Option<Vec<&str>>) -> Result<ParsedArguments, HpError> {
        let args: Vec<String>;
        if let Some(from_vec) = from {
            args = from_vec.iter().map(|each| each.to_string()).collect();
        } else {
            args = env::args().collect();
        }

        let mut hm = HashMap::new();
        let mut idhm = HashMap::new();

        let mut context = 0;

        for (index, arg) in args.iter().enumerate() {
            if arg == "--help" || arg == "-h" {
                self.help_and_exit()
            }
            let query = format!("{context}#{arg}");
            let query2 = format!("0#{arg}");

            if self.stored.get(&query).is_some() {
                if let Some(template) = self.stored.get(&query) {
                    context = template.id;
                    let mut i = index;
                    let mut count = 0;
                    let mut values: Vec<String> = Vec::new();

                    while i < index + template.num_values {
                        i += 1;
                        if i == args.len() {
                            break;
                        }
                        let value = &args[i];

                        let q1 = format!("{context}#{value}");
                        let q2 = format!("0#{value}");

                        if self.stored.get(&q1).is_some() || self.stored.get(&q2).is_some() {
                            break;
                        } else {
                            values.push(value.to_string());
                            count += 1;
                        }
                    }

                    if !template.optional_vals && count < template.num_values {
                        return Err(HpError::NumberOfValues(
                            arg.into(),
                            count,
                            template.num_values,
                        ));
                    }

                    if let Some(action) = &template.action {
                        action.borrow_mut()(values.clone());
                    }

                    let pa = ParsedArgument::new(template.id, values);
                    hm.insert(query, pa.clone());
                    idhm.insert(template.id, pa);
                }
            } else if let Some(template) = self.stored.get(&query2) {
                context = template.id;
                let mut i = index;
                let mut count = 0;
                let mut values: Vec<String> = Vec::new();

                while i < index + template.num_values {
                    i += 1;
                    if i == args.len() {
                        break;
                    }
                    let value = &args[i];

                    let q1 = format!("{context}#{value}");
                    let q2 = format!("0#{value}");

                    if self.stored.get(&q1).is_some() || self.stored.get(&q2).is_some() {
                        break;
                    } else {
                        values.push(value.to_string());
                        count += 1;
                    }
                }

                if !template.optional_vals && count < template.num_values {
                    return Err(HpError::NumberOfValues(
                        arg.into(),
                        count,
                        template.num_values,
                    ));
                }

                if let Some(action) = &template.action {
                    action.borrow_mut()(values.clone());
                }

                let pa = ParsedArgument::new(template.id, values);
                hm.insert(query2, pa.clone());
                idhm.insert(template.id, pa);
            } else if let Some(template) = self.stored.values().find(|t| t.matches.contains(arg)) {
                if let Some(parent) = template.subargument_of {
                    let parent = self.stored.values().find(|t| t.id == parent).unwrap();
                    let parent_match = &parent.matches[0];
                    return Err(HpError::OutOfContext(
                        arg.to_string(),
                        parent_match.to_string(),
                    ));
                }
            }
        }

        Ok(ParsedArguments { hm, ids: idhm })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn help() {
        let mut parser = Parser::new()
            .with_usage("")
            .with_author("me")
            .with_description("Example program")
            .exit_on_help(false);

        parser.add("--say", 0, "Repeat something");
        let expand = parser.add_template(
            Template::new()
                .matches("-x")
                .matches("--expand")
                .optional_values(false)
                .number_of_values(0)
                .with_help("Expand something."),
        );

        let sub_sub = parser.add_subcommand_template(
            expand,
            Template::new()
                .matches("--string")
                .number_of_values(0)
                .with_help("Expands a string"),
        );

        let sub_sub_sub = parser.add_subcommand(sub_sub, "--super-test", 0, "Amazing super test.");

        let _inf = parser.add_subcommand(sub_sub_sub, "-i", 0, "Infinite nesting!");

        parser.parse(Some(vec!["--help"])).unwrap();
    }

    #[test]
    fn parsed_args() {
        let mut parser = Parser::new();
        parser.add("--hello", 0, "hello");
        let arg = parser.add("arg", 3, "hello");
        parser.add_template(
            Template::new()
                .matches("-not-found")
                .number_of_values(0)
                .with_help("bad.")
                .on_parse(|_| ()),
        );

        let result = parser.parse(Some(vec!["--hello", "arg", "h", "w", "x"]));

        assert!(result.is_ok());
        let r = result.expect("bad");

        assert!(r.has("--hello"));
        assert!(r.get_with_id(arg).unwrap().values.len() == 3);
        assert!(!r.has("-not-found"))
    }

    #[test]
    fn context_parsing() {
        let mut parser = Parser::new()
            .with_usage("")
            .with_author("me")
            .with_description("Example program")
            .exit_on_help(true);

        parser.add("--say", 0, "Repeat something");
        let expand = parser.add_template(
            Template::new()
                .matches("-x")
                .matches("--expand")
                .optional_values(false)
                .number_of_values(0)
                .with_help("Expand something."),
        );

        let sub_sub = parser.add_subcommand_template(
            expand,
            Template::new()
                .matches("--string")
                .number_of_values(0)
                .with_help("Expands a string"),
        );

        let sub_sub_sub = parser.add_subcommand(sub_sub, "--super-test", 0, "Amazing super test.");

        parser.add_subcommand(sub_sub_sub, "-i", 0, "Infinite nesting!");

        let result = parser
            .parse(Some(vec!["-x", "--string", "--super-test", "-i"]))
            .unwrap();

        assert!(result.has_with_context(expand, "--string"));
        assert!(result.has_with_context(sub_sub, "--super-test"));
        assert!(result.has_with_context(sub_sub_sub, "-i"));
    }

    #[test]
    fn out_of_context() {
        let mut parser: Parser = Parser::new()
            .with_usage("")
            .with_author("me")
            .with_description("Example program")
            .exit_on_help(true);

        parser.add("--say", 0, "Repeat something");
        let expand = parser.add_template(
            Template::new()
                .matches("-x")
                .matches("--expand")
                .optional_values(false)
                .number_of_values(0)
                .with_help("Expand something."),
        );

        parser.add_subcommand_template(
            expand,
            Template::new()
                .matches("--string")
                .number_of_values(0)
                .with_help("Expands a string"),
        );

        let result = parser.parse(Some(vec!["--string"]));

        assert!(result.is_err());

        println!("{}", result.err().unwrap())
    }

    #[test]
    fn action() {
        let mut parser = Parser::new();
        let mut last_val = String::new();

        parser.add_template(
            Template::new()
                .matches("say")
                .on_parse(move |values| {
                    println!("Saying: ");
                    for each in values.iter() {
                        last_val = each.to_string().clone();
                    }

                    println!("Last val {last_val}");
                })
                .number_of_values(8)
                .optional_values(true),
        );

        assert!(parser.parse(Some(vec!["say", "hello", "world"])).is_ok())
    }
}
