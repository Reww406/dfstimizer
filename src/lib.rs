use std::{fs, sync::Arc};

use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use rusqlite::Connection;

use crate::player::*;

pub mod data_loader;
pub mod lineup;
pub mod optimizer;
pub mod player;
pub mod tables;

pub const DATABASE_FILE: &str = "./dfs_nfl.db3";
const SEASON: i16 = 2022;
const WEEK: i8 = 18;

lazy_static! {
    pub static ref QB_ATT_MAX: f32 = get_max_min(SEASON, WEEK, false, "atts", "qb_proj");
    pub static ref QB_ATT_MIN: f32 = get_max_min(SEASON, WEEK, true, "atts", "qb_proj");
    pub static ref RB_ATT_MAX: f32 = get_max_min(SEASON, WEEK, false, "atts", "rb_proj");
    pub static ref RB_ATT_MIN: f32 = get_max_min(SEASON, WEEK, true, "atts", "rb_proj");
    pub static ref WR_TGTS_MAX: f32 = get_max_min(SEASON, WEEK, false, "tgts", "wr_proj");
    pub static ref WR_TGTS_MIN: f32 = get_max_min(SEASON, WEEK, true, "tgts", "wr_proj");
    pub static ref TE_TGTS_MAX: f32 = get_max_min(SEASON, WEEK, false, "tgts", "te_proj");
    pub static ref TE_TGTS_MIN: f32 = get_max_min(SEASON, WEEK, true, "tgts", "te_proj");

    pub static ref QB_PTS_MAX: f32 = get_max_min(SEASON, WEEK, false, "pts", "qb_proj");
    pub static ref QB_PTS_MIN: f32 = get_max_min(SEASON, WEEK, true, "pts", "qb_proj");
    pub static ref RB_PTS_MAX: f32 = get_max_min(SEASON, WEEK, false, "pts", "rb_proj");
    pub static ref RB_PTS_MIN: f32 = get_max_min(SEASON, WEEK, true, "pts", "rb_proj");
    pub static ref WR_PTS_MAX: f32 = get_max_min(SEASON, WEEK, false, "pts", "wr_proj");
    pub static ref WR_PTS_MIN: f32 = get_max_min(SEASON, WEEK, true, "pts", "wr_proj");
    pub static ref TE_PTS_MAX: f32 = get_max_min(SEASON, WEEK, false, "pts", "te_proj");
    pub static ref TE_PTS_MIN: f32 = get_max_min(SEASON, WEEK, true, "pts", "te_proj");
}

fn get_max_min(season: i16, week: i8, min: bool, field: &str, table: &str) -> f32 {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let max_or_min = if min == true { "MIN" } else { "MAX" };
    let mut statement = conn
        .prepare(
            format!(
                "SELECT {}({}) FROM {} WHERE week = ?1 AND season = ?2",
                max_or_min, field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");
    statement.query_row((week, season), |r| r.get(0)).unwrap()
}
pub fn mean(data: &[f32]) -> Option<f32> {
    let count: usize = data.len();
    if count == 0 {
        return None;
    }
    let sum = data.iter().sum::<f32>();

    Some(sum / count as f32)
}

pub fn load_in_ownership(
    path: &str,
    week: i8,
    season: i16,
    teams: &[String],
) -> Vec<Arc<LitePlayer>> {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut players: Vec<Arc<LitePlayer>> = Vec::new();
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
        let lite_player: LitePlayer = LitePlayer::new(record, &conn);
        if proj_exists(lite_player.id, week, season, lite_player.pos, &conn) {
            println!("Removing Player: {}", lite_player.id);
            players.push(Arc::new(lite_player));
        }
    }
    players
}

pub fn return_if_field_exits(
    field: Option<Arc<LitePlayer>>,
    set_to: &Arc<LitePlayer>,
) -> Arc<LitePlayer> {
    if field.is_some() {
        panic!("Tried to set {:?} when one already exits", set_to.pos);
    }
    set_to.clone()
}

pub fn factorial(num: usize) -> BigUint {
    let mut fact: BigUint = 1.to_biguint().unwrap();
    for i in 2..num + 1 {
        let ib: BigUint = i.to_biguint().unwrap();
        fact *= &ib;
    }
    fact
}

pub fn total_comb(len: usize, sample: usize) -> u32 {
    if sample > len {
        return sample as u32;
    }
    (factorial(len) / (factorial(sample) * factorial(len - sample))).to_u32_digits()[0]
}

pub fn gen_comb(players: &[Arc<LitePlayer>], sample: usize) -> Vec<Vec<Arc<LitePlayer>>> {
    if sample == 1 {
        return players
            .iter()
            .map(|x| vec![x.clone()])
            .collect::<Vec<Vec<Arc<LitePlayer>>>>();
    }
    // Break condition
    if sample == players.len() {
        return vec![players.to_vec()];
    }
    // Slice 1.. iterates by one each time?
    let mut result: Vec<Vec<Arc<LitePlayer>>> = gen_comb(&players[1..], sample - 1)
        .into_iter()
        .map(|x| [&players[..1], x.as_slice()].concat())
        .collect::<Vec<Vec<Arc<LitePlayer>>>>();

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
