use std::{collections::HashMap, error::Error, hash::Hash};

use serde::{Deserialize, Serialize};

// TODO add max/min variance, high sacks for def,
// TODO throws to end zone
// TODO avg attempts, rec targets need to be pulled from stats
pub struct Player {
    pub id: i16,
    pub name: String,
    pub team: String,
    pub pos: Pos,
}

pub struct Ownership {
    pub id: i16,
    pub season: i16,
    pub week: i8,
    pub name: String,
    pub team: String,
    pub opp: String,
    pub pos: String,
    pub salary: i32,
    pub own_per: f32
}

pub struct RbProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points: f32,
    pub avg_att: f32,
    pub td: f32,
    pub yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

pub struct QbProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points: f32,
    pub com: f32,
    pub int: f32,
    pub passing_yds: f32,
    pub passing_tds: f32,
    pub rushing_yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

pub struct RecProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points: String,
    pub avg_rec: f32,
    pub avg_tgts: f32,
    pub td: f32,
    pub yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Copy)]
pub enum Pos {
    Qb = 0,
    Rb = 1,
    Wr = 2,
    Te = 3,
    D = 4,
}

impl Pos {
    pub fn from_str(input: &str) -> Result<Pos, ()> {
        match input {
            "QB" => Ok(Pos::Qb),
            "RB" => Ok(Pos::Rb),
            "WR" => Ok(Pos::Wr),
            "TE" => Ok(Pos::Te),
            "D" => Ok(Pos::D),
            _ => Err(()),
        }
    }

    pub fn to_str(&self) -> Result<&str, ()> {
        match self {
            Pos::D => Ok("D"),
            Pos::Qb => Ok("QB"),
            Pos::Wr => Ok("WR"),
            Pos::Te => Ok("TE"),
            Pos::Rb => Ok("RB"),
            _ => Err(()),
        }
    }
}
pub struct DefProj {}

// Can we do just ID
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LitePlayer {
    // Only POS and SAlARY load rest in the last function
    // We will generate our own once we load data into sqlite
    pub id: i16,
    pub pos: Pos,
    pub salary: i16,
}

// Id 0, player 1, team 2, opp 3, pos 4, salary 5, own 6
impl LitePlayer {
    pub fn new_test(record: csv::StringRecord, id: i16) -> Self {
        LitePlayer {
            id: id, // fetch from database
            pos: Pos::from_str(&record[4].to_string()).expect("Couldn't convert error"),
            salary: record[5].parse::<i16>().expect("Salary Missing"),
        }
    }

    // Could make this a singleton so it's only generated once
    pub fn player_lookup_map(players: &[LitePlayer]) -> HashMap<i16, &LitePlayer> {
        let mut lookup_map = HashMap::new();
        players.iter().for_each(|p| {
            lookup_map.insert(p.id, p);
        });
        lookup_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;

    #[test]
    fn test_enum_compartor() {
        let pos: Pos = Pos::from_str("QB").unwrap();
        assert!(pos == Pos::Qb)
    }

    //85546-69531,Jalen Hurts,PHI,NYG,QB,9000,18.6
    // #[test]
    // fn test_new_from_fd() {
    //     let test_record: StringRecord = StringRecord::from(vec![
    //         "85546-69531",
    //         "Jalen Hurts",
    //         "PHI",
    //         "NYG",
    //         "QB",
    //         "9000",
    //         "18.6",
    //     ]);
    //     let player: PlayerOwn = PlayerOwn::new(test_record.clone());
    //     assert_eq!(player.name_id, test_record[1].to_string());
    //     assert_eq!(player.team_id, test_record[2].to_string());
    //     assert_eq!(player.opp_id, test_record[3].to_string());
    //     assert_eq!(player.pos, test_record[4].to_string());
    //     assert_eq!(
    //         player.salary,
    //         test_record[5].parse::<i32>().expect("Missing salary")
    //     );
    //     assert_eq!(
    //         player.own_per,
    //         test_record[6].parse::<f32>().expect("Missing Own Per")
    //     );
    // }
}
