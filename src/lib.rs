use std::{fs, sync::Arc};

use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use rusqlite::{params, Connection, Params};

use crate::player::*;

use itertools::Itertools;
pub mod data_loader;
pub mod island_optimizer;
pub mod lineup;
pub mod optimizer;
pub mod player;
pub mod tables;

pub const DATABASE_FILE: &str = "./dfs_nfl.db3";
const SEASON: i16 = 2023;
const WEEK: i8 = 1;
const STAT_SEASON: i16 = 2022;
const STAT_WEEK: i8 = 18;

pub const WR_COUNT: i8 = 35;
pub const QB_COUNT: i8 = 10;
pub const TE_COUNT: i8 = 10;
pub const RB_COUNT: i8 = 25;
pub const D_COUNT: i8 = 8;

lazy_static! {
    // TODO Move to map - Proj
    pub static ref QB_RUSH_ATT_FILLER: f32 = get_field_filler(SEASON, WEEK, "avg_rush_atts", "qb_proj");
    pub static ref QB_RUSH_ATT_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "avg_rush_atts", "qb_proj");
    pub static ref QB_AVG_RZ_OP_FILLER: f32 = get_field_filler(SEASON, WEEK, "red_zone_op_pg", "qb_proj");
    pub static ref QB_AVG_RZ_OP_MAX_MIN: (f32, f32) = get_max_min(SEASON,WEEK, "red_zone_op_pg", "qb_proj");
    pub static ref QB_OWN_PROJ_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "own_proj", "qb_proj");
    pub static ref QB_WR_PASS_PER_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pass_to_wr_per", "qb_proj");
    pub static ref QB_VEGAS_TOTAL_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "vegas_total", "qb_proj");
    pub static ref QB_TE_PASS_PER_MEDIAN: f32 = get_field_median(SEASON, WEEK, "pass_to_te_per", "qb_proj", QB_COUNT);

    pub static ref RB_ATTS_FILLER: f32 = get_field_filler(SEASON,WEEK, "avg_atts", "rb_proj");
    pub static ref RB_ATTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "avg_atts", "rb_proj");
    pub static ref RB_SNAPS_PER_FILLER: f32 = get_field_filler(SEASON,WEEK, "snaps_per", "rb_proj");
    pub static ref RB_SNAPS_PER_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "snaps_per", "rb_proj");
    pub static ref RB_AVG_REC_YDS: (f32, f32) = get_max_min(SEASON, WEEK, "avg_rec_yds", "rb_proj");
    pub static ref RB_OWN_PROJ_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "own_proj", "rb_proj");
    pub static ref RB_PTS_PER_SALARY_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts_per_sal", "rb_proj");
    pub static ref RB_VEGAS_TOTAL_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "vegas_total", "rb_proj");
    pub static ref RB_YEAR_CONSISTENCY_FILLER: f32 = get_field_filler(SEASON,WEEK, "year_consistency", "rb_proj");
    pub static ref RB_YEAR_CONSISTENCY_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "year_consistency", "rb_proj");

    pub static ref WR_TGT_SHARE_FILLER: f32 = get_field_filler(SEASON,WEEK, "rec_tgt_share", "wr_proj");
    pub static ref WR_TGT_SHARE_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "rec_tgt_share", "wr_proj");
    pub static ref WR_RED_ZONE_FILLER: f32 = get_field_filler(SEASON, WEEK, "red_zone_op_pg", "wr_proj");
    pub static ref WR_RED_ZONE_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "red_zone_op_pg", "wr_proj");
    pub static ref WR_OWN_PROJ_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "own_proj", "wr_proj");
    pub static ref WR_VEGAS_TOTAL_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "vegas_total", "wr_proj");
    pub static ref WR_YEAR_CONSISTENCY_FILLER: f32 = get_field_filler(SEASON,WEEK, "year_consistency", "wr_proj");
    pub static ref WR_YEAR_CONSISTENCY_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "year_consistency", "wr_proj");
    pub static ref WR_YEAR_UPSIDE_FILLER: f32 = get_field_filler(SEASON,WEEK, "year_upside", "wr_proj");
    pub static ref WR_YEAR_UPSIDE_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "year_upside", "wr_proj");
    pub static ref WR_SALARY_MEDIAN: f32 = get_field_median(SEASON, WEEK, "salary", "wr_proj", WR_COUNT);

    pub static ref TE_REC_TGT_FILLER: f32 = get_field_filler(SEASON, WEEK, "rec_tgt_share", "te_proj");
    pub static ref TE_REC_TGT_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "rec_tgt_share", "te_proj");
    pub static ref TE_RED_ZONE_FILLER: f32 = get_field_filler(SEASON, WEEK, "red_zone_op_pg", "te_proj");
    pub static ref TE_RED_ZONE_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "red_zone_op_pg", "te_proj");
    pub static ref TE_OWN_PROJ_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "own_proj", "te_proj");
    pub static ref TE_VEGAS_TOTAL_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "vegas_total", "vegas_total");

    pub static ref ALL_PTS_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "pts_proj");
    pub static ref ALL_FLOOR_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "floor_proj");
    pub static ref ALL_CIELING_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "cieling_proj");
    pub static ref ALL_PTS_SAL_MAX_MIN: (f32, f32) = get_max_min_all(SEASON, WEEK, "pts_sal_proj");

    pub static ref DST_RATING_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "rating", "dst_proj");
    pub static ref DST_OWN_PROJ_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "own_proj", "dst_proj");
    pub static ref DST_VEGAS_OPP_TOTAL: (f32, f32) = get_max_min(SEASON, WEEK, "vegas_opp_total", "dst_proj");

    pub static ref OWN_PER_MAX_MIN: (f32, f32) = get_max_min_ownership(SEASON, WEEK);
}

