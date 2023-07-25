use std::fs;

use crate::player::*;

pub mod lineup;
pub mod optimizer;
pub mod player;

// &[T] can either be array or vec
pub fn mean(data: &[f32]) -> Option<f32> {
    let count: usize = data.len();
    if count == 0 {
        return None;
    }
    let sum = data.iter().sum::<f32>();

    Some(sum / count as f32)
}
pub fn load_in_fd_csv(path: &str, teams: &[String], week: i16) -> Vec<Player> {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<Player> = Vec::new();

    for record in rdr.records() {
        let record: csv::StringRecord = record.unwrap();
        if !teams.contains(&record[2].to_string()) {
            continue;
        }
        
        if &record[4].parse::<i16>().unwrap() != &week {
            continue;
        }

        players.push(Player::new_from_fd(record));
    }
    players
}

pub fn return_if_field_exits<'a>(field: Option<&'a Player>, set_to: &'a Player) -> &'a Player {
    if field.is_some() {
        panic!("Tried to set {} when one already exits", set_to.pos);
    }
    set_to
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for creating line ups

    #[test]
    fn test_team_fitler() {
        let players: Vec<Player> = load_in_fd_csv("fanduel.csv", &[String::from("PIT")], 1);
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
