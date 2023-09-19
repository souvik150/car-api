use std::fs::File;
use std::io::prelude::*;

pub fn read_json_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
  // Open the JSON file.
  let mut file = File::open(file_path)?;

  // Read the file content into a String.
  let mut json_string = String::new();
  file.read_to_string(&mut json_string)?;

  Ok(json_string)
}