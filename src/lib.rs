use std::{cmp::min, fs, rc::Rc};

use lazy_static::lazy_static;
use lineup::{dst_score, qb_score, rb_score, score_kicker, te_score, wr_stud_score};
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

pub const FILTER_TOP_QB: i8 = 3;
pub const FILTER_TOP_RB: i8 = 2;

pub const WR_COUNT: i8 = 20;
pub const QB_COUNT: i8 = 12;
pub const TE_COUNT: i8 = 12;
pub const RB_COUNT: i8 = 20;
pub const D_COUNT: i8 = 10;

lazy_static! {
    // pub static ref QB_SCORE_MAX_MIN: (f32, f32) = get_max_min_pos_score(SEASON, WEEK, &Pos::Qb);
    // pub static ref RB_SCORE_MAX_MIN: (f32, f32) = get_max_min_pos_score(SEASON, WEEK, &Pos::Rb);
    // pub static ref WR_SCORE_MAX_MIN: (f32, f32) = get_max_min_pos_score(SEASON, WEEK, &Pos::Wr);
    // pub static ref TE_SCORE_MAX_MIN: (f32, f32) = get_max_min_pos_score(SEASON, WEEK, &Pos::Te);
    // pub static ref D_SCORE_MAX_MIN: (f32, f32) = get_max_min_pos_score(SEASON, WEEK, &Pos::D);
    // pub static ref K_SCORE_MAX_MIN: (f32, f32) = get_max_min_pos_score(SEASON, WEEK, &Pos::K);

    // QB Stats
    pub static ref QB_RUSH_ATT_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "avg_rush_atts", "qb_proj");
    pub static ref QB_RUSH_ATT_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "avg_rush_atts", "qb_proj");
    pub static ref QB_AVG_RZ_OP_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "red_zone_op_pg", "qb_proj");
    pub static ref QB_AVG_RZ_OP_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "red_zone_op_pg", "qb_proj");
    pub static ref QB_OWN_PROJ_MAX_MIN: (f32, f32) =
        get_own_max_min(SEASON, WEEK, "qb_proj");
    pub static ref QB_WR_PASS_PER_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "pass_to_wr_per", "qb_proj");
    pub static ref QB_VEGAS_TOTAL_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "vegas_total", "qb_proj");
    pub static ref QB_TE_PASS_PER_MEDIAN: f32 =
        get_field_median(SEASON, WEEK, "pass_to_te_per", "qb_proj", QB_COUNT);
    pub static ref QB_PTS_PER_SAL: (f32, f32) = get_max_min(SEASON, WEEK , "pts_sal_proj", "qb_proj");
    pub static ref QB_CIELING: (f32, f32) = get_max_min(SEASON, WEEK, "cieling_proj", "qb_proj");
    pub static ref QB_OPP_DEF_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts_given_pg", "def_vs_qb");

    // RB Stats
    pub static ref RB_ATTS_FILLER: f32 = get_field_filler(SEASON, WEEK, "avg_atts", "rb_proj");
    pub static ref RB_ATTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "avg_atts", "rb_proj");
    pub static ref RB_SNAPS_PER_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "snaps_per", "rb_proj");
    pub static ref RB_SNAPS_PER_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "snaps_per", "rb_proj");
    pub static ref RB_AVG_REC_YDS: (f32, f32) = get_max_min(SEASON, WEEK, "avg_rec_yds", "rb_proj");
    pub static ref RB_AVG_REC_YDS_FILLER: f32 = get_field_filler(SEASON, WEEK, "avg_rec_yds", "rb_proj");
    pub static ref RB_OWN_PROJ_MAX_MIN: (f32, f32) =
        get_own_max_min(SEASON, WEEK, "rb_proj");
    pub static ref RB_PTS_PLUS_MINUS: (f32, f32) =
        get_max_min(SEASON, WEEK, "pts_plus_minus_proj", "rb_proj");
    pub static ref RB_VEGAS_TOTAL_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "vegas_total", "rb_proj");
    pub static ref RB_YEAR_CONSISTENCY_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "year_consistency", "rb_proj");
    pub static ref RB_YEAR_CONSISTENCY_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "year_consistency", "rb_proj");
    pub static ref RB_OPP_DEF_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts_given_pg", "def_vs_rb");

    // WR Stats
    pub static ref WR_TGT_SHARE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "rec_tgt_share", "wr_proj");
    pub static ref WR_TGT_SHARE_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "rec_tgt_share", "wr_proj");
    pub static ref WR_RED_ZONE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "red_zone_op_pg", "wr_proj");
    pub static ref WR_RED_ZONE_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "red_zone_op_pg", "wr_proj");
    pub static ref WR_OWN_PROJ_MAX_MIN: (f32, f32) =
        get_own_max_min(SEASON, WEEK, "wr_proj");
    pub static ref WR_VEGAS_TOTAL_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "vegas_total", "wr_proj");
    pub static ref WR_YEAR_CONSISTENCY_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "year_consistency", "wr_proj");
    pub static ref WR_YEAR_CONSISTENCY_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "year_consistency", "wr_proj");
    pub static ref WR_YEAR_UPSIDE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "year_upside", "wr_proj");
    pub static ref WR_YEAR_UPSIDE_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "year_upside", "wr_proj");
    pub static ref WR_SALARY_MEDIAN: f32 =
        get_field_median(SEASON, WEEK, "salary", "wr_proj", WR_COUNT);

    // TE Stats
    pub static ref TE_REC_TGT_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "rec_tgt_share", "te_proj");
    pub static ref TE_REC_TGT_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "rec_tgt_share", "te_proj");
    pub static ref TE_RED_ZONE_FILLER: f32 =
        get_field_filler(SEASON, WEEK, "red_zone_op_pg", "te_proj");
    pub static ref TE_RED_ZONE_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "red_zone_op_pg", "te_proj");
    pub static ref TE_OWN_PROJ_MAX_MIN: (f32, f32) =
        get_own_max_min(SEASON, WEEK, "te_proj");
    pub static ref TE_VEGAS_TOTAL_MAX_MIN: (f32, f32) =
        get_max_min(SEASON, WEEK, "vegas_total", "vegas_total");

    // ALL Proj
    pub static ref ALL_PTS_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "pts_proj");
    pub static ref ALL_FLOOR_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "floor_proj");
    pub static ref ALL_CIELING_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "cieling_proj");
    pub static ref ALL_PTS_SAL_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "pts_sal_proj");
    pub static ref ALL_PTS_PLUS_MINS_MAX_MIN: (f32, f32) =
        get_max_min_all(SEASON, WEEK, "pts_plus_minus_proj");

    // DST Stats
    pub static ref DST_RATING_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "rating", "dst_proj");
    pub static ref DST_OWN_PROJ_MAX_MIN: (f32, f32) =
        get_own_max_min(SEASON, WEEK, "dst_proj");
    pub static ref DST_VEGAS_OPP_TOTAL: (f32, f32) =
        get_max_min(SEASON, WEEK, "vegas_opp_total", "dst_proj");
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
fn get_max_min(season: i16, week: i8, field: &str, table: &str) -> (f32, f32) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT MAX({}) FROM {} WHERE week = ?1 AND season = ?2",
                field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");
    let mut min_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT MIN({}) FROM {} WHERE week = ?1 AND season = ?2",
                field, table
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

