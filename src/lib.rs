use std::fs;

use crate::models::*;

pub mod models;
pub mod optimizer;

// &[T] can either be array or vec
pub fn mean(data: &[f32]) -> Option<f32> {
    let count: usize = data.len();
    if count == 0 {
        return None;
    }
    let sum = data.iter().sum::<f32>();

    Some(sum / count as f32)
}

pub fn load_in_csv(path: &str) -> Vec<Player> {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<Player> = Vec::new();
    for record in rdr.records() {
        let record: csv::StringRecord = record.unwrap();
        players.push(Player::new(
            record[4].to_string(),
            record[2].parse::<i16>().unwrap_or_default(),
            record[3].parse::<f32>().unwrap_or_default(),
            record[6].to_string(),
            record[7].parse::<f32>().unwrap_or_default(),
        ))
    }
    players
}

pub fn load_in_csv_buff_test(path: &str) -> Vec<Player> {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<Player> = Vec::new();
    for record in rdr.records() {
        let record: csv::StringRecord = record.unwrap();
        if record[0].parse::<i32>().unwrap() != 1 {
            continue;
        }
        if record[5].to_string() != "Buffalo Bills" && record[5].to_string() != "New York Jets" {
            continue;
        }
        players.push(Player::new(
            record[4].to_string(),
            record[2].parse::<i16>().unwrap_or_default(),
            record[3].parse::<f32>().unwrap_or_default(),
            record[6].to_string(),
            record[7].parse::<f32>().unwrap_or_default(),
        ))
    }
    players.push(Player::new(
        String::from("Buffalo Bills"),
        4000,
        10.1,
        String::from("def"),
        14.0,
    ));
    players.push(Player::new(
        String::from("New York Jets"),
        4300,
        12.1,
        String::from("def"),
        32.0,
    ));
    players
}
