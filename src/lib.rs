use std::{cmp::min, fs, rc::Rc};

use itertools::Itertools;
use lazy_static::lazy_static;
use lineup::{dst_score, qb_score, rb_score, score_kicker, te_score, wr_stud_score, LineupBuilder};
use num_bigint::{BigUint, ToBigUint};
use rusqlite::{params, Connection};

use crate::player::*;

pub mod data_loader;
pub mod island_optimizer;
pub mod lineup;
pub mod optimizer;
pub mod player;
pub mod tables;

pub const DATABASE_FILE: &str = "./dfs_nfl.db3";
pub const SEASON: i16 = 2023;
pub const WEEK: i8 = 2;

pub const OWNERSHIP_CUTOFF_PER: f32 = 0.20;

pub const FILTER_TOP_QB: i8 = 3;
pub const FILTER_TOP_RB: i8 = 2;

// pub const WR_COUNT: i8 = 20;
// pub const QB_COUNT: i8 = 12;
// pub const TE_COUNT: i8 = 12;
// pub const RB_COUNT: i8 = 20;
// pub const D_COUNT: i8 = 10;
pub const WR_COUNT: i8 = 10;
pub const QB_COUNT: i8 = 10;
pub const TE_COUNT: i8 = 10;
pub const RB_COUNT: i8 = 10;
pub const D_COUNT: i8 = 10;

