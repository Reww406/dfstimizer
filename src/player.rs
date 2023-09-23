use lazy_static::lazy_static;
use std::str::Split;
use std::sync::RwLock;
use std::{collections::HashMap, hash::Hash};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::data_loader::*;

// TODO! Should populate all of these first so read writes are not blocked
lazy_static! {
    pub static ref REC_PROJ_CACHE: RwLock<HashMap<i16, RecProj>> = RwLock::new(HashMap::new());
    pub static ref RB_PROJ_CACHE: RwLock<HashMap<i16, RbProj>> = RwLock::new(HashMap::new());
    pub static ref QB_PROJ_CACHE: RwLock<HashMap<i16, QbProj>> = RwLock::new(HashMap::new());
    pub static ref DEF_PROJ_CACHE: RwLock<HashMap<i16, DefProj>> = RwLock::new(HashMap::new());
    pub static ref KICK_PROJ_CACHE: RwLock<HashMap<i16, KickProj>> = RwLock::new(HashMap::new());
    pub static ref DEF_VS_QB_CACHE: RwLock<HashMap<Team, DefVsPos>> = RwLock::new(HashMap::new());
    pub static ref DEF_VS_RB_CACHE: RwLock<HashMap<Team, DefVsPos>> = RwLock::new(HashMap::new());
    pub static ref DEF_VS_WR_CACHE: RwLock<HashMap<Team, DefVsPos>> = RwLock::new(HashMap::new());
    pub static ref DEF_VS_TE_CACHE: RwLock<HashMap<Team, DefVsPos>> = RwLock::new(HashMap::new());
    pub static ref DEF_ID_CACHE: RwLock<HashMap<Team, i16>> = RwLock::new(HashMap::new());
    pub static ref PLAYER_NAME_CACHE: RwLock<HashMap<i16, String>> = RwLock::new(HashMap::new());
    /// ID = name-pos-team
    pub static ref PLAYER_ID_CACHE: RwLock<HashMap<String, i16>> = RwLock::new(HashMap::new());
    pub static ref ID_LITEPLAYER_CACHE: RwLock<HashMap<i16, LitePlayer>> = RwLock::new(HashMap::new());
    pub static ref ID_LITEPLAYER_NO_SAL_CACHE: RwLock<HashMap<i16, LitePlayer>> = RwLock::new(HashMap::new());
    // pub static ref SLATE_CACHE: RwLock<Vec<LitePlayer>> = RwLock::new(Vec::new());

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Team {
    Lac,
    Phi,
    Chi,
    Mia,
    Jax,
    Cin,
    Det,
    Ind,
    Pit,
    Tb,
    Min,
    Ari,
    Lv,
    Den,
    Ten,
    Gb,
    Sea,
    Kc,
    Ne,
    Bal,
    Sf,
    La,
    Nyj,
    Buf,
    Car,
    Atl,
    Hou,
    Nyg,
    Dal,
    Cle,
    No,
    Was,
}

impl Team {
    pub fn from_str(team_abv: &String) -> Self {
        let no_at: &str = &team_abv.replace("@", "");
        match no_at {
            "LAC" => Self::Lac,
            "PHI" => Self::Phi,
            "CHI" => Self::Chi,
            "MIA" => Self::Mia,
            "JAX" => Self::Jax,
            "CIN" => Self::Cin,
            "DET" => Self::Det,
            "IND" => Self::Ind,
            "PIT" => Self::Pit,
            "TB" => Self::Tb,
            "MIN" => Self::Min,
            "ARI" => Self::Ari,
            "LV" => Self::Lv,
            "DEN" => Self::Den,
            "TEN" => Self::Ten,
            "GB" => Self::Gb,
            "SEA" => Self::Sea,
            "KC" => Self::Kc,
            "NE" => Self::Ne,
            "BAL" => Self::Bal,
            "SF" => Self::Sf,
            "LA" => Self::La,
            "NYJ" => Self::Nyj,
            "BUF" => Self::Buf,
            "CAR" => Self::Car,
            "ATL" => Self::Atl,
            "HOU" => Self::Hou,
            "NYG" => Self::Nyg,
            "DAL" => Self::Dal,
            "CLE" => Self::Cle,
            "NO" => Self::No,
            "WAS" => Self::Was,
            _ => panic!("Not a team"),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Team::Lac => "LAC",
            Team::Phi => "PHI",
            Team::Ind => "IND",
            Team::Pit => "PIT",
            Team::Tb => "TB",
            Team::Min => "MIN",
            Team::Ari => "ARI",
            Team::Lv => "LV",
            Team::Den => "DEN",
            Team::Ten => "TEN",
            Team::Gb => "GB",
            Team::Sea => "SEA",
            Team::Kc => "KC",
            Team::Ne => "NE",
            Team::Bal => "BAL",
            Team::Sf => "SF",
            Team::La => "LA",
            Team::Nyj => "NYJ",
            Team::Buf => "BUF",
            Team::Car => "CAR",
            Team::Atl => "ATL",
            Team::Hou => "HOU",
            Team::Nyg => "NYG",
            Team::Dal => "DAL",
            Team::Cle => "CLE",
            Team::No => "NO",
            Team::Was => "WAS",
            Team::Chi => "CHI",
            Team::Mia => "MIA",
            Team::Jax => "JAX",
            Team::Cin => "CIN",
            Team::Det => "DET",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: i16,
    pub name: String,
    pub team: String,
    pub pos: Pos,
}

// Change team and opp to ID
#[derive(Clone, Debug, Default, Copy)]
pub struct RbProj {
    // pub name: String,
    pub id: i16,
    pub team: Team,
    pub opp: Team,
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
    pub avg_rec_tgts: f32,
    pub salary: i32,
    pub own_proj: f32,
    pub rating: f32,
    pub snaps_per: f32,
    pub year_consistency: f32,
    pub vegas_team_total: f32,
    pub month_consistency: f32,
}

#[derive(Clone, Debug, Default, Copy)]
pub struct QbProj {
    // pub name: String,
    pub id: i16,
    pub team: Team,
    pub opp: Team,
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
    pub vegas_team_total: f32,
    pub month_consistency: f32,
    pub yds_per_pass_att: f32,
}

#[derive(Clone, Debug, Default, Copy)]
pub struct RecProj {
    // pub name: String,
    pub id: i16,
    pub team: Team,
    pub opp: Team,
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
    pub vegas_team_total: f32,
    pub month_consistency: f32,
    pub month_upside: f32,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct DefProj {
    // pub name: String,
    pub id: i16,
    pub team: Team,
    pub opp: Team,
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
    pub vegas_team_total: f32,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct KickProj {
    // pub name: String,
    pub id: i16,
    pub team: Team,
    pub opp: Team,
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

#[derive(Debug, Clone, Default, Copy)]
pub struct DefVsPos {
    pub team: Team,
    // pub team_name: String,
    pub pts_given_pg: f32,
    pub pos: Pos,
}

// Should be Enum will reduce code
#[derive(Debug, Clone, Default)]
pub struct FlexProj {
    pub pos: Pos,
    pub rec_proj: Option<RecProj>,
    pub rb_proj: Option<RbProj>,
}

#[derive(Debug, Clone)]
pub enum Proj {
    QbProj(QbProj),
    RecProj(RecProj),
    RbProj(RbProj),
    DefProj(DefProj),
    KickProj(KickProj),
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
    pub fn get_proj_own(&self) -> f32 {
        match self {
            Proj::QbProj(qb) => return qb.own_proj,
            Proj::DefProj(def) => return def.own_proj,
            Proj::RecProj(rec) => return rec.own_proj,
            Proj::RbProj(rb) => return rb.own_proj,
            Proj::KickProj(k) => return k.own_proj,
        }
    }
    pub fn get_proj_id(&self) -> i16 {
        match self {
            Proj::QbProj(qb) => return qb.id,
            Proj::DefProj(def) => return def.id,
            Proj::RecProj(rec) => return rec.id,
            Proj::RbProj(rb) => return rb.id,
            Proj::KickProj(k) => return k.id,
        }
    }

    pub fn get_proj_salary(&self) -> i32 {
        match self {
            Proj::QbProj(qb) => return qb.salary,
            Proj::DefProj(def) => return def.salary,
            Proj::RecProj(rec) => return rec.salary,
            Proj::RbProj(rb) => return rb.salary,
            Proj::KickProj(k) => return k.salary,
        }
    }

    pub fn get_name(&self, conn: &Connection) -> String {
        match self {
            Proj::QbProj(p) => get_player_name(p.id, conn),
            Proj::DefProj(p) => get_player_name(p.id, conn),
            Proj::RecProj(p) => get_player_name(p.id, conn),
            Proj::RbProj(p) => get_player_name(p.id, conn),
            Proj::KickProj(p) => get_player_name(p.id, conn),
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Copy)]
pub enum Pos {
    Qb = 0,
    Rb = 1,
    Wr = 2,
    Te = 3,
    D = 4,
    K = 5,
}

impl Default for Pos {
    fn default() -> Self {
        Self::D
    }
}

impl Default for Team {
    fn default() -> Self {
        panic!("Can't be default for team")
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

    pub fn get_proj_table(&self) -> &str {
        match self {
            Pos::Qb => "qb_proj",
            Pos::Rb => "rb_proj",
            Pos::Wr => "wr_proj",
            Pos::Te => "te_proj",
            Pos::D => "dst_proj",
            Pos::K => "kick_proj",
        }
    }

    pub fn get_def_table(&self) -> &str {
        match self {
            Pos::Qb => "def_vs_qb",
            Pos::Rb => "def_vs_rb",
            Pos::Wr => "def_vs_wr",
            Pos::Te => "def_vs_te",
            _ => panic!("No Def table for this pos"),
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

    pub fn from_string_ref(input: &String) -> Result<Pos, ()> {
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

pub fn get_player_name(id: i16, conn: &Connection) -> String {
    if PLAYER_NAME_CACHE.read().unwrap().get(&id).is_some() {
        return PLAYER_NAME_CACHE.read().unwrap().get(&id).unwrap().clone();
    }

    let query = "SELECT name FROM player WHERE id = ?1";
    let mut stmt = conn.prepare_cached(query).unwrap();
    let player: String = stmt.query_row(params![id], |row| row.get(0)).unwrap();

    PLAYER_NAME_CACHE
        .write()
        .unwrap()
        .insert(id, player.clone());
    player
}

pub fn get_player_by_id(week: i8, id: i16, season: i16, conn: &Connection) -> LitePlayer {
    if ID_LITEPLAYER_CACHE.read().unwrap().get(&id).is_some() {
        return *ID_LITEPLAYER_CACHE.read().unwrap().get(&id).unwrap();
    }

    let query = "SELECT * FROM ownership WHERE week = ?1 AND id = ?2 AND season = ?3";
    let mut stmt = conn.prepare_cached(query).unwrap();
    let player: LitePlayer = stmt
        .query_row(params![week, id, season], |row| {
            Ok(LitePlayer {
                id: row.get(0).unwrap(),
                salary: row.get(8).unwrap(),
                pos: Pos::from_string(row.get(7).unwrap()).unwrap(),
            })
        })
        .unwrap();

    ID_LITEPLAYER_CACHE.write().unwrap().insert(id, player);
    player
}

// Can we do just ID
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
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

    /// WARNING This liteplayer has no salary
    pub fn ids_to_liteplayer(ids: &[i16], conn: &Connection) -> Vec<Self> {
        let mut players: Vec<LitePlayer> = Vec::new();
        for id in ids {
            players.push(LitePlayer::id_to_liteplayer(id, conn));
        }
        players
    }

    pub fn id_to_liteplayer(id: &i16, conn: &Connection) -> Self {
        if ID_LITEPLAYER_NO_SAL_CACHE.read().unwrap().get(id).is_some() {
            return *ID_LITEPLAYER_NO_SAL_CACHE.read().unwrap().get(id).unwrap();
        }

        let query = "SELECT * FROM player WHERE id = ?1";
        let player: LitePlayer = conn
            .query_row(query, params![id], |row| {
                Ok(LitePlayer {
                    id: *id,
                    pos: Pos::from_string(row.get(3).unwrap()).expect("Pos is not valid."),
                    salary: 0,
                })
            })
            .unwrap();
        ID_LITEPLAYER_NO_SAL_CACHE
            .write()
            .unwrap()
            .insert(*id, player);
        player
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

fn add_def_to_cache(def_vs_pos: DefVsPos) {
    match def_vs_pos.pos {
        Pos::Qb => DEF_VS_QB_CACHE
            .write()
            .unwrap()
            .insert(def_vs_pos.team, def_vs_pos),
        Pos::Rb => DEF_VS_RB_CACHE
            .write()
            .unwrap()
            .insert(def_vs_pos.team, def_vs_pos),
        Pos::Wr => DEF_VS_WR_CACHE
            .write()
            .unwrap()
            .insert(def_vs_pos.team, def_vs_pos),
        Pos::Te => DEF_VS_TE_CACHE
            .write()
            .unwrap()
            .insert(def_vs_pos.team, def_vs_pos),
        _ => panic!("No Def Vs Pos"),
    };
}

// pub fn get_opp_player_id(opp: String, conn: &Connection) -> i16 {
//     let opp_no_at = opp.replace("@", "");
//     if DEF_ID_CACHE.read().unwrap().get(&opp_no_at).is_some() {
//         let id: i16 = *DEF_ID_CACHE.read().unwrap().get(&opp_no_at).unwrap();
//         return id;
//     }
//     let select_player: &str = "SELECT id FROM player WHERE team = ?1 AND pos = D";
//     let id: i16 = conn
//         .query_row(select_player, params![opp_no_at], |row| row.get(0))
//         .unwrap();
//     DEF_ID_CACHE.write().unwrap().insert(opp_no_at, id);

//     id
// }

pub fn query_def_id(opp: &Team, conn: &Connection) -> Result<i16, rusqlite::Error> {
    if DEF_ID_CACHE.read().unwrap().get(opp).is_some() {
        return Ok(*DEF_ID_CACHE.read().unwrap().get(opp).unwrap());
    }

    let select_player: &str = "SELECT id FROM player WHERE pos = 'D' AND team = ?1";
    let id: i16 = conn.query_row(select_player, params![opp.to_str()], |row| row.get(0))?;

    println!("Cache miss");
    DEF_ID_CACHE.write().unwrap().insert(*opp, id);
    Ok(id)
}

// This should just be team to avoid ID lookup
// and then if it's missing we get id by team!
pub fn query_def_vs_pos(opp: Team, player_pos: &Pos, conn: &Connection) -> DefVsPos {
    let cache_hit: Option<DefVsPos> = match player_pos {
        Pos::Qb => {
            if DEF_VS_QB_CACHE.read().unwrap().get(&opp).is_some() {
                Some(*DEF_VS_QB_CACHE.read().unwrap().get(&opp).unwrap())
            } else {
                None
            }
        }
        Pos::Rb => {
            if DEF_VS_RB_CACHE.read().unwrap().get(&opp).is_some() {
                Some(*DEF_VS_RB_CACHE.read().unwrap().get(&opp).unwrap())
            } else {
                None
            }
        }
        Pos::Wr => {
            if DEF_VS_WR_CACHE.read().unwrap().get(&opp).is_some() {
                Some(*DEF_VS_WR_CACHE.read().unwrap().get(&opp).unwrap())
            } else {
                None
            }
        }
        Pos::Te => {
            if DEF_VS_TE_CACHE.read().unwrap().get(&opp).is_some() {
                Some(*DEF_VS_TE_CACHE.read().unwrap().get(&opp).unwrap())
            } else {
                None
            }
        }
        _ => panic!("No Def Vs For Pos for that Pos"),
    };

    if cache_hit.is_some() {
        return cache_hit.unwrap().to_owned();
    }
    let id = query_def_id(&opp, conn).unwrap();
    let mut stmt = conn
        .prepare_cached(format!("SELECT * FROM {} WHERE id = ?1", player_pos.get_def_table()).as_str())
        .unwrap();
    let def_vs_pos: DefVsPos = stmt
        .query_row(params![id], |row| {
            Ok(DefVsPos {
                team: opp,
                // team_name: row.get(1).unwrap(),
                pts_given_pg: row.get(2).unwrap(),
                pos: *player_pos,
            })
        })
        .unwrap();
    add_def_to_cache(def_vs_pos);
    println!("Cache miss");
    def_vs_pos
}

pub fn query_proj(player: Option<&LitePlayer>, week: i8, season: i16, conn: &Connection) -> Proj {
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

pub fn query_rec_proj_helper(
    player: Option<&LitePlayer>,
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
    player: Option<&LitePlayer>,
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
    player: Option<&LitePlayer>,
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
    player: Option<&LitePlayer>,
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
    player: Option<&LitePlayer>,
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

//TODO Refactor all of these into options
pub fn query_kick_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<KickProj> {
    if KICK_PROJ_CACHE.read().unwrap().get(&id).is_some() {
        let proj: KickProj = *KICK_PROJ_CACHE.read().unwrap().get(&id).unwrap();
        return Some(proj);
    }
    let mut query = conn
        .prepare_cached("SELECT * FROM kick_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't prepare query");
    let kick_proj: Option<KickProj> = query
        .query_row((id, week, season), |row| {
            Ok(KickProj {
                // name: row.get(3)?,
                id: row.get(0)?,
                team: Team::from_str(&row.get(4)?),
                opp: Team::from_str(&row.get(5)?),
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
        .write()
        .unwrap()
        .insert(id, kick_proj.unwrap());
    println!("Cache miss");
    kick_proj
}

pub fn query_rec_proj(
    id: i16,
    week: i8,
    season: i16,
    pos: &Pos,
    conn: &Connection,
) -> Option<RecProj> {
    if REC_PROJ_CACHE.read().unwrap().get(&id).is_some() {
        let proj: RecProj = *REC_PROJ_CACHE.read().unwrap().get(&id).unwrap();
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
                // name: row.get(3)?,
                team: Team::from_str(&row.get(4)?),
                opp: Team::from_str(&row.get(5)?),
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
                vegas_team_total: row.get(24)?,
                month_consistency: row.get(25)?,
                month_upside: row.get(26)?,
                // Day
            })
        })
        .optional()
        .expect("Could not get WR");
    if rec_proj.is_none() {
        return None;
    }
    REC_PROJ_CACHE
        .write()
        .unwrap()
        .insert(id, rec_proj.unwrap());
    println!("Cache miss");
    rec_proj
}

pub fn query_rb_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<RbProj> {
    if RB_PROJ_CACHE.read().unwrap().get(&id).is_some() {
        let proj: RbProj = *RB_PROJ_CACHE.read().unwrap().get(&id).unwrap();
        return Some(proj);
    }
    let mut query = conn
        .prepare_cached("SELECT * FROM rb_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't Prepare statement");
    let rb_proj: Option<RbProj> = query
        .query_row((id, week, season), |row| {
            Ok(RbProj {
                id: row.get(0)?,
                // name: row.get(3)?,
                team: Team::from_str(&row.get(4)?),
                opp: Team::from_str(&row.get(5)?),
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
                avg_rec_tgts: row.get(16)?,
                salary: row.get(17)?,
                own_proj: row.get(18)?,
                rating: row.get(19)?,
                snaps_per: row.get(20)?,
                year_consistency: row.get(21)?,
                vegas_team_total: row.get(22)?,
                month_consistency: row.get(23)?,
                // Day
            })
        })
        .optional()
        .unwrap();
    if rb_proj.is_none() {
        return None;
    }
    RB_PROJ_CACHE.write().unwrap().insert(id, rb_proj.unwrap());
    println!("Cache miss");
    rb_proj
}

pub fn query_qb_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<QbProj> {
    if QB_PROJ_CACHE.read().unwrap().get(&id).is_some() {
        let proj: QbProj = *QB_PROJ_CACHE.read().unwrap().get(&id).unwrap();
        return Some(proj);
    }
    let mut query = conn
        .prepare_cached("SELECT * FROM qb_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't prepare query");
    let qb_proj: Option<QbProj> = query
        .query_row((id, week, season), |row| {
            Ok(QbProj {
                id: row.get(0)?,
                // name: row.get(3)?,
                team: Team::from_str(&row.get(4)?),
                opp: Team::from_str(&row.get(5)?),
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
                vegas_team_total: row.get(25)?,
                month_consistency: row.get(26)?,
                yds_per_pass_att: row.get(27)?,
                // Day
            })
        })
        .optional()
        .unwrap();
    if qb_proj.is_none() {
        return None;
    }
    QB_PROJ_CACHE.write().unwrap().insert(id, qb_proj.unwrap());
    println!("Cache miss");
    qb_proj
}

pub fn query_def_proj(id: i16, week: i8, season: i16, conn: &Connection) -> Option<DefProj> {
    if DEF_PROJ_CACHE.read().unwrap().get(&id).is_some() {
        let proj: DefProj = *DEF_PROJ_CACHE.read().unwrap().get(&id).unwrap();
        return Some(proj);
    }
    let mut query = conn
        .prepare_cached("SELECT * FROM dst_proj WHERE id = ?1 AND week = ?2 AND season = ?3")
        .expect("Couldn't prepare query");
    let def_proj: Option<DefProj> = query
        .query_row((id, week, season), |row| {
            Ok(DefProj {
                id: row.get(0)?,
                // name: row.get(3)?,
                team: Team::from_str(&row.get(4)?),
                opp: Team::from_str(&row.get(5)?),
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
                // Day
                vegas_team_total: row.get(17)?,
            })
        })
        .optional()
        .unwrap();
    if def_proj.is_none() {
        return None;
    }
    DEF_PROJ_CACHE
        .write()
        .unwrap()
        .insert(id, def_proj.unwrap());
    println!("Cache miss");
    def_proj
}

pub fn get_player_id_create_if_missing(
    name: &String,
    team: &String,
    pos: &Pos,
    conn: &Connection,
) -> i16 {
    let id: Option<i16> = get_player_id(name, team, pos, conn);
    if id.is_some() {
        return id.unwrap();
    }
    let player: Player = Player {
        id: 0,
        name: name.clone(),
        team: team.clone(),
        pos: pos.clone(),
    };
    return load_player_id(&player, conn);
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
pub fn get_player_id(name: &String, team: &String, pos: &Pos, conn: &Connection) -> Option<i16> {
    // Try Exact Match
    let key = format!("{}-{}-{}", name, team, pos.to_str().unwrap());
    if PLAYER_ID_CACHE.read().unwrap().get(&key).is_some() {
        return Some(*PLAYER_ID_CACHE.read().unwrap().get(&key).unwrap());
    }

    let select_player: &str = "SELECT id FROM player WHERE name = ?1 AND pos = ?2 AND team = ?3";
    let id: Option<i16> = conn
        .query_row(select_player, (name, pos.to_str().unwrap(), team), |row| {
            row.get(0)
        })
        .optional()
        .unwrap();
    if id.is_some() {
        PLAYER_ID_CACHE.write().unwrap().insert(key, id.unwrap());
        return id;
    }

    // No hit on exact match
    let fuzzy_select: &str = "SELECT id FROM player WHERE name LIKE ?1 and pos = ?2 AND team = ?3";
    let mut name_split: Split<'_, &str> = name.trim().split(" ");
    let first_name: &str = name_split.next().unwrap();
    let last_name: &str = name_split.next().unwrap();
    let fuzzy_name: String = first_name.chars().nth(0).unwrap().to_string() + "%" + last_name + "%";

    let id: Option<i16> = conn
        .query_row(
            fuzzy_select,
            (&fuzzy_name, pos.to_str().unwrap(), team),
            |row| row.get(0),
        )
        .optional()
        .unwrap();
    if id.is_some() {
        PLAYER_ID_CACHE.write().unwrap().insert(key, id.unwrap());
    }
    id
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
