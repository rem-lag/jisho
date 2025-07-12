use crate::{DictionaryEntry, DictionaryResult};
use reqwest::blocking::get;
use scraper::{Html, Selector, ElementRef};
use colored::*;

pub fn search_weblio(term: &str) -> DictionaryResult<String> {
    let url = format!("https://www.weblio.jp/content/{}", term);
    
    let response = get(&url)?;
    let html = response.text()?;
    
    let document = Html::parse_document(&html);
    
    
    let entries = parse_weblio_entries(&document)?;
    
    if entries.is_empty() {
        Ok("No definitions found.".to_string())
    } else {
        let formatted_entries: Vec<String> = entries.iter()
            .map(|entry| format_weblio_entry(entry))
            .collect();
        Ok(formatted_entries.join("\n\n"))
    }
}

fn parse_weblio_entries(document: &Html) -> DictionaryResult<Vec<DictionaryEntry>> {
    let mut entries = Vec::new();
    
    // Find all dictionary entry headers
    let header_selector = Selector::parse("h2.midashigo").unwrap();
    
    for header in document.select(&header_selector) {
        if let Some(entry) = parse_single_entry(header, document)? {
            entries.push(entry);
        }
    }
    
    Ok(entries)
}

fn parse_single_entry(header: ElementRef, _document: &Html) -> DictionaryResult<Option<DictionaryEntry>> {
    // Extract kanji/word from title attribute
    let word = header.value().attr("title").unwrap_or("").to_string();
    
    // Extract reading from header text
    let header_text = header.text().collect::<String>();
    let reading = extract_reading_from_header(&header_text);
    
    // Skip example usage headers
    if header_text.contains("例文・使い方・用例・文例") {
        return Ok(None);
    }
    
    // Find the content div that follows this header
    let mut content_div = None;
    
    // Look for the content div by traversing all siblings
    let mut current_node = header.next_sibling();
    while let Some(node) = current_node {
        if let Some(element) = ElementRef::wrap(node) {
            let class_attr = element.value().attr("class").unwrap_or("");
            if element.value().name() == "div" && class_attr.contains("Sgkdj") {
                content_div = Some(element);
                break;
            }
        }
        current_node = node.next_sibling();
    }
    
    if let Some(content) = content_div {
        let reading_text = extract_reading_from_content(&content);
        let part_of_speech = extract_part_of_speech(&content);
        let definitions = extract_definitions(&content);
        let synonyms = extract_synonyms(&content);
        
        
        let word_reading = if !word.is_empty() && !reading_text.is_empty() {
            format!("{}【{}】", reading_text, word)
        } else if !reading.is_empty() {
            reading
        } else {
            word
        };
        
        if !definitions.is_empty() {
            let entry = DictionaryEntry::new(word_reading, part_of_speech, definitions)
                .with_synonyms(synonyms);
            return Ok(Some(entry));
        }
    }
    
    Ok(None)
}

fn extract_reading_from_header(header_text: &str) -> String {
    // Extract reading from format like "せい‐かい【正解】"
    if let Some(bracket_pos) = header_text.find('【') {
        header_text[..bracket_pos].replace('‐', "")
    } else {
        header_text.to_string()
    }
}

fn extract_reading_from_content(content: &ElementRef) -> String {
    let mut readings = Vec::new();
    
    // Look for paragraph with reading pattern
    let p_selector = Selector::parse("p").unwrap();
    for p in content.select(&p_selector) {
        let p_text = p.text().collect::<String>();
        if p_text.contains("読み方：") {
            if let Some(start) = p_text.find("読み方：") {
                let after_marker = &p_text[start + "読み方：".len()..];
                readings.push(after_marker.trim().to_string());
            }
        }
        
        // Look for alternative readings like 《「いぞん」とも》
        if let Some(start) = p_text.find("《「") {
            if let Some(end) = p_text.find("」とも》") {
                let alt_reading = &p_text[start + "《「".len()..end];
                if !alt_reading.is_empty() {
                    readings.push(alt_reading.to_string());
                }
            }
        }
        
        // Look for pronunciation notes like 「ふいんき」と発音する
        if let Some(start) = p_text.find("「") {
            if let Some(end) = p_text.find("」と発音する") {
                let alt_reading = &p_text[start + "「".len()..end];
                if !alt_reading.is_empty() && alt_reading.chars().all(|c| "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽゃゅょっー".contains(c)) {
                    readings.push(alt_reading.to_string());
                }
            }
        }
    }
    
    if readings.is_empty() {
        String::new()
    } else if readings.len() == 1 {
        readings[0].clone()
    } else {
        // Join multiple readings with "・"
        readings.join("・")
    }
}

