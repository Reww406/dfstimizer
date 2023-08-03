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
pub fn load_in_ownership(path: &str, teams: &[String]) -> Vec<PlayerOwn> {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<PlayerOwn> = Vec::new();
    let mut player_id: i16 = 0;
    for record in rdr.records() {
        // This can be refactored into xor I think
        let mut skip = false;
        let record: csv::StringRecord = record.unwrap();
        if !teams.contains(&record[2].to_string()) {
            skip = true;
        }
        if teams[0] == "*" {
            skip = false
        }
        if skip {
            continue;
        }

        players.push(PlayerOwn::new_test(record, player_id));
        player_id += 1;
    }
    players
}

pub fn return_if_field_exits<'a>(
    field: Option<&'a PlayerOwn>,
    set_to: &'a PlayerOwn,
) -> &'a PlayerOwn {
    if field.is_some() {
        panic!("Tried to set {:?} when one already exits", set_to.pos);
    }
    set_to
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for creating line ups

    #[test]
    // fn test_team_fitler() {
    //     let players: Vec<PlayerOwn> = load_in_ownership("fd-ownership.csv", &[String::from("PIT")]);
    //     for player in players {
    //         assert_eq!(player.team_id, "PIT");
    //     }
    // }
    #[test]
    fn test_mean() {
        let mean = mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mean, Some(3.0));
    }
}