lazy_static! {
    pub static ref RB_WR_FLEX_PTS_PLUS: (f32, f32) = get_max_min_flex(SEASON, WEEK, "pts_plus_minus_proj", &[Pos::Rb, Pos::Wr]);
    pub static ref RB_WR_FLEX_CIELING: (f32, f32) = get_max_min_flex(SEASON, WEEK, "cieling_proj", &[Pos::Rb, Pos::Wr]);
    // Own cum
    pub static ref OWN_CUM_CUTOFF: f32 = get_sunday_ownership_cut_off(WEEK, SEASON);

    // QB Stats
    pub static ref QB_RUSH_ATT_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "avg_rush_atts", "qb_proj");
    pub static ref QB_RUSH_ATT: (f32, f32) =
        get_max_min(SEASON, WEEK, "avg_rush_atts", Pos::Qb);
    pub static ref QB_AVG_RZ_OP_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "red_zone_op_pg", "qb_proj");
    pub static ref QB_AVG_RZ_OP: (f32, f32) =
        get_max_min(SEASON, WEEK, "red_zone_op_pg", Pos::Qb);
    pub static ref QB_WR_PASS_PER: (f32, f32) =
        get_max_min(SEASON, WEEK, "pass_to_wr_per", Pos::Qb);
    pub static ref QB_TE_PASS_PER_MEDIAN: f32 =
        get_field_median(SEASON, WEEK, "pass_to_te_per", "qb_proj", QB_COUNT);
    pub static ref QB_PTS_PER_SAL: (f32, f32) = get_max_min(SEASON, WEEK , "pts_sal_proj", Pos::Qb);
    pub static ref QB_CIELING: (f32, f32) = get_max_min(SEASON, WEEK, "cieling_proj", Pos::Qb);
    pub static ref QB_OPP_DEF: (f32, f32) = get_def_max_min(&Pos::Qb);

    // RB Stats
    pub static ref RB_ATTS_FILLER: f32 = get_field_filler(SEASON, WEEK, "avg_atts", "rb_proj");
    pub static ref RB_ATTS: (f32, f32) = get_max_min(SEASON, WEEK, "avg_atts", Pos::Rb);
    pub static ref RB_AVG_REC_YDS: (f32, f32) = get_max_min(SEASON, WEEK, "avg_rec_yds", Pos::Rb);
    pub static ref RB_AVG_REC_YDS_FILLER: f32 = get_field_filler(SEASON, WEEK, "avg_rec_yds", "rb_proj");
    pub static ref RB_PTS_PLUS_MINUS: (f32, f32) =
        get_max_min(SEASON, WEEK, "pts_plus_minus_proj", Pos::Rb);
    pub static ref RB_YEAR_CONSISTENCY_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "year_consistency", "rb_proj");
    pub static ref RB_YEAR_CONSISTENCY: (f32, f32) =
        get_max_min(SEASON, WEEK, "year_consistency", Pos::Rb);
    pub static ref RB_OPP_DEF: (f32, f32) = get_def_max_min(&Pos::Rb);
    // WR Stats
    pub static ref WR_TGT_SHARE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "rec_tgt_share", "wr_proj");
    pub static ref WR_TGT_SHARE: (f32, f32) =
        get_max_min(SEASON, WEEK, "rec_tgt_share", Pos::Wr);
    pub static ref WR_RED_ZONE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "red_zone_op_pg", "wr_proj");
    pub static ref WR_RED_ZONE: (f32, f32) =
        get_max_min(SEASON, WEEK, "red_zone_op_pg", Pos::Wr);
    pub static ref WR_YEAR_CONSISTENCY_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "year_consistency", "wr_proj");
    pub static ref WR_YEAR_CONSISTENCY: (f32, f32) =
        get_max_min(SEASON, WEEK, "year_consistency", Pos::Wr);
    pub static ref WR_YEAR_UPSIDE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "year_upside", "wr_proj");
    pub static ref WR_YEAR_UPSIDE: (f32, f32) =
        get_max_min(SEASON, WEEK, "year_upside", Pos::Wr);
    pub static ref WR_CIELING: (f32, f32) =
        get_max_min(SEASON, WEEK, "cieling_proj", Pos::Wr);
    pub static ref WR_SALARY_MEDIAN: f32 =
        get_field_median(SEASON, WEEK, "salary", "wr_proj", WR_COUNT);
    pub static ref WR_OPP_DEF: (f32, f32) = get_def_max_min(&Pos::Wr);
    // TE Stats
    pub static ref TE_REC_TGT_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "rec_tgt_share", "te_proj");
    pub static ref TE_REC_TGT: (f32, f32) =
        get_max_min(SEASON, WEEK, "rec_tgt_share", Pos::Te);
    pub static ref TE_RED_ZONE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "red_zone_op_pg", "te_proj");
    pub static ref TE_RED_ZONE: (f32, f32) =
        get_max_min(SEASON, WEEK, "red_zone_op_pg", Pos::Te);
    pub static ref TE_OPP_DEF: (f32, f32) = get_def_max_min(&Pos::Te);
    pub static ref TE_PTS_SAL: (f32, f32) = get_max_min(SEASON, WEEK, "pts_sal_proj", Pos::Te);
    // TODO need to be used for Island games
    pub static ref ALL_PTS_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "pts_proj");
    pub static ref ALL_FLOOR_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "floor_proj");
    pub static ref ALL_CIELING_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "cieling_proj");
    pub static ref ALL_PTS_SAL_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "pts_sal_proj");
    pub static ref ALL_PTS_PLUS_MINS_MAX_MIN: (f32, f32) =
        get_max_min_all(SEASON, WEEK, "pts_plus_minus_proj");

    // DST Stats
    pub static ref DST_RATING: (f32, f32) = get_max_min(SEASON, WEEK, "rating", Pos::D);
    pub static ref DST_VEGAS_OPP_TOTAL: (f32, f32) =
        get_inverse_max_min(SEASON, WEEK, "vegas_opp_total", &Pos::D);
    pub static ref DST_PTS_PLUS_MINUS: (f32, f32) = get_max_min(SEASON, WEEK, "pts_plus_minus_proj", Pos::D);

    pub static ref ALL_VEGAS_TOTAL: (f32, f32) = get_max_min_all(SEASON, WEEK, "vegas_total");
}

