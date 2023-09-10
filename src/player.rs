use lazy_static::lazy_static;
use std::str::Split;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, error::Error, hash::Hash};

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::data_loader::NON_OFF_TO_OFF_ABBR;
use crate::lineup::LineupBuilder;
use crate::{data_loader::*, DATABASE_FILE};

lazy_static! {
    pub static ref REC_PROJ_CACHE: Mutex<HashMap<i16, RecProj>> = Mutex::new(HashMap::new());
    pub static ref RB_PROJ_CACHE: Mutex<HashMap<i16, RbProj>> = Mutex::new(HashMap::new());
    pub static ref QB_PROJ_CACHE: Mutex<HashMap<i16, QbProj>> = Mutex::new(HashMap::new());
    pub static ref DEF_PROJ_CACHE: Mutex<HashMap<i16, DefProj>> = Mutex::new(HashMap::new());
    pub static ref KICK_PROJ_CACHE: Mutex<HashMap<i16, KickProj>> = Mutex::new(HashMap::new());
}

#[derive(Debug)]
pub enum Proj {
    QbProj(QbProj),
    RecProj(RecProj),
    RbProj(RbProj),
    DefProj(DefProj),
    KickProj(KickProj),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Copy)]
pub enum Pos {
    Qb = 0,
    Rb = 1,
    Wr = 2,
    Te = 3,
    D = 4,
    K = 5,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Default)]
pub struct RbProj {
    pub name: String,
    pub id: i16,
    pub team: String,
    pub opp: String,
    pub pts_proj: f32,
    pub cieling_proj: f32,
    pub floor_proj: f32,
    pub pts_plus_minus_proj: f32,
    pub pts_sal_proj: f32,
    pub vegas_total: f32,
    pub rush_yds_share: f32,
    pub avg_att: f32,
    pub avg_td: f32,
    pub avg_rush_yds: f32,
    pub avg_rec_yds: f32,
    pub salary: i32,
    pub own_proj: f32,
    pub rating: f32,
    pub snaps_per: f32,
    pub year_consistency: f32,
}

#[derive(Clone, Debug, Default)]
pub struct QbProj {
    pub name: String,
    pub id: i16,
    pub team: String,
    pub opp: String,
    pub pts_proj: f32,
    pub cieling_proj: f32,
    pub floor_proj: f32,
    pub pts_plus_minus_proj: f32,
    pub pts_sal_proj: f32,
    pub vegas_total: f32,
    pub avg_pass_atts: f32,
    pub avg_pass_comps: f32,
    pub avg_pass_yds: f32,
    pub avg_pass_tds: f32,
    pub avg_rush_atts: f32,
    pub avg_long_pass_yds: f32,
    pub pass_to_wr_per: f32,
    pub pass_to_te_per: f32,
    pub wind_speed: f32,
    pub salary: i32,
    pub own_proj: f32,
    pub rating: f32,
    pub red_zone_op_pg: f32,
}
#[derive(Clone, Debug, Default)]
pub struct RecProj {
    pub name: String,
    pub id: i16,
    pub team: String,
    pub opp: String,
    pub pos: Pos,
    pub pts_proj: f32,
    pub cieling_proj: f32,
    pub floor_proj: f32,
    pub pts_plus_minus_proj: f32,
    pub pts_sal_proj: f32,
    pub vegas_total: f32,
    pub avg_rec: f32,
    pub avg_tgts: f32,
    pub avg_td: f32,
    pub avg_rec_yds: f32,
    pub avg_rush_yds: f32,
    pub red_zone_op_pg: f32,
    pub rec_tgt_share: f32,
    pub salary: i32,
    pub own_proj: f32,
    pub rating: f32,
    pub year_consistency: f32,
    pub year_upside: f32,
}

#[derive(Debug, Clone, Default)]
pub struct DefProj {
    pub name: String,
    pub id: i16,
    pub team: String,
    pub opp: String,
    pub pts_proj: f32,
    pub cieling_proj: f32,
    pub floor_proj: f32,
    pub pts_plus_minus_proj: f32,
    pub pts_sal_proj: f32,
    pub vegas_total: f32,
    pub salary: i32,
    pub own_proj: f32,
    pub rating: f32,
    pub vegas_opp_total: f32,
}

#[derive(Debug, Clone, Default)]
pub struct KickProj {
    pub name: String,
    pub id: i16,
    pub team: String,
    pub opp: String,
    pub pts_proj: f32,
    pub cieling_proj: f32,
    pub floor_proj: f32,
    pub pts_plus_minus_proj: f32,
    pub pts_sal_proj: f32,
    pub vegas_total: f32,
    pub salary: i32,
    pub own_proj: f32,
    pub rating: f32,
}

