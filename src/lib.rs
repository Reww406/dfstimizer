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
const SEASON: i16 = 2022;
const WEEK: i8 = 1;
const STAT_SEASON: i16 = 2022;
const STAT_WEEK: i8 = 18;

lazy_static! {
    // TODO Move to map - Proj
    pub static ref QB_ATT_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "atts", "qb_proj");
    pub static ref QB_PTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts", "qb_proj");
    pub static ref RB_ATT_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "atts", "rb_proj");
    pub static ref RB_PTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts", "rb_proj");
    pub static ref WR_TGTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "tgts", "wr_proj");
    pub static ref WR_PTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts", "wr_proj");
    pub static ref TE_TGTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "tgts", "te_proj");
    pub static ref TE_PTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts", "te_proj");
    pub static ref WR_TDS_MAX_MIN: (f32, f32) =  get_max_min(SEASON, WEEK, "tds", "wr_proj");

    pub static ref QB_PASS_TDS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pass_tds", "qb_proj");
    pub static ref QB_RUSH_YDS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "rush_yds", "qb_proj");

    pub static ref DST_PTS_MAX_MIN: (f32, f32) = get_max_min(SEASON, WEEK, "pts", "dst_proj");
    pub static ref QB_AVG_DEPTH_MAX_MIN: (f32, f32) = get_last_avg_max_min(STAT_SEASON, STAT_WEEK, "avg_depth", "qb_stats", &Pos::Qb);
    pub static ref QB_RECENT_PTS_CIELING: f32 = get_recent_stat_ceiling_all(STAT_SEASON, STAT_WEEK, "fan_pts", "qb_stats", &Pos::Qb);
    pub static ref WR_RECENT_PTS_CIELING: f32 = get_recent_stat_ceiling_all(STAT_SEASON,STAT_WEEK, "fan_pts", "rush_rec_stats", &Pos::Wr);
    pub static ref TE_RECENT_PTS_CIELING: f32 = get_recent_stat_ceiling_all(STAT_SEASON, STAT_WEEK, "fan_pts", "rush_rec_stats", &Pos::Te);
    pub static ref RB_RECENT_PTS_CIELING: f32 = get_recent_stat_ceiling_all(STAT_SEASON, STAT_WEEK, "fan_pts", "rush_rec_stats", &Pos::Rb);
    // TODO Move to map - Stats

    pub static ref QB_AVG_EZ_ATT_MAX_MIN: (f32, f32) = get_last_avg_max_min(STAT_SEASON, STAT_WEEK, "ez_pass_atts", "qb_stats", &Pos::Qb);
    pub static ref QB_AVG_EZ_RATT_MAX_MIN: (f32, f32) = get_last_avg_max_min(STAT_SEASON, STAT_WEEK, "ez_rush_atts", "qb_stats", &Pos::Qb);

    pub static ref WR_AVG_EZ_TGT_MAX_MIN: (f32, f32) = get_last_avg_max_min(STAT_SEASON, STAT_WEEK, "rz_tgts", "rush_rec_stats",&Pos::Wr);
    pub static ref TE_AVG_EZ_TGT_MAX_MIN: (f32, f32) = get_last_avg_max_min(STAT_SEASON, STAT_WEEK, "rz_tgts", "rush_rec_stats",&Pos::Te);
    pub static ref RB_AVG_EZ_ATT_MAX_MIN: (f32, f32) = get_last_avg_max_min(STAT_SEASON, STAT_WEEK, "rz_atts", "rush_rec_stats",&Pos::Rb);
    pub static ref OWN_PER_MAX_MIN: (f32, f32) = get_max_min_ownership(SEASON, WEEK);


}

fn get_own_per(query: &str, pos: &Pos, conn: &Connection) -> f32 {
    conn.query_row(query, params![Pos::Qb.to_str().unwrap()], |r| r.get(0))
        .unwrap()
}

fn get_max_min_ownership(season: i16, week: i8) -> (f32, f32) {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let max_query = format!(
        "SELECT MAX(own_per) FROM ownership WHERE pos = ?1 AND week = {} AND season = {}",
        week, season
    );
    let min_query = format!(
        "SELECT MIN(own_per) FROM ownership WHERE pos = ?1 AND week = {} AND season = {}",
        week, season
    );
    let positions = &[&Pos::Qb, &Pos::Wr, &Pos::Te, &Pos::Rb, &Pos::D];

    let max_sum: f32 = positions
        .iter()
        .map(|x| get_own_per(&max_query, x, &conn))
        .sum();
    let min_sum: f32 = positions
        .iter()
        .map(|x| get_own_per(&min_query, x, &conn))
        .sum();
    let own_max_avg = max_sum / 5.0;
    let own_min_avg = min_sum / 5.0;
    (own_max_avg, own_min_avg)
}

