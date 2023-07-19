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
#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for creating line ups

    #[test]
    fn test_team_fitler() {
        let players: Vec<Player> = load_in_fd_csv("fanduel.csv", &[String::from("PIT")]);
        for player in players {
            assert_eq!(player.team, "PIT");
        }
    }

    #[test]
    fn test_mean() {
        let mean = mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mean, Some(3.0));
    }
}