pub enum Day {
    Mon,
    Thu,
    Sun,
}
impl Day {
    pub fn to_str(&self) -> &str {
        match self {
            Day::Mon => "mon",
            Day::Thu => "thu",
            Day::Sun => "sun",
        }
    }

    pub fn from_str(day: &str) -> Day {
        match day {
            "mon" => Day::Mon,
            "thu" => Day::Thu,
            "sun" => Day::Sun,
            &_ => panic!("Not a day when games are playedll"),
        }
    }
}

/// Returns tuple of (max: f32,min: f32)
fn get_max_min(season: i16, week: i8, field: &str, pos: Pos) -> (f32, f32) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT MAX({}) FROM {} WHERE week = ?1 AND season = ?2",
                field,
                pos.get_proj_table()
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");
    let mut min_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT MIN({}) FROM {} WHERE week = ?1 AND season = ?2",
                field,
                pos.get_proj_table()
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");

    let max: f32 = max_statement
        .query_row((week, season), |r| r.get(0))
        .unwrap();
    let mut min: f32 = min_statement
        .query_row((week, season), |r| r.get(0))
        .unwrap();

    if min <= 0.0 {
        min = min - 0.2;
    } else {
        min = 0.0;
    }

    (max, min)
}

fn get_def_max_min(pos: &Pos) -> (f32, f32) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(format!("SELECT MAX(pts_given_pg) FROM {}", pos.get_def_table()).as_str())
        .expect("Couldn't prepare statement..");
    let max: f32 = max_statement.query_row((), |r| r.get(0)).unwrap();
    (max, 0.0)
}

/// Returns the inverse of the score, remove after cummulative
fn get_inverse_max_min(season: i16, week: i8, field: &str, pos: &Pos) -> (f32, f32) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT MAX({}) FROM {} WHERE week = ?1 AND season = ?2",
                field,
                pos.get_proj_table()
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");

    let max: f32 = max_statement
        .query_row((week, season), |r| r.get(0))
        .unwrap();
    (0.0, -1.0 * max)
}

/// Returns avg of field minus 20%
fn get_field_filler(season: i16, week: i8, field: &str, table: &str) -> f32 {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut avg_statement = conn
        .prepare_cached(
            format!(
                "SELECT AVG({}) FROM {} WHERE week = ?1 AND season = ?2 AND ?3 > 0.0",
                field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");

    let avg: f32 = avg_statement
        .query_row((week, season, field), |r| r.get(0))
        .unwrap();
    avg - (avg * 0.20)
}

// Avoid the clone by passing a mutable reference
fn get_median(vec: &mut Vec<f32>) -> f32 {
    if vec.is_empty() {
        return 0.0;
    }
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = vec.len() / 2;

    if vec.len() % 2 == 1 {
        vec[index] as f32
    } else {
        (vec[index - 1] as f32 + vec[index] as f32) / 2.0
    }
}

/// Returns median of field no built in function for median
fn get_field_median(season: i16, week: i8, field: &str, table: &str, limit: i8) -> f32 {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut avg_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT {} FROM {} WHERE week = ?1 AND season = ?2 AND ?3 > 0.0 ORDER BY rating DESC LIMIT ?4",
                field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");

    let mut values: Vec<f32> = avg_statement
        .query_map((week, season, field, limit), |r| r.get(0))
        .unwrap()
        .into_iter()
        .map(|p| p.unwrap())
        .collect::<Vec<f32>>();
    get_median(&mut values)
}

fn get_max_min_all(season: i16, week: i8, field: &str) -> (f32, f32) {
    let tables: [Pos; 5] = [Pos::D, Pos::Qb, Pos::Rb, Pos::Te, Pos::Wr];
    let mut max: f32 = 0.0;
    let mut min: f32 = 200.0;
    for table in tables {
        let max_min: (f32, f32) = get_max_min(season, week, field, table);
        if max_min.0 > max {
            max = max_min.0
        }
        if max_min.1 < min {
            min = max_min.1
        }
    }
    (max, min)
}

fn get_max_min_flex(season: i16, week: i8, field: &str, positions: &[Pos]) -> (f32, f32) {
    let mut max: f32 = 0.0;
    let mut min: f32 = 200.0;
    for pos in positions {
        let max_min: (f32, f32) = get_max_min(season, week, field, *pos);
        if max_min.0 > max {
            max = max_min.0
        }
        if max_min.1 < min {
            min = max_min.1
        }
    }
    (max, min)
}

// TODO this will not get kickers
// These Ids should be cached in an option.
pub fn get_slate(week: i8, season: i16, day: &Day, filter_top: bool) -> Vec<Rc<LitePlayer>> {
    if SLATE_CACHE.read().unwrap().len() > 0 {
        return SLATE_CACHE
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .map(|lp| Rc::new(lp))
            .collect::<Vec<Rc<LitePlayer>>>();
    }

    let mut players: Vec<LitePlayer> = Vec::new();
    let top_qb: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Qb, QB_COUNT, day);
    let top_rb: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Rb, RB_COUNT, day);
    let top_wr: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Wr, WR_COUNT, day);
    let top_te: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Te, TE_COUNT, day);
    let top_d: Vec<i16> = get_top_players_by_pos(season, week, &Pos::D, D_COUNT, day);
    let top_ids: [Vec<i16>; 5] = [top_qb, top_rb, top_d, top_te, top_wr];
    for ids in top_ids {
        players.extend(get_players_by_ids(week, season, &ids))
    }
    if filter_top {
        players = filter_top_players(players, day);
    }

    SLATE_CACHE.write().unwrap().extend(players.clone());

    players
        .into_iter()
        .map(|lp| Rc::new(lp))
        .collect::<Vec<Rc<LitePlayer>>>()
}

