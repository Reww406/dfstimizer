use std::fs;

use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};

use crate::player::*;

pub mod data_loader;
pub mod lineup;
pub mod optimizer;
pub mod player;

pub const DATABASE_FILE: &str = "./dfs_nfl.db3";

pub fn mean(data: &[f32]) -> Option<f32> {
    let count: usize = data.len();
    if count == 0 {
        return None;
    }
    let sum = data.iter().sum::<f32>();

    Some(sum / count as f32)
}

pub fn load_in_ownership(path: &str, teams: &[String]) -> Vec<LitePlayer> {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<LitePlayer> = Vec::new();
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

        players.push(LitePlayer::new_test(record, player_id));
        player_id += 1;
    }
    players
}

pub fn return_if_field_exits<'a>(
    field: Option<&'a LitePlayer>,
    set_to: &'a LitePlayer,
) -> &'a LitePlayer {
    if field.is_some() {
        panic!("Tried to set {:?} when one already exits", set_to.pos);
    }
    set_to
}

pub fn factorial(num: usize) -> BigUint {
    let mut fact: BigUint = 1.to_biguint().unwrap();
    for i in 2..num + 1 {
        let ib = i.to_biguint().unwrap();
        fact *= &ib;
    }
    fact
}

pub fn total_comb(len: usize, sample: usize) -> u32 {
    (factorial(len) / (factorial(sample) * factorial(len - sample))).to_u32_digits()[0]
}

// Is this going to blow up the stack? maybe
pub fn gen_comb(players: &[LitePlayer], sample: usize) -> Vec<Vec<LitePlayer>> {
    if sample == 1 {
        return players
            .iter()
            .map(|x| vec![x.clone()])
            .collect::<Vec<Vec<LitePlayer>>>();
    }
    // Break condition
    if sample == players.len() {
        return vec![players.to_vec()];
    }
    // Slice 1.. iterates by one each time?
    let mut result = gen_comb(&players[1..], sample - 1)
        .into_iter()
        .map(|x| [&players[..1], x.as_slice()].concat())
        .collect::<Vec<Vec<LitePlayer>>>();

    result.extend(gen_comb(&players[1..], sample));

    result
}

#[cfg(test)]
mod tests {

    use num_bigint::ToBigUint;

    use super::*;
    // Helper function for creating line ups

    // #[test]
    // fn test_team_fitler() {
    //     let players: Vec<PlayerOwn> = load_in_ownership("fd-ownership.csv", &[String::from("PIT")]);
    //     for player in players {
    //         assert_eq!(player.team_id, "PIT");
    //     }
    // }

    #[test]
    fn test_total_comb() {
        assert_eq!(253, total_comb(23, 2));
    }

    #[test]
    fn test_factorial() {
        assert_eq!(5040.to_biguint().unwrap(), factorial(7));
    }

    #[test]
    fn test_mean() {
        let mean = mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mean, Some(3.0));
    }
}