/// Returns the inverse of the score, remove after cummulative
fn get_own_max_min(season: i16, week: i8, table: &str) -> (f32, f32) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement: rusqlite::CachedStatement<'_> = conn
        .prepare_cached(
            format!(
                "SELECT MAX(own_proj) FROM {} WHERE week = ?1 AND season = ?2",
                table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");

    let max: f32 = max_statement
        .query_row((week, season), |r| r.get(0))
        .unwrap();
    (0.0, -1.0 * max)
}

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
    let tables: [&str; 5] = ["dst_proj", "qb_proj", "rb_proj", "te_proj", "wr_proj"];
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

// TODO could put tables in list and than iterate through that
pub fn get_sunday_slate(week: i8, season: i16, filter_top: bool) -> Vec<Rc<LitePlayer>> {
    let mut players: Vec<Rc<LitePlayer>> = Vec::new();
    let top_qb: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Qb, QB_COUNT);
    let top_rb: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Rb, RB_COUNT);
    let top_wr: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Wr, WR_COUNT);
    let top_te: Vec<i16> = get_top_players_by_pos(season, week, &Pos::Te, TE_COUNT);
    let top_d: Vec<i16> = get_top_players_by_pos(season, week, &Pos::D, D_COUNT);
    let top_ids: [Vec<i16>; 5] = [top_qb, top_rb, top_d, top_te, top_wr];
    for ids in top_ids {
        players.extend(get_players_by_ids(week, season, &ids))
    }
    if filter_top {
        players = filter_top_players(players);
    }

    players
}