fn should_filter_top_player(lp: &LitePlayer, conn: &Connection, filter_ids: &Vec<i16>) -> bool {
    let proj = query_proj(&Some(Rc::new(*lp)), WEEK, SEASON, conn);
    match proj {
        Proj::QbProj(qb) => {
            if !filter_ids.contains(&qb.id) {
                return false;
            }
            if qb.pts_sal_proj < 3.0 {
                return true;
            }
            false
        }
        Proj::RecProj(rec) => {
            if !filter_ids.contains(&rec.id) {
                return false;
            }
            if rec.pts_sal_proj < 3.0 {
                return true;
            }
            false
        }
        Proj::RbProj(rb) => {
            if !filter_ids.contains(&rb.id) {
                return false;
            }
            if rb.pts_sal_proj < 3.0 {
                return true;
            }
            false
        }
        Proj::DefProj(def) => {
            if !filter_ids.contains(&def.id) {
                return false;
            }
            if def.pts_sal_proj < 3.0 {
                return true;
            }
            false
        }
        Proj::KickProj(k) => {
            if !filter_ids.contains(&k.id) {
                return false;
            }
            if k.pts_sal_proj < 3.0 {
                return true;
            }
            false
        }
    }
}

fn filter_top_players(players: Vec<LitePlayer>, day: &Day) -> Vec<LitePlayer> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut filtered: Vec<LitePlayer> = Vec::new();
    let top_qbs = get_top_salary(SEASON, WEEK, &Pos::Qb, day, 3);
    let top_rbs = get_top_salary(SEASON, WEEK, &Pos::Rb, day, 2);
    for player in &players {
        match player.pos {
            Pos::Qb => {
                if !should_filter_top_player(player, &conn, &top_qbs) {
                    filtered.push(player.clone())
                }
            }
            Pos::Rb => {
                if !should_filter_top_player(player, &conn, &top_rbs) {
                    filtered.push(player.clone())
                }
            }
            _ => filtered.push(player.clone()),
        }
    }
    filtered
}

