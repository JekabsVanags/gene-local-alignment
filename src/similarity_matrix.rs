use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};

#[derive(Debug)]
pub struct SimilarityMatrix {
    pub scores: HashMap<(char, char), i32>,
}

impl SimilarityMatrix{
  pub fn get_score(&self, a: char, b: char) -> Option<i32> {
    self.scores.get(&(a, b)).or_else(|| self.scores.get(&(b, a))).copied()
  }
}

pub fn create_similarity_matrix_from_file(path: &std::path::PathBuf) -> Result<SimilarityMatrix, String>{
  let file = File::open(path).expect("Failed to open");

  let reader = BufReader::new(file);
  let mut alphabet: Vec<char> = Vec::new();
  let mut scores: HashMap<(char, char), i32> = HashMap::new();
  let mut header_found = false;
  let mut line_nr = 0;

  for line in reader.lines() {
    line_nr += 1;
    let line = line.map_err(|e| format!("Error reading line {}: {}", line_nr, e))?;
    let trimmed = line.trim();

    //Skip empty and comments
    if trimmed.is_empty() || trimmed.starts_with("#"){
      continue;
    }

    let characters: Vec<&str> = trimmed.split_whitespace().collect();

    //First line contains the alphabet
    if !header_found{
      for character in &characters{
        if let Some(item) = character.chars().next(){
          alphabet.push(item)
        }
      }

      header_found = true;
      continue;
    }

    //Next lines contain data
    
    let row_char: char = characters[0].chars().next().expect("No amino acid"); //Amino acid of row
    let score_values: Vec<i32> = characters[1..].iter().map(|value| value.parse::<i32>().map_err(|e: std::num::ParseIntError| e.to_string())).collect::<Result<Vec<i32>, String>>()?;

    for (col_idx, &col_char) in alphabet.iter().enumerate() {
            scores.insert((row_char, col_char), score_values[col_idx]);
    }
  }

  Ok(SimilarityMatrix {scores})
}