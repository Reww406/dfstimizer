use lazy_static::lazy_static;
use std::str::Split;
use std::sync::Mutex;
use std::{collections::HashMap, error::Error, hash::Hash};

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::data_loader::NON_OFF_TO_OFF_ABBR;
use crate::data_loader::*;

lazy_static! {
    pub static ref REC_PROJ_CACHE: Mutex<HashMap<i16, RecProj>> = Mutex::new(HashMap::new());
    pub static ref RB_PROJ_CACHE: Mutex<HashMap<i16, RbProj>> = Mutex::new(HashMap::new());
    pub static ref QB_PROJ_CACHE: Mutex<HashMap<i16, QbProj>> = Mutex::new(HashMap::new());
    pub static ref DEF_PROJ_CACHE: Mutex<HashMap<i16, DefProj>> = Mutex::new(HashMap::new());
}

// TODO add max/min variance, high sacks for def,
// TODO throws to end zone
// TODO avg attempts, rec targets need to be pulled from stats
#[derive(Clone)]
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
    pub own_per: f32,
}

#[derive(Clone, Debug)]
pub struct RbProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub pts: f32,
    pub atts: f32,
    pub tds: f32,
    pub rush_yds: f32,
    pub rec_yds: f32,
    pub salary: i32,
    pub own_per: f32,
}
#[derive(Clone, Debug)]
pub struct QbProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub pts: f32,
    pub atts: f32,
    pub comps: f32,
    pub ints: f32,
    pub pass_yds: f32,
    pub pass_tds: f32,
    pub rush_yds: f32,
    pub salary: i32,
    pub own_per: f32,
}
#[derive(Clone, Debug)]
pub struct RecProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub pos: Pos,
    pub pts: String,
    pub rec: f32,
    pub tgts: f32,
    pub td: f32,
    pub rec_yds: f32,
    pub rush_yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

#[derive(Debug, Clone)]
pub struct DefProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub pts: String,
    pub salary: i32,
    pub own_per: f32,
}

