use basic_parser::basic_parser::{char, digit, string};

fn main() {
    println!("Hello, world!");

    let input = "SELECT id, name FROM foo;".to_string();
    let parser = char('S');
    match parser.run(input) {
        Some((parsed, remaining)) => {
            println!("Parsed: '{parsed}', Remaining: '{remaining}'")
        }
        None => {
            println!("Parsing failed.");
        }
    }

    let input = "5dsf".to_string();
    let parser = digit();
    match parser.run(input) {
        Some((parsed, remaining)) => {
            println!("Parsed: '{parsed}', Remaining: '{remaining}'")
        }
        None => {
            println!("Parsing failed.");
        }
    }

    let input = "hello, world".to_string();
    let parser = string("hell");
    match parser.run(input) {
        Some((parsed, remaining)) => {
            println!("Parsed: '{parsed}', Remaining: '{remaining}'")
        }
        None => {
            println!("Parsing failed.");
        }
    }
}
