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
pub fn load_in_fd_csv(path: &str, teams: &[String]) -> Vec<Player> {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<Player> = Vec::new();

    for record in rdr.records() {
        let record: csv::StringRecord = record.unwrap();
        if !teams.contains(&record[2].to_string()) {
            continue;
        }

        players.push(Player::new_from_fd(record));
    }
    players
}


// TODO Test team filtering
// TODO Test mean function