#[derive(Debug, Clone)]
pub struct FlexProj {
    pub pos: Pos,
    pub rec_proj: Option<RecProj>,
    pub rb_proj: Option<RbProj>,
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
        let input = input.to_uppercase();
        match input.as_str() {
            "QB" => Ok(Pos::Qb),
            "RB" => Ok(Pos::Rb),
            "WR" => Ok(Pos::Wr),
            "TE" => Ok(Pos::Te),
            "D" => Ok(Pos::D),
            "DST" => Ok(Pos::D),
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

// Can we do just ID
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LitePlayer {
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

//
// Player Queries
//
pub fn query_rec_proj(
    id: i16,
    week: i8,
    season: i16,
    pos: &Pos,
    conn: &Connection,
) -> Option<RecProj> {
    if REC_PROJ_CACHE.lock().unwrap().get(&id).is_some() {
        let proj: RecProj = REC_PROJ_CACHE.lock().unwrap().get(&id).unwrap().clone();
        return Some(proj);
    }
    let table = if pos == &Pos::Wr {
        "wr_proj"
    } else {
        "te_proj"
    };
    let query: String = format!(
        "SELECT * FROM {} WHERE id = ?1 AND week = ?2 AND season = ?3",
        table
    );
    let rec_proj: Option<RecProj> = conn
        .query_row(query.as_str(), (id, week, season), |row| {
            Ok(RecProj {
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pos: *pos,
                pts: row.get(6)?,
                rec: row.get(7)?,
                tgts: row.get(8)?,
                td: row.get(9)?,
                rec_yds: row.get(10)?,
                rush_yds: row.get(11)?,
                salary: row.get(12)?,
                own_per: row.get(13)?,
            })
        })
        .optional()
        .unwrap();
    if rec_proj.is_none() {
        return None;
    }
    REC_PROJ_CACHE
        .lock()
        .unwrap()
        .insert(id, rec_proj.clone().unwrap());
    rec_proj
}

pub fn query_rb_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<RbProj> {
    if RB_PROJ_CACHE.lock().unwrap().get(&id).is_some() {
        let proj: RbProj = RB_PROJ_CACHE.lock().unwrap().get(&id).unwrap().clone();
        return Some(proj);
    }
    let query: &str = "SELECT * FROM rb_proj WHERE id = ?1 AND week = ?2 AND season = ?3";
    let rb_proj: Option<RbProj> = conn
        .query_row(query, (id, week, season), |row| {
            Ok(RbProj {
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts: row.get(6)?,
                atts: row.get(7)?,
                tds: row.get(8)?,
                rush_yds: row.get(9)?,
                rec_yds: row.get(10)?,
                salary: row.get(11)?,
                own_per: row.get(12)?,
            })
        })
        .optional()
        .unwrap();
    if rb_proj.is_none() {
        return None;
    }
    RB_PROJ_CACHE
        .lock()
        .unwrap()
        .insert(id, rb_proj.clone().unwrap());
    rb_proj
}

pub fn query_qb_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<QbProj> {
    if QB_PROJ_CACHE.lock().unwrap().get(&id).is_some() {
        let proj: QbProj = QB_PROJ_CACHE.lock().unwrap().get(&id).unwrap().clone();
        return Some(proj);
    }
    let query: &str = "SELECT * FROM qb_proj WHERE id = ?1 AND week = ?2 AND season = ?3";
    let qb_proj: Option<QbProj> = conn
        .query_row(query, (id, week, season), |row| {
            Ok(QbProj {
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts: row.get(6)?,
                atts: row.get(7)?,
                comps: row.get(8)?,
                ints: row.get(9)?,
                pass_yds: row.get(10)?,
                pass_tds: row.get(11)?,
                rush_yds: row.get(12)?,
                salary: row.get(13)?,
                own_per: row.get(14)?,
            })
        })
        .optional()
        .unwrap();
    if qb_proj.is_none() {
        return None;
    }
    QB_PROJ_CACHE
        .lock()
        .unwrap()
        .insert(id, qb_proj.clone().unwrap());
    qb_proj
}

pub fn query_def_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<DefProj> {
    if DEF_PROJ_CACHE.lock().unwrap().get(&id).is_some() {
        let proj: DefProj = DEF_PROJ_CACHE.lock().unwrap().get(&id).unwrap().clone();
        return Some(proj);
    }
    let query: &str = "SELECT * FROM dst_proj WHERE id = ?1 AND week = ?2 AND season = ?3";
    let def_proj: Option<DefProj> = conn
        .query_row(query, (id, week, season), |row| {
            Ok(DefProj {
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts: row.get(6)?,
                salary: row.get(7)?,
                own_per: row.get(8)?,
            })
        })
        .optional()
        .unwrap();
    if def_proj.is_none() {
        return None;
    }
    DEF_PROJ_CACHE
        .lock()
        .unwrap()
        .insert(id, def_proj.clone().unwrap());
    def_proj
}

pub fn query_own_per(id: i32, week: i8, season: i16, conn: &Connection) -> Option<f32> {
    let select_ownership: &str =
        "SELECT own_per FROM ownership WHERE id = ?1 AND week = ?2 AND season = ?3";
    let own_per: Option<f32> = conn
        .query_row(select_ownership, (id, week, season), |row| row.get(0))
        .optional()
        .unwrap();
    return own_per;
}

pub fn get_player_id_create_if_missing(
    name: &String,
    team: &String,
    pos: &Pos,
    conn: &Connection,
) -> i32 {
    let id: Option<i32> = get_player_id(name, team, pos, conn);
    if id.is_some() {
        return id.unwrap();
    }
    let player: Player = Player {
        id: 0,
        name: name.clone(),
        team: team.clone(),
        pos: pos.clone(),
    };
    return load_player_id(player, conn);
}

// Get Player ID, Searches D, Then Exact, Then Fuzzy
pub fn get_player_id(name: &String, team: &String, pos: &Pos, conn: &Connection) -> Option<i32> {
    let team_conversion: Option<&&str> = NON_OFF_TO_OFF_ABBR.get(&team[..]);
    let correct_team: &str = if team_conversion.is_some() {
        team_conversion.unwrap().to_owned()
    } else {
        team
    };

    if pos == &Pos::D {
        let def_select: String = format!("SELECT id FROM player WHERE team = '{}'", correct_team);
        return conn
            .query_row(&def_select, (), |row| row.get(0))
            .optional()
            .unwrap();
    }
    // Try Exact Match
    let select_player: &str = "SELECT id FROM player WHERE name = ?1 and pos = ?2";
    let id: Option<i32> = conn
        .query_row(select_player, (name, pos.to_str().unwrap()), |row| {
            row.get(0)
        })
        .optional()
        .unwrap();
    if id.is_some() {
        return id;
    }

    // No hit on exact match
    let fuzzy_select: &str = "SELECT id FROM player WHERE name LIKE ?1 and pos = ?2";
    let mut name_split: Split<'_, &str> = name.trim().split(" ");
    let first_name: &str = name_split.next().unwrap();
    let last_name: &str = name_split.next().unwrap();
    let fuzzy_name: String = first_name.chars().nth(0).unwrap().to_string() + "%" + last_name + "%";

    let id: Option<i32> = conn
        .query_row(fuzzy_select, (&fuzzy_name, pos.to_str().unwrap()), |row| {
            row.get(0)
        })
        .optional()
        .unwrap();
    return id;
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