fn get_own_per(query: &str, pos: &Pos, conn: &Connection) -> f32 {
    conn.query_row(query, params![pos.to_str().unwrap()], |r| r.get(0))
        .unwrap()
}

// TODO Does not work
fn get_max_min_ownership(season: i16, week: i8) -> (f32, f32) {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let max_query = format!(
        "SELECT MAX(own_per) FROM ownership WHERE pos = ?1 AND week = {} AND season = {} ORDER BY rating DESC LIMIT 10",
        week, season
    );
    let min_query = format!(
        "SELECT MIN(own_per) FROM ownership WHERE pos = ?1 AND week = {} AND season = {}",
        week, season
    );
    let max_positions = &[&Pos::Qb, &Pos::Wr, &Pos::Te, &Pos::Rb];
    let min_positions = &[&Pos::K, &Pos::Te, &Pos::D];

    let max_sum: f32 = max_positions
        .iter()
        .map(|x| get_own_per(&max_query, x, &conn))
        .sum();
    let min_sum: f32 = min_positions
        .iter()
        .map(|x| get_own_per(&min_query, x, &conn))
        .sum();
    let own_max_avg = max_sum / 4.0;
    let own_min_avg = 4.1;
    (own_max_avg, own_min_avg)
}

/// Returns tuple of (max: f32,min: f32)
fn get_max_min(season: i16, week: i8, field: &str, table: &str) -> (f32, f32) {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement = conn
        .prepare_cached(
            format!(
                "SELECT MAX({}) FROM {} WHERE week = ?1 AND season = ?2",
                field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");
    let mut min_statement = conn
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
        min = min - 1.0;
    } else {
        min = 0.0;
    }

    (max, min)
}

fn get_field_filler(season: i16, week: i8, field: &str, table: &str) -> f32 {
    let conn = Connection::open(DATABASE_FILE).unwrap();
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
    let sub_amt = avg * 0.20;
    avg - sub_amt
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
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut avg_statement = conn
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
    return get_median(&mut values);
}

fn get_max_min_all(season: i16, week: i8, field: &str) -> (f32, f32) {
    let tables: [&str; 5] = [
        "dst_proj", // "kick_proj",
        "qb_proj", "rb_proj", "te_proj", "wr_proj",
    ];
    let mut max: f32 = 0.0;
    let mut min: f32 = 200.0;
    for table in tables {
        let max_min = get_max_min(season, week, field, table);
        if max_min.0 > max {
            max = max_min.0
        }
        if max_min.1 < min {
            min = max_min.1
        }
    }
    (max, min)
}

pub fn get_all_active_players_pos(pos: &Pos, week: i8) -> Vec<Arc<LitePlayer>> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let query = "SELECT * FROM ownership WHERE pos = ?1 AND week = ?2";
    let mut stmt = conn.prepare_cached(query).unwrap();
    stmt.query_map(params![pos.to_str().unwrap(), week], |row| {
        Ok(Arc::new(LitePlayer {
            id: row.get(0).unwrap(),
            salary: row.get(7).unwrap(),
            pos: Pos::from_string(row.get(6).unwrap()).unwrap(),
        }))
    })
    .unwrap()
    .into_iter()
    .map(|p| p.unwrap())
    .collect::<Vec<Arc<LitePlayer>>>()
}