// Should be Enum will reduce code
#[derive(Debug, Clone, Default)]
pub struct FlexProj {
    pub pos: Pos,
    pub rec_proj: Option<RecProj>,
    pub rb_proj: Option<RbProj>,
}

impl Proj {
    pub fn get_proj_pos(&self) -> Pos {
        match self {
            Proj::QbProj(_) => return Pos::Qb,
            Proj::DefProj(_) => return Pos::D,
            Proj::RecProj(rec_proj) => return rec_proj.pos,
            Proj::RbProj(_) => return Pos::Rb,
            Proj::KickProj(_) => return Pos::K,
        }
    }
    pub fn print_name(&self) {
        match self {
            Proj::QbProj(p) => print!("{}, {} ", p.name, p.team),
            Proj::DefProj(p) => print!("{}, {} ", p.name, p.team),
            Proj::RecProj(p) => print!("{}, {} ", p.name, p.team),
            Proj::RbProj(p) => print!("{}, {} ", p.name, p.team),
            Proj::KickProj(p) => print!("{}, {} ", p.name, p.team),
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            Proj::QbProj(p) => &p.name,
            Proj::DefProj(p) => &p.name,
            Proj::RecProj(p) => &p.name,
            Proj::RbProj(p) => &p.name,
            Proj::KickProj(p) => &p.name,
        }
    }

    pub fn get_qb_proj(&self) -> &QbProj {
        match self {
            Proj::QbProj(qb_proj) => return qb_proj,
            _ => panic!("Not a QB Proj"),
        }
    }