fn extract_part_of_speech(content: &ElementRef) -> String {
    let hinshi_selector = Selector::parse("span.hinshi").unwrap();
    
    for span in content.select(&hinshi_selector) {
        let text = span.text().collect::<String>();
        if !text.trim().is_empty() {
            return text.trim().to_string();
        }
    }
    
    String::new()
}

fn extract_definitions(content: &ElementRef) -> Vec<String> {
    let mut definitions = Vec::new();
    
    // Look for numbered definitions marked with <b>１</b>, <b>２</b>, etc.
    let b_selector = Selector::parse("b").unwrap();
    let mut numbered_defs = Vec::new();
    
    for b_tag in content.select(&b_selector) {
        let b_text = b_tag.text().collect::<String>();
        if b_text.trim().chars().all(|c| "１２３４５６７８９０".contains(c)) {
            // This is a numbered definition marker
            if let Some(parent) = b_tag.parent() {
                if let Some(p_element) = ElementRef::wrap(parent) {
                    if p_element.value().name() == "p" {
                        let p_text = p_element.text().collect::<String>();
                        // Extract text after the number
                        if let Some(start) = p_text.find(&b_text) {
                            let after_number = &p_text[start + b_text.len()..];
                            let def = after_number.trim().to_string();
                            if !def.is_empty() {
                                numbered_defs.push(def);
                            }
                        }
                    }
                }
            }
        }
    }
    
    if !numbered_defs.is_empty() {
        definitions.extend(numbered_defs);
    } else {
        // Look for definition paragraphs
        let p_selector = Selector::parse("p").unwrap();
        for p in content.select(&p_selector) {
            let p_text = p.text().collect::<String>();
            let cleaned = p_text.trim();
            
            
            // Skip reading markers
            if cleaned.starts_with("読み方：") {
                continue;
            }
            
            // For paragraphs that contain part of speech markers, extract the definition part
            if cleaned.contains("［") && cleaned.contains("］") {
                // Find the end of the part of speech section - look for either ）or 》
                if let Some(pos) = cleaned.find("》") {
                    let chars: Vec<char> = cleaned.chars().collect();
                    let char_pos = cleaned[..pos].chars().count();
                    if char_pos + 1 < chars.len() {
                        let after_pos: String = chars[char_pos + 1..].iter().collect();
                        let def = after_pos.trim().to_string();
                        if !def.is_empty() && def.len() > 10 {
                            definitions.push(def);
                        }
                    }
                } else if let Some(pos) = cleaned.find("）") {
                    let chars: Vec<char> = cleaned.chars().collect();
                    let char_pos = cleaned[..pos].chars().count();
                    if char_pos + 1 < chars.len() {
                        let after_pos: String = chars[char_pos + 1..].iter().collect();
                        let def = after_pos.trim().to_string();
                        if !def.is_empty() && def.len() > 10 {
                            definitions.push(def);
                        }
                    }
                }
            } else if !cleaned.is_empty() && 
                      cleaned.len() > 10 &&
                      !cleaned.chars().all(|c| "１２３４５６７８９０".contains(c)) {
                // Check if this looks like a definition (contains Japanese characters)
                if cleaned.chars().any(|c| c.is_ascii_punctuation() || "。、".contains(c)) {
                    definitions.push(cleaned.to_string());
                }
            }
        }
    }
    
    definitions
}

fn extract_synonyms(content: &ElementRef) -> Vec<String> {
    let mut synonyms = Vec::new();
    
    let synonym_selector = Selector::parse("div.synonymsUnderDict a").unwrap();
    for link in content.select(&synonym_selector) {
        let synonym = link.text().collect::<String>();
        if !synonym.trim().is_empty() {
            synonyms.push(synonym.trim().to_string());
        }
    }
    
    synonyms
}

fn format_weblio_entry(entry: &DictionaryEntry) -> String {
    let mut result = String::new();
    
    // Word and reading in cyan
    result.push_str(&entry.word_reading.bright_cyan().to_string());
    result.push('\n');
    
    // Part of speech in yellow (if available)
    if !entry.part_of_speech.is_empty() {
        result.push_str(&entry.part_of_speech.bright_yellow().to_string());
        result.push('\n');
    }
    
    // Format definitions
    if entry.definitions.len() == 1 {
        result.push_str(&format!("  {}", entry.definitions[0].bright_white()));
    } else {
        for (i, def) in entry.definitions.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&format!("  {} {}", 
                format!("({})", i + 1).bright_magenta(),
                def.bright_white()));
        }
    }
    
    // Add synonyms if available
    if !entry.synonyms.is_empty() {
        result.push('\n');
        result.push_str(&format!("  {} {}", 
            "類語:".bright_green(),
            entry.synonyms.join(", ").bright_white()));
    }
    
    result
}