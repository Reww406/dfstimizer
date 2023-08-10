use csv::{Reader, StringRecord};
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Params, Result};
use std::{collections::HashMap, fs, str::Split};

use crate::{
    player::{self, Ownership, Player, Pos, QbProj, RbProj, RecProj},
    DATABASE_FILE,
};

const REC_RUSH_POS: [Pos; 3] = [Pos::Wr, Pos::Te, Pos::Rb];

lazy_static! {
    static ref QB_PATH: Regex = Regex::new(r"^.*?-qb[.]csv$").unwrap();
    static ref D_PATH: Regex = Regex::new(r"^.*?-d[.]csv$").unwrap();
    static ref NON_OFF_TO_OFF_ABBR: HashMap<&'static str, &'static str> = HashMap::from([
        ("ARI", "ARZ"),
        ("BAL", "BLT"),
        ("CLE", "CLV"),
        ("HOU", "HST"),
        ("JAC", "JAX"),
        ("LAR", "LA")
    ]);
}

// Gonna be used to load data from FantasyData to
// Sqlite which will be used to enrich lineups...

// QB, Rec -> Projection table 1 to 1 of project csv
// Ownership and Salary table -> can look at historic salaries than
// QB, RB, Rec, Def Adv stats
// Player table with name team and ID

// "Player" "Team" "Fantasy Points Pts" "Fantasy Points x-Pts" "Fantasy Points +/-"
// "Points Per Game PPG","Points Per Game x-PPG","Points Per Game +/-","#G","Passing Att"
// "Passing Com","Passing X-Com","Passing Int","Passing X-Int","Passing Yds","Passing X-Yds"
// "Passing TD","Passing X-TD","Scrambles Yds","Scrambles X-Yds","Scrambles TD","Scrambles X-TD"
// "Designed Runs Yds","Designed Runs X-Yds","Designed Runs TD","Designed Runs X-TD"
fn get_qb_proj(rec: StringRecord, season: i16, week: i8) -> Option<QbProj> {
    None
}
// ID if player doesn't exist increment id
fn load_in_qb_proj(path: String) {
    let conn = Connection::open(DATABASE_FILE);
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());

    for record in rdr.records() {}
}

// "Player","Team","Pos","Fantasy Points Pts","Fantasy Points x-Pts",
// "Fantasy Points +/-","Points Per Game PPG","Points Per Game x-PPG",
// "Points Per Game +/-","#G","Receiving Rec","Receiving X-Rec","Receiving Yds",
// "Receiving X-Yds","Receiving TD","Receiving X-TD","Rushing Yds","Rushing X-Yds",
// "Rushing TD","Rushing X-TD"
fn load_in_rush_rec_proj(path: String) {}

fn get_rb_proj(record: &StringRecord, season: i16, week: i8) -> Option<RbProj> {
    None
}

fn get_rec_proj(record: &StringRecord, season: i16, week: i8) -> Option<RecProj> {
    None
}

fn load_player_info_rb_wr(path: String) {
    let conn = Connection::open("./dfs_nfl.db3");
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());

    for r in rdr.records() {}
}

// "team","games","sacks","fumbleRecoveries","interceptions","defenseTouchdowns",
// "safeties","blockedKicks","kickYards","kickTouchdowns","puntYards","puntTouchdowns"
// ,"paPerGame","pa0Games","pa16Games","pa713Games","pa1420Games","pa2127Games",
// "pa2834Games","pa3545Games","pa46plusGames","passYdsPerGame","runYdsPerGame",
// "yaNegGames","ya099Games","ya100199Games","ya200299Games","ya300349Games",
// "ya350399Games","ya400449Games","ya450499Games","ya500549Games","ya550plusGames",
// "fantasyPts","fantasyPpg"

fn load_player_info_dst(path: String) {}

fn get_ownership(rec: StringRecord, season: i16, week: i8) -> Option<Ownership> {
    let name = rec[1].to_string();
    let team = rec[2].to_string();
    let pos = Pos::from_str(&rec[4].to_string()).expect("Couldnt get pos");
    let id: Option<i32> = get_player_id(&name, &team, &pos);
    if id.is_none() {
        println!("Could not find player if for {} {} {:?}", name, team, pos);
        return None;
    }

    Some(Ownership {
        id: id.unwrap() as i16,
        season,
        week,
        name,
        team,
        opp: rec[3].to_string(),
        pos: pos.to_str().unwrap().to_string(),
        salary: rec[5].parse::<i32>().expect("Salary is not a number"),
        own_per: rec[6].parse::<f32>().expect("Could not parse ownership"),
    })
}

