//! # Lowher
//!
//! Lowher is a command-line tool that converts text to lowercase while optionally
//! preserving the case of proper nouns, acronyms, and code blocks.
//!
//! ## Installation
//!
//! 1. Ensure you have Rust and Cargo installed. If not, visit https://www.rust-lang.org/tools/install
//!
//! 2. Clone this repository:
//!    ```
//!    git clone https://github.com/yourusername/lowher.git
//!    cd lowher
//!    ```
//!
//! 3. Build the project:
//!    ```
//!    cargo build --release
//!    ```
//!
//! 4. The binary will be available at `target/release/lowher`
//!
//! ## Usage
//!
//! Run the program with the input file and optional flags:
//!
//! ```
//! ./lowher [OPTIONS] <filename>
//! ```
//!
//! Options:
//!   -a, --lowercase-all    Lowercase all words, including those starting with capital letters
//!
//! The processed text will be printed to stdout. To save the output to a file, use:
//!
//! ```
//! ./lowher [OPTIONS] input.txt > output.txt
//! ```
//!
//! ## Features
//!
//! - Converts text to lowercase
//! - Preserves case of words that are all uppercase (assumed to be acronyms)
//! - Optionally preserves case of words that start with an uppercase letter (assumed to be proper nouns)
//! - Preserves text within code blocks (text between ``` or single backticks)

use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Read};

fn mark_code_blocks(text: &str) -> (String, Vec<String>, Vec<String>) {
    let code_block_pattern = Regex::new(r"(```[\s\S]*?```|`[^`]*`)").unwrap();
    let code_blocks: Vec<String> = code_block_pattern
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect();
    let placeholders: Vec<String> = (0..code_blocks.len())
        .map(|i| format!("__CODE_BLOCK_{i}__"))
        .collect();

    let mut marked_text = text.to_string();
    for (placeholder, code_block) in placeholders.iter().zip(code_blocks.iter()) {
        marked_text = marked_text.replace(code_block, placeholder);
    }

    (marked_text, placeholders, code_blocks)
}

fn unmark_code_blocks(text: &str, placeholders: &[String], code_blocks: &[String]) -> String {
    let mut unmarked_text = text.to_string();
    for (placeholder, code_block) in placeholders.iter().zip(code_blocks.iter()) {
        unmarked_text = unmarked_text.replace(placeholder, code_block);
    }
    unmarked_text
}

fn process_text(text: &str, preserve_capitalized: bool) -> String {
    let word_pattern = Regex::new(r"\b\w+\b").unwrap();
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for cap in word_pattern.captures_iter(text) {
        let word = cap.get(0).unwrap();
        let start = word.start();
        let end = word.end();

        // Add any text between the last word and this one
        result.push_str(&text[last_end..start]);

        // Check if the word is all uppercase or (if preserving) starts with uppercase
        if word.as_str().chars().all(char::is_uppercase)
            || (preserve_capitalized && word.as_str().chars().next().unwrap().is_uppercase())
        {
            result.push_str(word.as_str());
        } else {
            result.push_str(&word.as_str().to_lowercase());
        }

        last_end = end;
    }

    // Add any remaining text
    result.push_str(&text[last_end..]);
    result
}

fn lowher(text: &str, preserve_capitalized: bool) -> String {
    let (marked_text, placeholders, code_blocks) = mark_code_blocks(text);
    let processed_text = process_text(&marked_text, preserve_capitalized);
    unmark_code_blocks(&processed_text, &placeholders, &code_blocks)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut preserve_capitalized = true;
    let mut input_source = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-a" | "--lowercase-all" => preserve_capitalized = false,
            "--help" => {
                print_help();
                return Ok(());
            }
            "--test" => {
                run_test();
                return Ok(());
            }
            "-" => input_source = Some(InputSource::Stdin),
            _ if input_source.is_none() => input_source = Some(InputSource::File(arg.to_string())),
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_help();
                std::process::exit(1);
            }
        }
    }

    let content = match input_source {
        Some(InputSource::File(filename)) => fs::read_to_string(filename)?,
        Some(InputSource::Stdin) => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let output_text = lowher(&content, preserve_capitalized);
    println!("{}", output_text);

    Ok(())
}

enum InputSource {
    File(String),
    Stdin,
}

fn run_test() {
    let test_string = "This is a TEST String with ACRONYMS like NASA and proper Nouns like John Doe. \
                       Here's some `inline code` and a code block:
                       ```
                       function testFunction() {
                           console.log('HELLO WORLD');
                       }
                       ```
                       More TEXT here. Let's include an email: John.Doe@Example.com and a URL: https://www.Example.com.";

    println!("Original text:");
    println!("{}\n", test_string);

    println!("Processed text (preserving capitalized words):");
    println!("{}\n", lowher(test_string, true));

    println!("Processed text (lowercasing all words):");
    println!("{}", lowher(test_string, false));
}

fn print_help() {
    println!("Lowher - Convert text to lowercase while optionally preserving proper nouns and code blocks");
    println!("\nUsage:");
    println!("  lowher [OPTIONS] [<filename> | -]");
    println!("\nOptions:");
    println!("  -a, --lowercase-all    Lowercase all words, including those starting with capital letters");
    println!("  --help                 Print this help message");
    println!("  -                      Read from stdin instead of a file");
    println!("\nDescription:");
    println!("  Lowher reads the content of the specified file or from stdin, converts it to lowercase");
    println!("  while optionally preserving the case of proper nouns, always preserving");
    println!("  acronyms and code blocks. The result is printed to stdout.");
    println!("\nExamples:");
    println!("  lowher input.txt > output.txt");
    println!("  lowher -a input.txt > output_all_lowercase.txt");
    println!("  pbpaste | lowher - > output.txt");
    println!("  echo 'Some TEXT' | lowher");
}