/// Returns tuple of (max: f32,min: f32)
fn get_max_min(season: i16, week: i8, field: &str, table: &str) -> (f32, f32) {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut max_statement = conn
        .prepare(
            format!(
                "SELECT MAX({}) FROM {} WHERE week = ?1 AND season = ?2",
                field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");
    let mut min_statement = conn
        .prepare(
            format!(
                "SELECT MIN({}) FROM {} WHERE week = ?1 AND season = ?2",
                field, table
            )
            .as_str(),
        )
        .expect("Couldn't prepare statement..");

    let max = max_statement
        .query_row((week, season), |r| r.get(0))
        .unwrap();
    let min = min_statement
        .query_row((week, season), |r| r.get(0))
        .unwrap();
    (max, min)
}

//TODO should we change this to an averge?
fn get_recent_stat_ceiling_all(season: i16, week: i8, field: &str, table: &str, pos: &Pos) -> f32 {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let week_range_start: i8 = week - (2);
    let players: Vec<Player> = get_all_active_players(pos, week);
    let mut ceiling: f32 = 0.0;
    let query: String = format!(
        "SELECT ({}) FROM {} WHERE season = ?1 AND week BETWEEN ?2 AND ?3 AND id = ?4",
        field, table
    );
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(query.as_str()).unwrap();
    for player in players {
        stmt.query_map((season, week_range_start, week, player.id), |row| {
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
    }
    ceiling
}

/// Returns the avg of the last three highest right now.
fn get_last_avg_max_min(season: i16, week: i8, field: &str, table: &str, pos: &Pos) -> (f32, f32) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let week_range_start: i8 = week - (4);
    let keepers: usize = 3;
    let players: Vec<Player> = get_all_active_players(pos, week);
    let mut max_avg: f32 = 0.0;
    let mut min_avg: f32 = 4000.0;
    let query: String = format!(
        "SELECT ({}) FROM {} WHERE season = ?1 AND week BETWEEN ?2 AND ?3 AND id = ?4",
        field, table
    );
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(query.as_str()).unwrap();
    for player in players {
        let mut result: Vec<f32> = stmt
            .query_map((season, week_range_start, week, player.id), |row| {
                row.get(0)
            })
            .unwrap()
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<f32>>();
        if result.len() >= keepers {
            result.sort_by(|a: &f32, b: &f32| b.partial_cmp(a).unwrap());
            let avg: f32 = &result[0..3].iter().sum::<f32>() / 3.0;
            if avg > max_avg {
                max_avg = avg
            } else if avg < min_avg {
                min_avg = avg
            }
        }
    }
    (max_avg, min_avg)
}

fn get_all_active_players(pos: &Pos, week: i8) -> Vec<Player> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let query = "SELECT * FROM ownership WHERE pos = ?1 AND week = ?2";
    let mut stmt = conn.prepare(query).unwrap();
    stmt.query_map(params![pos.to_str().unwrap(), week], |row| {
        Ok(Player {
            id: row.get(0).unwrap(),
            name: row.get(3).unwrap(),
            team: row.get(4).unwrap(),
            pos: Pos::from_string(row.get(6).unwrap()).unwrap(),
        })
    })
    .unwrap()
    .into_iter()
    .map(|p| p.unwrap())
    .collect::<Vec<Player>>()
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
        for player in get_all_active_players(&Pos::Qb, 18) {
            println!("{:?}", player)
        }
    }

    #[test]
    fn get_avg() {
        get_last_avg_max_min(2022, 18, "att", "qb_stats", &Pos::Qb);
    }
    #[test]
    fn get_stat_celing_test() {
        println!(
            "{:?}",
            get_recent_stat_ceiling_all(2022, 18, "fan_pts", "qb_stats", &Pos::Qb)
        );
    }

    #[test]
    fn test_max_min_own_per() {
        println!("{:?}", get_max_min_ownership(2023, 1));
    }
}