pub fn load_ownership_stats(path: &str, season: i16, week: i8) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let ownership_in: &str =
        "INSERT INTO ownership (id, season, week, name, team, opp, pos, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
    let contents: String = fs::read_to_string(path).expect("Failed to read file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    for rec in rdr.records() {
        let record: StringRecord = rec.unwrap();
        let opt_ownership: Option<Ownership> = get_ownership(record, season, week);
        if opt_ownership.is_none() {
            continue;
        }
        let ownership = opt_ownership.unwrap();
        conn.execute(
            ownership_in,
            (
                ownership.id,
                season,
                week,
                ownership.name,
                ownership.team,
                ownership.opp,
                ownership.pos,
                ownership.salary,
                ownership.own_per,
            ),
        )
        .expect("Failed to insert Ownership into database");
    }
}

fn get_player_pos(path: &str, record: &StringRecord) -> Pos {
    if QB_PATH.is_match(path) {
        return Pos::Qb;
    }
    if D_PATH.is_match(path) {
        return Pos::D;
    }
    Pos::from_str(&record[3].to_string()).expect("Position is missing.")
}

// name, team, pos
fn get_player_from_record(record: &StringRecord, pos: Pos) -> Player {
    if pos == Pos::D {
        return Player {
            id: 0,
            name: record[0].to_string(),
            team: record[0].to_string(),
            pos: pos,
        };
    }
    Player {
        id: 0,
        name: record[0].to_string(),
        team: record[1].to_string(),
        pos: pos,
    }
}

// Players
pub fn load_all_player_ids(stats: [&str; 3]) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let player_in: &str = "INSERT INTO player (name, team , pos) VALUES (?1, ?2, ?3)";
    stats.iter().for_each(|p: &&str| {
        let contents: String = fs::read_to_string(p).expect("Failed to read file");
        let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
        for rec in rdr.records() {
            let record: StringRecord = rec.unwrap();
            let pos: Pos = get_player_pos(p, &record);
            let player: Player = get_player_from_record(&record, pos);
            if get_player_id(&player.name, &player.team, &pos).is_some() {
                // Player already exists
                continue;
            }
            conn.execute(
                player_in,
                (
                    player.name,
                    player.team,
                    player.pos.to_str().expect("Failed to convert Pos to Str"),
                ),
            )
            .expect("Failed to insert Player into database");
        }
    })
}

pub fn get_player_id(name: &String, team: &String, pos: &Pos) -> Option<i32> {
    // IF Pos = D use team
    // Fix Team names
    let mut correct_team: &str = team;
    let mut correct_name: &str = name;
    let team_conversion = NON_OFF_TO_OFF_ABBR.get(&team[..]);
    if team_conversion.is_some() {
        correct_team = team_conversion.unwrap();
    }

    if pos == &Pos::D {
        correct_name = correct_team
    }

    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let select_player = "SELECT id FROM player WHERE name = ?1 AND team = ?2 and pos = ?3";
    let id: Option<i32> = conn
        .query_row(
            select_player,
            (correct_name, correct_team, pos.to_str().unwrap()),
            |row| row.get(0),
        )
        .optional()
        .unwrap();
    if id.is_some() {
        return id;
    }

    // No hit on exact match
    let fuzzy_select = "SELECT id FROM player WHERE name LIKE ?1 AND team = ?2 and pos = ?3";
    let mut name_split: Split<'_, &str> = name.trim().split(" ");
    let first_name: &str = name_split.next().unwrap();
    let last_name: &str = name_split.next().unwrap();
    let fuzzy_name: String = first_name.chars().nth(0).unwrap().to_string() + "%" + last_name + "%";
    println!("Missed exact search, searching for {}", fuzzy_name);

    let id: Option<i32> = conn
        .query_row(
            fuzzy_select,
            (fuzzy_name, correct_team, pos.to_str().unwrap()),
            |row| row.get(0),
        )
        .optional()
        .unwrap();
    return id;
}

#[cfg(test)]
mod tests {

    use num_bigint::ToBigUint;

    use super::*;

    #[test]
    fn test_path_regex() {
        assert!(QB_PATH.is_match("all-qb.csv"));
        assert!(D_PATH.is_match("all-d.csv"));
    }

    #[test]
    fn test_get_player_id() {
        let id: i32 = get_player_id(
            &String::from("Isaiah Hodgins"),
            &String::from("NYG"),
            &Pos::Wr,
        )
        .unwrap();
        assert_eq!(id, 154)
    }
}
