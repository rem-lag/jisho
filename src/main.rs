use std::env;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use colored::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <japanese_word>", args[0]);
        std::process::exit(1);
    }
    
    let search_term = &args[1];
    
    match search_jisho(search_term) {
        Ok(definition) => println!("\n{}\n", definition),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn search_jisho(term: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://www.edrdg.org/cgi-bin/wwwjdic/wwwjdic?1ZUJ{}", term);
    
    let response = get(&url)?;
    let html = response.text()?;
    
    let document = Html::parse_document(&html);
    let selector = Selector::parse("pre").unwrap();
    
    let mut formatted_entries = Vec::new();

    for element in document.select(&selector) {
        let text = element.text().collect::<String>();
        if !text.trim().is_empty() {
            let lines: Vec<&str> = text.trim().lines().collect();
            for line in lines {
                if !line.trim().is_empty() {
                    formatted_entries.push(format_entry(line));
                }
            }
        }
    }
    
    if formatted_entries.is_empty() {
        Ok("No definitions found.".to_string())
    } else {
        Ok(formatted_entries.join("\n\n"))
    }
}

fn format_entry(entry: &str) -> String {
    // Parse entry format: "入る(P);這入る(rK) [はいる] /(v5r,vi) (1) (ant: 出る・1) to enter/to come in/to go in/to get in/to arrive/(v5r,vi) (2) to join (a club, company, etc.)/..."
    let parts: Vec<&str> = entry.split(" /").collect();
    if parts.len() < 2 {
        return entry.to_string();
    }
    
    let word_reading = parts[0];
    let definition_part = parts[1];
    
    // First, extract the part of speech (first parentheses)
    let pos_start = definition_part.find('(');
    let pos_end = definition_part.find(')');
    
    if pos_start.is_none() || pos_end.is_none() {
        return entry.to_string();
    }
    
    let pos_start = pos_start.unwrap();
    let pos_end = pos_end.unwrap();
    
    let pos = &definition_part[pos_start..=pos_end];
    let after_pos = &definition_part[pos_end + 1..];
    
    // Parse definitions - handle both numbered and non-numbered
    let mut definitions = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = after_pos.chars().collect();
    
    // First, collect all numbered definitions
    let mut temp_pos = 0;
    while temp_pos < chars.len() {
        // Skip whitespace and slashes
        while temp_pos < chars.len() && (chars[temp_pos] == '/' || chars[temp_pos].is_whitespace()) {
            temp_pos += 1;
        }
        
        if temp_pos >= chars.len() {
            break;
        }
        
        // Check if we're at a numbered definition: (number)
        if chars[temp_pos] == '(' {
            let mut end_paren = temp_pos + 1;
            while end_paren < chars.len() && chars[end_paren] != ')' {
                end_paren += 1;
            }
            
            if end_paren < chars.len() {
                let inside_parens: String = chars[temp_pos + 1..end_paren].iter().collect();
                
                // Check if it's a number
                if inside_parens.trim().chars().all(|c| c.is_ascii_digit()) {
                    // This is a numbered definition
                    temp_pos = end_paren + 1;
                    
                    // Skip whitespace
                    while temp_pos < chars.len() && chars[temp_pos].is_whitespace() {
                        temp_pos += 1;
                    }
                    
                    // Collect definition text until next /(something) or end
                    let mut def_text = String::new();
                    let mut paren_depth = 0;
                    
                    while temp_pos < chars.len() {
                        let ch = chars[temp_pos];
                        
                        if ch == '(' {
                            paren_depth += 1;
                            def_text.push(ch);
                        } else if ch == ')' {
                            paren_depth -= 1;
                            def_text.push(ch);
                        } else if ch == '/' && paren_depth == 0 {
                            // Check if this is followed by a part of speech or number
                            let mut next_pos = temp_pos + 1;
                            while next_pos < chars.len() && chars[next_pos].is_whitespace() {
                                next_pos += 1;
                            }
                            
                            if next_pos < chars.len() && chars[next_pos] == '(' {
                                let mut end_next_paren = next_pos + 1;
                                while end_next_paren < chars.len() && chars[end_next_paren] != ')' {
                                    end_next_paren += 1;
                                }
                                
                                if end_next_paren < chars.len() {
                                    let next_inside: String = chars[next_pos + 1..end_next_paren].iter().collect();
                                    
                                    // If it's a part of speech (contains letters) or number, this ends the definition
                                    if next_inside.chars().any(|c| c.is_alphabetic()) || next_inside.trim().chars().all(|c| c.is_ascii_digit()) {
                                        break;
                                    }
                                }
                            }
                            def_text.push(ch);
                        } else {
                            def_text.push(ch);
                        }
                        
                        temp_pos += 1;
                    }
                    
                    let cleaned_def = def_text.trim().to_string();
                    if !cleaned_def.is_empty() {
                        definitions.push(cleaned_def);
                    }
                } else {
                    // Not a numbered definition, skip this parenthetical
                    temp_pos = end_paren + 1;
                }
            } else {
                temp_pos += 1;
            }
        } else {
            temp_pos += 1;
        }
    }
    
    // If no numbered definitions found, look for non-numbered definition
    if definitions.is_empty() {
        // Skip whitespace and slashes at the beginning
        while current_pos < chars.len() && (chars[current_pos] == '/' || chars[current_pos].is_whitespace()) {
            current_pos += 1;
        }
        
        // Collect all remaining text as a single definition
        let mut def_text = String::new();
        while current_pos < chars.len() {
            let ch = chars[current_pos];
            if ch == '/' && current_pos + 1 < chars.len() && chars[current_pos + 1] == '/' {
                // Skip double slash at end
                break;
            }
            def_text.push(ch);
            current_pos += 1;
        }
        
        let cleaned_def = def_text.trim().trim_end_matches('/').trim().to_string();
        if !cleaned_def.is_empty() {
            definitions.push(cleaned_def);
        }
    }
    
    // Format output with colors
    let mut result = String::new();
    result.push_str(&word_reading.bright_cyan().to_string());
    result.push('\n');
    result.push_str(&pos.bright_yellow().to_string());
    result.push('\n');
    
    // Format definitions
    if definitions.len() == 1 {
        result.push_str(&format!("  {}", definitions[0].bright_white()));
    } else {
        for (i, def) in definitions.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&format!("  {} {}", 
                format!("({})", i + 1).bright_magenta(),
                def.bright_white()));
        }
    }
    
    result
}
