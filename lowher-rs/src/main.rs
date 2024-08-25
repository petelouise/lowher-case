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

fn process_text(text: &str, preserve_capitalized: bool, preserve_sentence_case: bool) -> String {
    let sentence_pattern = Regex::new(r"(?:^|[.!?]\s+)([A-Z][^.!?]*(?:[.!?]|$))").unwrap();
    let word_pattern = Regex::new(r"\b\w+\b").unwrap();
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for sentence_cap in sentence_pattern.captures_iter(text) {
        let sentence = sentence_cap.get(1).unwrap();
        let sentence_start = sentence.start();
        let sentence_end = sentence.end();

        // Add any text before the sentence
        result.push_str(&text[last_end..sentence_start]);

        let mut sentence_result = String::with_capacity(sentence.len());
        let mut sentence_last_end = 0;

        for word_cap in word_pattern.captures_iter(sentence.as_str()) {
            let word = word_cap.get(0).unwrap();
            let word_start = word.start();
            let word_end = word.end();

            // Add any text between the last word and this one
            sentence_result.push_str(&sentence.as_str()[sentence_last_end..word_start]);

            let word_str = word.as_str();
            let is_first_word = word_start == 0;

            if word_str.chars().all(char::is_uppercase)
                || (preserve_capitalized
                    && word_str.chars().next().unwrap().is_uppercase()
                    && !is_first_word)
                || (preserve_sentence_case && is_first_word)
            {
                sentence_result.push_str(word_str);
            } else {
                sentence_result.push_str(&word_str.to_lowercase());
            }

            sentence_last_end = word_end;
        }

        // Add any remaining text in the sentence
        sentence_result.push_str(&sentence.as_str()[sentence_last_end..]);

        // If not preserving sentence case, lowercase the first character
        if !preserve_sentence_case {
            if let Some(first_char) = sentence_result.chars().next() {
                let lowercased = first_char.to_lowercase().collect::<String>();
                sentence_result.replace_range(0..1, &lowercased);
            }
        }

        result.push_str(&sentence_result);
        last_end = sentence_end;
    }

    // Add any remaining text
    result.push_str(&text[last_end..]);
    result
}

fn lowher(text: &str, preserve_capitalized: bool, preserve_sentence_case: bool) -> String {
    let (marked_text, placeholders, code_blocks) = mark_code_blocks(text);
    let processed_text = process_text(&marked_text, preserve_capitalized, preserve_sentence_case);
    unmark_code_blocks(&processed_text, &placeholders, &code_blocks)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut preserve_capitalized = true;
    let mut preserve_sentence_case = false;
    let mut input_source = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-a" | "--lowercase-all" => preserve_capitalized = false,
            "-s" | "--preserve-sentence-case" => preserve_sentence_case = true,
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

    let output_text = lowher(&content, preserve_capitalized, preserve_sentence_case);
    println!("{}", output_text);

    Ok(())
}

enum InputSource {
    File(String),
    Stdin,
}

fn run_test() {
    let test_string = "This is a TEST String with ACRONYMS like NASA and proper Nouns like John Doe. \
                       Here's some `inlineCode` and a code block:
                       ```
                       function testFunction() {
                           console.log('HELLO WORLD');
                       }
                       ```
                       More TEXT here. Let's include an email: John.Doe@Example.com and a URL: https://www.Example.com. \
                       Another sentence. And one more.";

    println!("Original text:");
    println!("{}\n", test_string);

    println!(
        "Processed text (preserving capitalized words, lowercasing first letter of sentences):"
    );
    println!("{}\n", lowher(test_string, true, false));

    println!("Processed text (lowercasing all words, lowercasing first letter of sentences):");
    println!("{}\n", lowher(test_string, false, false));

    println!("Processed text (preserving capitalized words and sentence case):");
    println!("{}\n", lowher(test_string, true, true));

    println!("Processed text (lowercasing all words, preserving sentence case):");
    println!("{}", lowher(test_string, false, true));
}

fn print_help() {
    println!("Lowher - Convert text to lowercase while optionally preserving proper nouns, sentence case, and code blocks");
    println!("\nUsage:");
    println!("  lowher [OPTIONS] [<filename> | -]");
    println!("\nOptions:");
    println!("  -a, --lowercase-all           Lowercase all words, including those starting with capital letters");
    println!(
        "  -s, --preserve-sentence-case  Preserve the case of the first letter in each sentence"
    );
    println!("  --help                        Print this help message");
    println!("  -                             Read from stdin instead of a file");
    println!("\nDescription:");
    println!(
        "  Lowher reads the content of the specified file or from stdin, converts it to lowercase"
    );
    println!("  while optionally preserving the case of proper nouns and sentence beginnings, always preserving");
    println!("  acronyms and code blocks. The result is printed to stdout.");
    println!("\nExamples:");
    println!("  lowher input.txt > output.txt");
    println!("  lowher -a input.txt > output_all_lowercase.txt");
    println!("  lowher -s input.txt > output_preserve_sentence_case.txt");
    println!("  pbpaste | lowher - > output.txt");
    println!("  echo 'Some TEXT. Another sentence.' | lowher");
}