pub fn get_rc_players_by_ids(week: i8, season: i16, ids: &[i16]) -> Vec<Rc<LitePlayer>> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut players = Vec::new();
    for id in ids {
        players.push(get_rc_player_by_id(week, *id, season, &conn));
    }
    players
}

pub fn get_players_by_ids(week: i8, season: i16, ids: &[i16]) -> Vec<LitePlayer> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut players = Vec::new();
    for id in ids {
        players.push(get_player_by_id(week, *id, season, &conn));
    }
    players
}

pub fn get_rc_player_by_id(week: i8, id: i16, season: i16, conn: &Connection) -> Rc<LitePlayer> {
    let query = "SELECT * FROM ownership WHERE week = ?1 AND id = ?2 AND season = ?3";
    let mut stmt = conn.prepare_cached(query).unwrap();
    stmt.query_row(params![week, id, season], |row| {
        Ok(Rc::new(LitePlayer {
            id: row.get(0).unwrap(),
            salary: row.get(8).unwrap(),
            pos: Pos::from_string(row.get(7).unwrap()).unwrap(),
        }))
    })
    .unwrap()
}

pub fn get_player_by_id(week: i8, id: i16, season: i16, conn: &Connection) -> LitePlayer {
    let query = "SELECT * FROM ownership WHERE week = ?1 AND id = ?2 AND season = ?3";
    let mut stmt = conn.prepare_cached(query).unwrap();
    stmt.query_row(params![week, id, season], |row| {
        Ok(LitePlayer {
            id: row.get(0).unwrap(),
            salary: row.get(8).unwrap(),
            pos: Pos::from_string(row.get(7).unwrap()).unwrap(),
        })
    })
    .unwrap()
}

pub fn get_active_players(season: i16, week: i8, day: &Day) -> Vec<i16> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut query = conn
        .prepare("SELECT id FROM ownership WHERE week = ?1 AND season = ?2 AND day = ?3")
        .unwrap();
    let ids: Vec<i16> = query
        .query_map((week, season, day.to_str()), |r| r.get(0))
        .unwrap()
        .into_iter()
        .map(|p| p.unwrap())
        .collect::<Vec<i16>>();
    return ids;
}

struct IdAndScore {
    id: i16,
    score: f32,
}

fn get_score_for_pos(lp: &LitePlayer, week: i8, season: i16, conn: &Connection) -> f32 {
    let proj = query_proj(&Some(Rc::new(lp.clone())), week, season, &conn);
    match proj {
        Proj::QbProj(qb_proj) => qb_score(&qb_proj, &conn),
        Proj::RecProj(rec_proj) => {
            if rec_proj.pos == Pos::Wr {
                wr_stud_score(&rec_proj, &conn, false)
            } else {
                te_score(&rec_proj, &conn)
            }
        }
        Proj::RbProj(rb_proj) => rb_score(&[&rb_proj], conn, false),
        Proj::DefProj(def_proj) => dst_score(&def_proj),
        Proj::KickProj(kick_proj) => score_kicker(&kick_proj),
    }
}

/// Takes all players for week/day and filters using our scoring
pub fn get_top_players_by_pos(season: i16, week: i8, pos: &Pos, count: i8, day: &Day) -> Vec<i16> {
    let conn = &Connection::open(DATABASE_FILE).unwrap();
    let ids = get_active_players(season, week, day);
    if ids.len() == 0 {
        panic!("No players found for pos")
    }
    let mut id_and_score = Vec::new();
    let players = LitePlayer::ids_to_liteplayer(&ids, conn);
    for lp in players.iter().filter(|lp| lp.pos == *pos) {
        let score = get_score_for_pos(lp, week, season, conn);
        id_and_score.push(IdAndScore { id: lp.id, score })
    }
    let take: usize = min(count as usize, id_and_score.len());
    id_and_score.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let mut index: usize = 0;
    let mut result = Vec::new();
    while index < take {
        result.push(id_and_score.get(index).unwrap().id);
        index += 1;
    }
    result
}