pub fn get_all_active_players(week: i8) -> Vec<Arc<LitePlayer>> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let query = "SELECT * FROM ownership WHERE week = ?1";
    let mut stmt = conn.prepare_cached(query).unwrap();
    stmt.query_map(params![week], |row| {
        Ok(Arc::new(LitePlayer {
            id: row.get(0).unwrap(),
            salary: row.get(7).unwrap(),
            pos: Pos::from_string(row.get(6).unwrap()).unwrap(),
        }))
    })
    .unwrap()
    .into_iter()
    .map(|p| p.unwrap())
    .collect::<Vec<Arc<LitePlayer>>>()
}

pub fn get_players_by_ids(week: i8, ids: &[f32]) -> Vec<Arc<LitePlayer>> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let query = "SELECT * FROM ownership WHERE week = ?1 AND id = ?2";
    let mut stmt = conn.prepare_cached(query).unwrap();
    let mut players = Vec::new();
    for id in ids {
        players.push(
            stmt.query_row(params![week, id], |row| {
                Ok(Arc::new(LitePlayer {
                    id: row.get(0).unwrap(),
                    salary: row.get(7).unwrap(),
                    pos: Pos::from_string(row.get(6).unwrap()).unwrap(),
                }))
            })
            .unwrap(),
        );
    }
    players
}

pub fn get_top_players(season: i16, week: i8, table: &str, count: i8) -> Vec<f32> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut query = conn
        .prepare(
            format!(
                "SELECT id FROM {} WHERE week = ?1 AND season = ?2 ORDER BY rating DESC LIMIT ?3",
                table
            )
            .as_str(),
        )
        .unwrap();
    let ids: Vec<f32> = query
        .query_map((week, season, count), |r| r.get(0))
        .unwrap()
        .into_iter()
        .map(|p| p.unwrap())
        .collect::<Vec<f32>>();
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

#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use num_bigint::ToBigUint;

    use crate::lineup::get_normalized_score;

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
    fn test_mean() {
        let mean = mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mean, Some(3.0));
    }

    #[test]
    fn test_gen_flat_comb() {
        let mut lite_players: Vec<Arc<LitePlayer>> = Vec::new();
        for i in 0..4 {
            lite_players.push(Arc::new(LitePlayer {
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
    fn get_players() {
        for player in get_all_active_players_pos(&Pos::Qb, 18) {
            println!("{:?}", player)
        }
    }

    #[test]
    fn test_max_min_own_per() {
        println!("{:?}", get_max_min_ownership(2023, 1));
    }

    #[test]
    fn test_max_min_all() {
        println!("{:?}", get_max_min_all(2023, 1, "floor_proj"));
    }

    #[test]
    fn test_get_top_players() {
        println!("{:?}", get_top_players(2023, 1, "qb_proj", 10));
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
