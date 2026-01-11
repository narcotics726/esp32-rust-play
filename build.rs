use std::fs;
use std::io::{BufRead, BufReader};

fn main() {
    let env_file = std::path::Path::new(".env");
    if env_file.exists() {
        let file = fs::File::open(env_file).expect("Failed to open .env file");
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("#") || line.trim().is_empty() {
                continue; // Skip comments and empty lines
            }
            if let Some((key, value)) = line.split_once('=') {
                println!("cargo:rustc-env={}={}", key.trim(), value.trim());
            }
        }
    }
    embuild::espidf::sysenv::output();
}
