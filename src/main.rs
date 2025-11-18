mod tokens;
mod tokenizer;

use std::env;
use std::process::exit;
use std::fs::File;
use std::io::prelude::*;
use crate::tokenizer::Tokenizer;

fn main() {
    // first lets get args
    let argv = env::args();
    let args = argv.len();
    let filename: String;
    match args {
        2 => {
            filename = argv.last().unwrap_or(String::from(""));
        }
        _ => {
            println!("Usage: m [file]");
            exit(1);
        }
    }
    if filename.is_empty() {
        exit(1)
    }

    let mut source_file = File::open(filename.as_str()).unwrap();
    let mut contents = String::new();

    source_file.read_to_string(&mut contents).expect(""); // TODO: panic message

    let mut tokenizer = Tokenizer::new(contents.as_str());
    let r = tokenizer.generate_tokens();
    if r.is_err() {
        println!("Error: {}", r.err().unwrap());
        exit(1);
    }
    for a in tokenizer.get_tokens() {
        println!("token {:?}: {:?}", a.token_type, a.literal);
    }
    println!("Hello, world!");
}
