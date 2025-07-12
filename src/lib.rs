pub mod jisho_search;
pub mod weblio_search;

use std::error::Error;

pub type DictionaryResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    pub word_reading: String,
    pub part_of_speech: String,
    pub definitions: Vec<String>,
    pub synonyms: Vec<String>,
}

impl DictionaryEntry {
    pub fn new(word_reading: String, part_of_speech: String, definitions: Vec<String>) -> Self {
        Self {
            word_reading,
            part_of_speech,
            definitions,
            synonyms: Vec::new(),
        }
    }

    pub fn with_synonyms(mut self, synonyms: Vec<String>) -> Self {
        self.synonyms = synonyms;
        self
    }
}