fn filter_top_players(players: Vec<Rc<LitePlayer>>) -> Vec<Rc<LitePlayer>> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut filtered: Vec<Rc<LitePlayer>> = Vec::new();
    let top_qbs = get_top_salary(SEASON, WEEK, &Pos::Qb, 3);
    let top_rbs = get_top_salary(SEASON, WEEK, &Pos::Rb, 2);
    for player in &players {
        match player.pos {
            Pos::Qb => {
                if top_qbs.contains(&player.id) {
                    let proj: QbProj =
                        query_qb_proj_helper(&Some(player.clone()), WEEK, SEASON, &conn);
                    if proj.pts_sal_proj > 2.9 {
                        filtered.push(player.clone());
                    } else {
                        println!("Filtered: {}", proj.name);
                    }
                } else {
                    filtered.push(player.clone())
                }
            }
            Pos::Rb => {
                if top_rbs.contains(&player.id) {
                    let proj: RbProj =
                        query_rb_proj_helper(&Some(player.clone()), WEEK, SEASON, &conn);
                    if proj.pts_sal_proj > 2.9 {
                        filtered.push(player.clone());
                    } else {
                        println!("Filtered: {}", proj.name)
                    }
                } else {
                    filtered.push(player.clone())
                }
            }
            _ => (),
        }
    }
    filtered
}

pub fn get_players_by_ids(week: i8, season: i16, ids: &[i16]) -> Vec<Rc<LitePlayer>> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut players = Vec::new();
    for id in ids {
        players.push(get_player_by_id(week, *id, season, &conn));
    }
    players
}

