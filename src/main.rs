use std::env;
use jisho::jisho_search::search_jisho;
use jisho::weblio_search::search_weblio;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments
    let (use_weblio, search_term) = parse_args(&args);
    
    let result = if use_weblio {
        search_weblio(&search_term)
    } else {
        search_jisho(&search_term)
    };
    
    match result {
        Ok(definition) => println!("\n{}\n", definition),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn parse_args(args: &[String]) -> (bool, String) {
    if args.len() < 2 {
        eprintln!("Usage: {} [-j] <japanese_word>", args[0]);
        eprintln!("  -j: Use Japanese monolingual dictionary (Weblio)");
        std::process::exit(1);
    }
    
    if args.len() == 2 {
        // No flags, just the search term
        (false, args[1].clone())
    } else if args.len() == 3 {
        // Check for -j flag
        if args[1] == "-j" {
            (true, args[2].clone())
        } else {
            eprintln!("Unknown flag: {}", args[1]);
            eprintln!("Usage: {} [-j] <japanese_word>", args[0]);
            std::process::exit(1);
        }
    } else {
        eprintln!("Too many arguments");
        eprintln!("Usage: {} [-j] <japanese_word>", args[0]);
        std::process::exit(1);
    }
}