    pub fn get_rec_proj(&self) -> &RecProj {
        match self {
            Proj::RecProj(rec_proj) => return rec_proj,
            _ => panic!("Not a WR Proj"),
        }
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self::D
    }
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
            "K" => Ok(Pos::K),
            _ => Err(()),
        }
    }

    pub fn from_string(input: String) -> Result<Pos, ()> {
        let input = input.to_uppercase();

        match input.as_str() {
            "QB" => Ok(Pos::Qb),
            "RB" => Ok(Pos::Rb),
            "WR" => Ok(Pos::Wr),
            "TE" => Ok(Pos::Te),
            "D" => Ok(Pos::D),
            "DST" => Ok(Pos::D),
            "K" => Ok(Pos::K),
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
            Pos::K => Ok("K"),
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
    pub fn new(record: csv::StringRecord, conn: &Connection) -> Self {
        let pos: Pos = Pos::from_str(&record[4].to_string()).expect("Couldn't convert error");
        LitePlayer {
            id: get_player_id(&record[1].to_string(), &record[2].to_string(), &pos, conn).unwrap()
                as i16,
            pos: pos,
            salary: record[5].parse::<i16>().expect("Salary Missing"),
        }
    }

    pub fn test() -> Self {
        LitePlayer {
            id: 1,
            pos: Pos::Rb,
            salary: 15000,
        }
    }

    // Could make this a singleton so it's only generated once
    pub fn player_lookup_map(players: &[Arc<LitePlayer>]) -> HashMap<i16, &Arc<LitePlayer>> {
        let mut lookup_map = HashMap::new();
        players.iter().for_each(|p| {
            lookup_map.insert(p.id, p);
        });
        lookup_map
    }
}

pub fn query_proj(
    player: &Option<Arc<LitePlayer>>,
    week: i8,
    season: i16,
    conn: &Connection,
) -> Proj {
    match player.as_ref().unwrap().pos {
        Pos::Qb => return Proj::QbProj(query_qb_proj_helper(player, week, season, conn)),
        Pos::Rb => return Proj::RbProj(query_rb_proj_helper(player, week, season, conn)),
        Pos::Wr => {
            return Proj::RecProj(query_rec_proj_helper(player, week, season, &Pos::Wr, conn))
        }
        Pos::Te => {
            return Proj::RecProj(query_rec_proj_helper(player, week, season, &Pos::Te, conn))
        }
        Pos::D => return Proj::DefProj(query_def_proj_helper(player, week, season, conn)),
        Pos::K => return Proj::KickProj(query_kick_proj_helper(player, week, season, conn)),
    }
}

pub fn get_recent_stat_ceiling(
    season: i16,
    week: i8,
    field: &str,
    table: &str,
    player_id: i16,
    conn: &Connection,
) -> f32 {
    let week_range_start: i8 = week - (2);
    let mut ceiling: f32 = 0.0;
    let query: String = format!(
        "SELECT ({}) FROM {} WHERE season = ?1 AND week BETWEEN ?2 AND ?3 AND id = ?4",
        field, table
    );
    let mut stmt: rusqlite::CachedStatement<'_> = conn.prepare_cached(query.as_str()).unwrap();
    stmt.query_map((season, week_range_start, week, player_id), |row| {
        row.get(0)
    })
    .unwrap()
    .into_iter()
    .for_each(|row| {
        let value: f32 = row.unwrap();
        if value > ceiling {
            ceiling = value
        }
    });
    ceiling
}

pub fn query_rec_proj_helper(
    player: &Option<Arc<LitePlayer>>,
    week: i8,
    season: i16,
    pos: &Pos,
    conn: &Connection,
) -> RecProj {
    query_rec_proj(
        player
            .as_ref()
            .expect("WR/TE was not set when trying to filter")
            .id,
        week,
        season,
        pos,
        conn,
    )
    .expect("Could not find WR/TE when trying to filter")
}

pub fn query_def_proj_helper(
    player: &Option<Arc<LitePlayer>>,
    week: i8,
    season: i16,
    conn: &Connection,
) -> DefProj {
    query_def_proj(
        player
            .as_ref()
            .expect("DST was not set when trying to get Proj")
            .id,
        week,
        season,
        conn,
    )
    .expect("Could not find DST Proj")
}

pub fn query_kick_proj_helper(
    player: &Option<Arc<LitePlayer>>,
    week: i8,
    season: i16,
    conn: &Connection,
) -> KickProj {
    query_kick_proj(
        player
            .as_ref()
            .expect("DST was not set when trying to get Proj")
            .id,
        week,
        season,
        conn,
    )
    .expect("Could not find DST Proj")
}

pub fn query_rb_proj_helper(
    player: &Option<Arc<LitePlayer>>,
    week: i8,
    season: i16,
    conn: &Connection,
) -> RbProj {
    query_rb_proj(
        player
            .as_ref()
            .expect("RB was not set when trying to get Proj")
            .id,
        week,
        season,
        conn,
    )
    .expect("Could not find RB when trying to get Proj")
}

pub fn query_qb_proj_helper(
    player: &Option<Arc<LitePlayer>>,
    week: i8,
    season: i16,
    conn: &Connection,
) -> QbProj {
    query_qb_proj(
        player
            .as_ref()
            .expect("QB was not set when trying to get Proj")
            .id,
        week,
        season,
        conn,
    )
    .expect("Could not find QB when trying to get Proj")
}
pub fn query_kick_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<KickProj> {
    if KICK_PROJ_CACHE.lock().unwrap().get(&id).is_some() {
        let proj: KickProj = KICK_PROJ_CACHE.lock().unwrap().get(&id).unwrap().clone();
        return Some(proj);
    }
    let mut query = conn
        .prepare_cached("SELECT * FROM kick_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't prepare query");
    let kick_proj: Option<KickProj> = query
        .query_row((id, week, season), |row| {
            Ok(KickProj {
                name: row.get(3)?,
                id: row.get(0)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts_proj: row.get(6)?,
                cieling_proj: row.get(7)?,
                floor_proj: row.get(8)?,
                pts_plus_minus_proj: row.get(9)?,
                pts_sal_proj: row.get(10)?,
                vegas_total: row.get(11)?,
                salary: row.get(12)?,
                own_proj: row.get(13)?,
                rating: row.get(14)?,
            })
        })
        .optional()
        .unwrap();
    if kick_proj.is_none() {
        return None;
    }
    KICK_PROJ_CACHE
        .lock()
        .unwrap()
        .insert(id, kick_proj.clone().unwrap());
    kick_proj
}

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
    let mut query = conn
        .prepare_cached(
            format!(
                "SELECT * FROM {} WHERE id = ?1 AND week = ?2 AND season = ?3",
                table
            )
            .as_str(),
        )
        .expect("Couldn't Prepare statement");
    let rec_proj: Option<RecProj> = query
        .query_row((id, week, season), |row| {
            Ok(RecProj {
                id: row.get(0)?,
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pos: *pos,
                pts_proj: row.get(6)?,
                cieling_proj: row.get(7)?,
                floor_proj: row.get(8)?,
                pts_plus_minus_proj: row.get(9)?,
                pts_sal_proj: row.get(10)?,
                vegas_total: row.get(11)?,
                avg_rec: row.get(12)?,
                avg_tgts: row.get(13)?,
                avg_td: row.get(14)?,
                avg_rec_yds: row.get(15)?,
                avg_rush_yds: row.get(16)?,
                red_zone_op_pg: row.get(17)?,
                rec_tgt_share: row.get(18)?,
                salary: row.get(19)?,
                own_proj: row.get(20)?,
                rating: row.get(21)?,
                year_consistency: row.get(22)?,
                year_upside: row.get(23)?,
            })
        })
        .optional()
        .expect("Could not get WR");
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
    let mut query = conn
        .prepare_cached("SELECT * FROM rb_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't Prepare statement");
    let rb_proj: Option<RbProj> = query
        .query_row((id, week, season), |row| {
            Ok(RbProj {
                id: row.get(0)?,
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts_proj: row.get(6)?,
                cieling_proj: row.get(7)?,
                floor_proj: row.get(8)?,
                pts_plus_minus_proj: row.get(9)?,
                pts_sal_proj: row.get(10)?,
                vegas_total: row.get(11)?,
                rush_yds_share: row.get(12)?,
                avg_att: row.get(13)?,
                avg_td: row.get(14)?,
                avg_rush_yds: row.get(15)?,
                avg_rec_yds: row.get(16)?,
                salary: row.get(17)?,
                own_proj: row.get(18)?,
                rating: row.get(19)?,
                snaps_per: row.get(20)?,
                year_consistency: row.get(21)?,
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
    let mut query = conn
        .prepare_cached("SELECT * FROM qb_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't prepare query");
    let qb_proj: Option<QbProj> = query
        .query_row((id, week, season), |row| {
            Ok(QbProj {
                id: row.get(0)?,
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts_proj: row.get(6)?,
                cieling_proj: row.get(7)?,
                floor_proj: row.get(8)?,
                pts_plus_minus_proj: row.get(9)?,
                pts_sal_proj: row.get(10)?,
                vegas_total: row.get(11)?,
                avg_pass_atts: row.get(12)?,
                avg_pass_comps: row.get(13)?,
                avg_pass_yds: row.get(14)?,
                avg_pass_tds: row.get(15)?,
                avg_rush_atts: row.get(16)?,
                avg_long_pass_yds: row.get(17)?,
                pass_to_wr_per: row.get(18)?,
                pass_to_te_per: row.get(19)?,
                wind_speed: row.get(20)?,
                salary: row.get(21)?,
                own_proj: row.get(22)?,
                rating: row.get(23)?,
                red_zone_op_pg: row.get(24)?,
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
    let mut query = conn
        .prepare_cached("SELECT * FROM dst_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't prepare query");
    let def_proj: Option<DefProj> = query
        .query_row((id, week, season), |row| {
            Ok(DefProj {
                id: row.get(0)?,
                name: row.get(3)?,
                team: row.get(4)?,
                opp: row.get(5)?,
                pts_proj: row.get(6)?,
                cieling_proj: row.get(7)?,
                floor_proj: row.get(8)?,
                pts_plus_minus_proj: row.get(9)?,
                pts_sal_proj: row.get(10)?,
                vegas_total: row.get(11)?,
                salary: row.get(12)?,
                own_proj: row.get(13)?,
                rating: row.get(14)?,
                vegas_opp_total: row.get(15)?,
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

pub fn proj_exists(id: i16, week: i8, season: i16, pos: Pos, conn: &Connection) -> bool {
    match pos {
        Pos::D => return query_def_proj(id, week, season, conn).is_some(),
        Pos::Qb => return query_qb_proj(id, week, season, conn).is_some(),
        Pos::Rb => return query_rb_proj(id, week, season, conn).is_some(),
        Pos::Te => return query_rec_proj(id, week, season, &pos, conn).is_some(),
        Pos::Wr => return query_rec_proj(id, week, season, &pos, conn).is_some(),
        Pos::K => return query_kick_proj(id, week, season, conn).is_some(),
    }
}

// Get Player ID, Searches D, Then Exact, Then Fuzzy
pub fn get_player_id(name: &String, team: &String, pos: &Pos, conn: &Connection) -> Option<i32> {
    // Try Exact Match
    let select_player: &str = "SELECT id FROM player WHERE name = ?1 AND pos = ?2 AND team = ?3";
    let id: Option<i32> = conn
        .query_row(select_player, (name, pos.to_str().unwrap(), team), |row| {
            row.get(0)
        })
        .optional()
        .unwrap();
    if id.is_some() {
        return id;
    }

    // No hit on exact match
    let fuzzy_select: &str = "SELECT id FROM player WHERE name LIKE ?1 and pos = ?2 AND team = ?3";
    let mut name_split: Split<'_, &str> = name.trim().split(" ");
    let first_name: &str = name_split.next().unwrap();
    let last_name: &str = name_split.next().unwrap();
    let fuzzy_name: String = first_name.chars().nth(0).unwrap().to_string() + "%" + last_name + "%";

    let id: Option<i32> = conn
        .query_row(
            fuzzy_select,
            (&fuzzy_name, pos.to_str().unwrap(), team),
            |row| row.get(0),
        )
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