pub fn get_player_by_id(week: i8, id: i16, season: i16, conn: &Connection) -> Rc<LitePlayer> {
    let query = "SELECT * FROM ownership WHERE week = ?1 AND id = ?2 AND season = ?3";
    let mut stmt = conn.prepare_cached(query).unwrap();
    stmt.query_row(params![week, id, season], |row| {
        Ok(Rc::new(LitePlayer {
            id: row.get(0).unwrap(),
            salary: row.get(7).unwrap(),
            pos: Pos::from_string(row.get(6).unwrap()).unwrap(),
        }))
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
    println!("Active players..");
    return ids;
}

struct IdAndScore {
    id: i16,
    score: f32,
}

/// Takes all players for week/day and filters using our scoring
pub fn get_top_players_by_pos(season: i16, week: i8, pos: &Pos, count: i8, day: &Day) -> Vec<i16> {
    let conn = &Connection::open(DATABASE_FILE).unwrap();
    let ids = get_active_players(season, week, day);
    let mut players = Vec::new();
    for id in ids {
        let lp = LitePlayer {
            id,
            pos: pos.clone(),
            salary: 0,
        };
        let score = get_score_for_pos(lp, pos, week, season, conn);
        println!("pushing id");
        players.push(IdAndScore { id, score })
    }
    let take: usize = min(count as usize, players.len());
    println!("sorting");
    players.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let mut index: usize = 0;
    let mut result = Vec::new();
    while index < take {
        print!("1");
        result.push(players.get(index).unwrap().id);
        index += 1;
    }
    result
}

pub fn get_top_salary(season: i16, week: i8, pos: &Pos, count: i8) -> Vec<i16> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut query = conn
        .prepare(
            format!(
                "SELECT id FROM {} WHERE week = ?1 AND season = ?2 ORDER BY salary DESC LIMIT ?3",
                pos.get_proj_table()
            )
            .as_str(),
        )
        .unwrap();
    let ids: Vec<i16> = query
        .query_map((week, season, count), |r| r.get(0))
        .unwrap()
        .into_iter()
        .map(|p| p.unwrap())
        .collect::<Vec<i16>>();
    return ids;
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

#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use num_bigint::ToBigUint;

    use crate::{
        lineup::{get_normalized_score, qb_score, rb_score, Lineup},
        tables::init_tables,
    };

    use super::*;
    // Helper function for creating line ups

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
    fn wr_max_mins() {
        println!("{:?}", get_normalized_score(83.0, (83.0, -1.0)));
        println!("{:?}", -1.0 * get_normalized_score(-1.0, (23.04, -1.0)));
        println!("{:?}", get_normalized_score(1.0, (2.8, -1.0)));
        println!("{:?}", get_normalized_score(26.7, (32.0, -1.0)));
        println!("{:?}", get_normalized_score(41.5, (51.0, 0.0)));
        println!("{:?}", get_normalized_score(2.94, (3.0, 0.0)));
        println!(
            "{:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            *WR_OWN_PROJ_MAX_MIN,
            *WR_RED_ZONE_MAX_MIN,
            *WR_SALARY_MEDIAN,
            *WR_TGT_SHARE_MAX_MIN,
            *WR_VEGAS_TOTAL_MAX_MIN,
            *WR_YEAR_CONSISTENCY_MAX_MIN,
            *WR_YEAR_UPSIDE_MAX_MIN
        )
    }

    #[test]
    fn all_rb_scores() {
        let players = get_top_players_by_pos(SEASON, WEEK, &Pos::Rb, 50);
        let conn = Connection::open(DATABASE_FILE).unwrap();
        for rb in players {
            let rb_proj = query_rb_proj(rb, WEEK, SEASON, &conn).unwrap();
            println!("{}: {} \n\n", &rb_proj.name, rb_score(&[&rb_proj]));
        }
    }

    #[test]
    fn all_qb_scores() {
        let players = get_top_players_by_pos(SEASON, WEEK, &Pos::Qb, 50);
        let conn = Connection::open(DATABASE_FILE).unwrap();
        for qb in players {
            let qb_proj = query_qb_proj(qb, WEEK, SEASON, &conn).unwrap();
            println!("{}: {} \n\n", &qb_proj.name, qb_score(&qb_proj));
        }

        println!("{}", get_normalized_score(-10.0, (0.0, -11.0)));
        println!("{}", get_normalized_score(-1.0, (0.0, -11.0)));
    }

    #[test]
    fn score_winning_lineup() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let week: i8 = 1;
        let season: i16 = 2023;
        let lineup = Lineup {
            qb: query_qb_proj(27, 1, 2023, &conn).expect(""),
            rb1: query_rb_proj(53, week, season, &conn).unwrap(),
            rb2: query_rb_proj(62, week, season, &conn).unwrap(),
            wr1: query_rec_proj(142, week, season, &Pos::Wr, &conn).unwrap(),
            wr2: query_rec_proj(184, week, season, &Pos::Wr, &conn).unwrap(),
            wr3: query_rec_proj(161, week, season, &Pos::Wr, &conn).unwrap(),
            te: query_rec_proj(139, week, season, &Pos::Te, &conn).unwrap(),
            flex: FlexProj {
                pos: Pos::Wr,
                rec_proj: Some(query_rec_proj(141, week, season, &Pos::Wr, &conn).unwrap()),
                rb_proj: None,
            },
            def: query_def_proj(21, week, season, &conn).unwrap(),
            salary_used: 60000,
        };
        println!("{}", lineup.lineup_str());

        let lineup = Lineup {
            qb: query_qb_proj(36, 1, 2023, &conn).expect(""),
            rb1: query_rb_proj(61, week, season, &conn).unwrap(),
            rb2: query_rb_proj(69, week, season, &conn).unwrap(),
            wr1: query_rec_proj(155, week, season, &Pos::Wr, &conn).unwrap(),
            wr2: query_rec_proj(167, week, season, &Pos::Wr, &conn).unwrap(),
            wr3: query_rec_proj(175, week, season, &Pos::Wr, &conn).unwrap(),
            te: query_rec_proj(107, week, season, &Pos::Te, &conn).unwrap(),
            flex: FlexProj {
                pos: Pos::Wr,
                rec_proj: Some(query_rec_proj(163, week, season, &Pos::Wr, &conn).unwrap()),
                rb_proj: None,
            },
            def: query_def_proj(1, week, season, &conn).unwrap(),
            salary_used: 60000,
        };
        println!("{}", lineup.lineup_str());
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
    fn test_get_top_players() {
        println!("{:?}", get_top_players_by_pos(2023, 1, &Pos::Qb, 10));
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
