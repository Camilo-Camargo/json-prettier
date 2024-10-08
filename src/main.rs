use pest::error::Error;
use pest::Parser;
use pest_derive::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct JSONParser;

//const TODO_TEST_FILE: &str = "./test/test.json";

enum JSONValue<'a> {
    String(&'a str),
    Number(f64),
    Object(Vec<(&'a str, JSONValue<'a>)>),
    Array(Vec<JSONValue<'a>>),
    Boolean(bool),
    Null,
}

fn prettier_json(val: &JSONValue, indent_level: usize) -> String {
    use JSONValue::*;
    let indent = "  ".repeat(indent_level);
    match val {
        Object(o) => {
            let contents: Vec<_> = o
                .iter()
                .map(|(name, value)| {
                    format!(
                        "{}\"{}\": {}",
                        "  ".repeat(indent_level + 1),
                        name,
                        prettier_json(value, indent_level + 1)
                    )
                })
                .collect();
            format!("{{\n{}\n{}}}", contents.join(",\n"), indent)
        }
        Array(a) => {
            let contents: Vec<_> = a
                .iter()
                .map(|value| {
                    format!(
                        "{}{}",
                        "  ".repeat(indent_level + 1),
                        prettier_json(value, indent_level + 1)
                    )
                })
                .collect();
            format!(
                "[\n{}\n{}]",
                contents.join(",\n"),
                "  ".repeat(indent_level)
            )
        }
        String(s) => format!("{s}"),
        Number(n) => format!("{n}"),
        Boolean(b) => format!("{b}"),
        Null => format!("null"),
    }
}

fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    let json = JSONParser::parse(Rule::json, file)?.next().unwrap();

    use pest::iterators::Pair;

    fn parse_value(pair: Pair<Rule>) -> JSONValue {
        match pair.as_rule() {
            Rule::object => JSONValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let name = inner_rules
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap()
                            .as_str();
                        let value = parse_value(inner_rules.next().unwrap());
                        (name, value)
                    })
                    .collect(),
            ),
            Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
            Rule::string => JSONValue::String(pair.into_inner().next().unwrap().as_str()),
            Rule::number => JSONValue::Number(pair.as_str().parse().unwrap()),
            Rule::boolean => JSONValue::Boolean(pair.as_str().parse().unwrap()),
            Rule::null => JSONValue::Null,
            Rule::json
            | Rule::EOI
            | Rule::pair
            | Rule::value
            | Rule::inner
            | Rule::char
            | Rule::WHITESPACE => unreachable!(),
        }
    }
    Ok(parse_value(json))
}

fn main() {
    let matches = clap::Command::new("JSON Pretty Printer")
        .version("0.0.1")
        .author("Camilo Camargo <camilocamargo49@gmail.com>")
        .about("Parses and formats JSON files")
        .arg(
            clap::Arg::new("file")
                .help("Path to the JSON file")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::new("output")
                .short('o')
                .help("Path to the output file")
                .long("output"),
        )
        .get_matches();

    let file_path = matches
        .get_one::<String>("file")
        .expect("File path is required");
    let output_path = matches.get_one::<String>("output");

    let file = fs::read_to_string(file_path).expect("cannot read file");
    let json: JSONValue = parse_json_file(&file).expect("parse error");
    let prettified_json = prettier_json(&json, 0);

    match output_path {
        Some(path) => {
            fs::write(path, prettified_json).expect("cannot write to file");
            println!("Prettified JSON written to {}", path);
        }
        None => print!("{}", prettified_json),
    }
}
