mod tokens;
mod tokenizer;
mod parser;
mod ast;
mod symbols;
mod resolver;
mod type_checker;
mod visitor;
mod error;

use std::env;
use std::process::exit;
use std::fs::File;
use std::io::prelude::*;
use crate::parser::Parser;
use crate::resolver::SymbolResolver;
use crate::tokenizer::Tokenizer;
use crate::visitor::ASTVisitor;

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
    println!("TOKENS: ==================================\n");
    for a in tokenizer.get_tokens() {
        println!("token {:?}: {:?}", a.token_type, a.literal);
    }
    let mut parser = Parser::new(tokenizer.get_tokens().clone(), &contents); // TODO: no need to clone
    let b = parser.parse_program();
    println!("AST: ==================================\n");
    if b.is_ok() {
        let mut a = b.unwrap();
        println!("{:?}", a);
        let mut resolver = SymbolResolver::new(&contents);
        resolver.visit_program(&mut a);
        println!("Symbol Resolver: ==================================\n");
        for res_erro in resolver.errors {
            println!("{}", res_erro);
        }
    } else {
        print!("{}", b.err().unwrap())
    }
}