// Needs to factor in DAY
pub fn get_top_salary(season: i16, week: i8, pos: &Pos, day: &Day, count: i8) -> Vec<i16> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let lp = LitePlayer::ids_to_liteplayer(&get_active_players(season, week, day), &conn);
    let mut pos_players = lp
        .iter()
        .filter(|lp| lp.pos == *pos)
        .map(|lp| query_proj(&Some(Rc::new(*lp)), week, season, &conn))
        .collect::<Vec<Proj>>();
    pos_players.sort_by(|a, b| {
        b.get_proj_salary()
            .partial_cmp(&a.get_proj_salary())
            .unwrap()
    });
    pos_players
        .iter()
        .map(|p| p.get_proj_id())
        .collect::<Vec<i16>>()[0..count as usize]
        .to_vec()
}

pub fn mean(data: &[f32]) -> Option<f32> {
    let count: usize = data.len();
    if count == 0 {
        return None;
    }
    let sum = data.iter().sum::<f32>();
    Some(sum / count as f32)
}

pub fn return_if_field_exits(
    field: Option<Rc<LitePlayer>>,
    set_to: &Rc<LitePlayer>,
) -> Rc<LitePlayer> {
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

pub fn sort_ownership(ids: &[i16], week: i8, season: i16) -> Vec<i16> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let lps = LitePlayer::ids_to_liteplayer(ids, &conn);
    let mut projs = Vec::new();
    for lp in lps {
        projs.push(query_proj(&Some(Rc::new(lp)), week, season, &conn));
    }
    projs.sort_by(|a, b: &Proj| b.get_proj_own().partial_cmp(&a.get_proj_own()).unwrap());
    projs
        .into_iter()
        .map(|p| p.get_proj_id())
        .collect::<Vec<i16>>()
}

pub fn get_sunday_ownership_cut_off(week: i8, season: i16) -> f32 {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let qbs = sort_ownership(
        &get_top_players_by_pos(season, week, &Pos::Qb, 100, &Day::Sun),
        week,
        season,
    );
    let wrs = sort_ownership(
        &get_top_players_by_pos(season, week, &Pos::Wr, 100, &Day::Sun),
        week,
        season,
    );
    let tes = sort_ownership(
        &get_top_players_by_pos(season, week, &Pos::Te, 100, &Day::Sun),
        week,
        season,
    );
    let rbs = sort_ownership(
        &get_top_players_by_pos(season, week, &Pos::Rb, 100, &Day::Sun),
        week,
        season,
    );
    let defs = sort_ownership(
        &get_top_players_by_pos(season, week, &Pos::D, 100, &Day::Sun),
        week,
        season,
    );
    let mut lineup = LineupBuilder::new();

    for i in 0..3 {
        lineup = lineup.clone().set_pos(
            &Rc::new(LitePlayer::id_to_liteplayer(&wrs[i], &conn)),
            lineup::Slot::int_to_slot(i as i8 + 1),
        );
    }
    for i in 0..2 {
        lineup = lineup.clone().set_pos(
            &Rc::new(LitePlayer::id_to_liteplayer(&rbs[i], &conn)),
            lineup::Slot::int_to_slot(i as i8 + 1),
        );
    }
    lineup = lineup
        .clone()
        .set_pos(
            &Rc::new(LitePlayer::id_to_liteplayer(&tes[0], &conn)),
            lineup::Slot::None,
        )
        .set_pos(
            &Rc::new(LitePlayer::id_to_liteplayer(&defs[0], &conn)),
            lineup::Slot::None,
        )
        .set_pos(
            &Rc::new(LitePlayer::id_to_liteplayer(&wrs[3], &conn)),
            lineup::Slot::Flex,
        )
        .set_pos(
            &Rc::new(LitePlayer::id_to_liteplayer(&qbs[0], &conn)),
            lineup::Slot::None,
        );
    let max_own = lineup
        .build(week, season, &conn)
        .unwrap()
        .get_cum_ownership();

    max_own - (max_own * OWNERSHIP_CUTOFF_PER)
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use num_bigint::ToBigUint;

    use crate::lineup::{get_normalized_score, qb_score, rb_score, Lineup};

    use super::*;
    // Helper function for creating line ups

    #[test]
    fn get_max_sunday_ownership_test() {
        println!("{}", get_sunday_ownership_cut_off(1, 2023))
    }

    #[test]
    fn test_slice_iter() {
        let test = [0, 1, 2, 3, 4];
        test[0..2].iter().for_each(|i| println!("{}", i))
    }

    #[test]
    fn test_total_comb() {
        assert_eq!(253, total_comb(23, 2));
    }

    #[test]
    fn test_factorial() {
        assert_eq!(5040.to_biguint().unwrap(), factorial(7));
    }

    #[test]
    fn test_filter_top_players() {
        let players = get_slate(1, 2023, &Day::Sun, true);
        players.iter().for_each(|p| assert!(p.id != 118));
        let mut found: bool = false;
        let no_filter_players = get_slate(1, 2023, &Day::Sun, false);
        no_filter_players.iter().for_each(|p| {
            if p.id == 118 {
                found = true;
            }
        });
        assert!(found);
    }

    #[test]
    fn test_get_top_players() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let player = get_top_players_by_pos(2023, 1, &Pos::Wr, 25, &Day::Sun);
        let projs = player
            .iter()
            .map(|id| {
                query_proj(
                    &Some(Rc::new(LitePlayer::id_to_liteplayer(id, &conn))),
                    1,
                    2023,
                    &conn,
                )
            })
            .collect::<Vec<Proj>>();
        projs.iter().for_each(|p| println!("{:?}", p.get_name()))
    }

    #[test]
    fn all_rb_scores() {
        let players = get_top_players_by_pos(SEASON, WEEK, &Pos::Rb, 50, &Day::Mon);
        let conn = Connection::open(DATABASE_FILE).unwrap();
        for rb in players {
            let rb_proj = query_rb_proj(rb, WEEK, SEASON, &conn).unwrap();
            println!(
                "{}: {} \n\n",
                &rb_proj.name,
                rb_score(&[&rb_proj], &conn, false)
            );
        }
    }

    #[test]
    fn all_qb_scores() {
        let players = get_top_players_by_pos(SEASON, WEEK, &Pos::Qb, 50, &Day::Sun);
        let conn = Connection::open(DATABASE_FILE).unwrap();
        for qb in players {
            let qb_proj = query_qb_proj(qb, WEEK, SEASON, &conn).unwrap();
            println!("{}: {} \n\n", &qb_proj.name, qb_score(&qb_proj, &conn));
        }

        println!("{}", get_normalized_score(-10.0, (0.0, -11.0)));
        println!("{}", get_normalized_score(-1.0, (0.0, -11.0)));
    }



    #[test]
    fn test_mean() {
        let mean = mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mean, Some(3.0));
    }

    #[test]
    fn test_gen_flat_comb() {
        let mut lite_players: Vec<Rc<LitePlayer>> = Vec::new();
        for i in 0..4 {
            lite_players.push(Rc::new(LitePlayer {
                id: i,
                pos: Pos::Wr,
                salary: 100,
            }));
        }

        for combo in lite_players.iter().combinations(2) {
            println!("{:?}", combo)
        }
    }

    #[test]
    fn test_max_min_all() {
        println!("{:?}", get_max_min_all(2023, 1, "floor_proj"));
    }

    #[test]
    fn test_get_avg() {
        println!("{:?}", get_field_filler(2023, 1, "avg_atts", "rb_proj"));
    }

    #[test]
    fn te_pass_per_median_test() {
        println!("{}", *QB_TE_PASS_PER_MEDIAN)
    }
}
