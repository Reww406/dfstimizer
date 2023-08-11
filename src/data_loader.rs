use crate::{player::*, DATABASE_FILE};
use csv::{Reader, StringRecord};
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, str::Split};

lazy_static! {
    pub static ref NON_OFF_TO_OFF_ABBR: HashMap<&'static str, &'static str> = HashMap::from([
        ("ARI", "ARZ"),
        ("BAL", "BLT"),
        ("CLE", "CLV"),
        ("HOU", "HST"),
        ("JAC", "JAX"),
        ("LAR", "LA")
    ]);
}

// There is more fields we can grab if needed
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjRecord {
    fantasy_points_rank: i32,
    player_name: String,
    team_name: String,
    position: String,
    // Actually opponent
    #[serde(rename = "games")]
    opp: String,
    fantasy_points: f32,
    salary: i32,
    pass_comp: f32,
    pass_att: f32,
    pass_yds: f32,
    pass_td: f32,
    pass_int: f32,
    pass_sacked: f32,
    rush_att: f32,
    rush_yds: f32,
    rush_td: f32,
    recv_yds: f32,
    recv_targets: f32,
    recv_receptions: f32,
    recv_td: f32,
    fumbles: f32,
    fumbles_lost: f32,
    two_pt: f32,
    return_yds: f32,
    return_td: f32,
    pat_made: f32,
    pat_att: f32,
    dst_sacks: f32,
}

struct IdAndOwnership {
    id: i32,
    own_per: f32,
}
pub fn load_in_proj(path: &str, season: i16, week: i8) {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    for res in rdr.deserialize() {
        let rec: ProjRecord = res.unwrap();
        let pos: Result<Pos, ()> = Pos::from_str(&rec.position);
        match pos {
            Ok(Pos::Qb) => store_qb_proj(&rec, season, week, &conn),
            Ok(Pos::D) => store_dst_proj(&rec, season, week, &conn),
            Ok(Pos::Rb) => store_rb_proj(&rec, season, week, &conn),
            Ok(Pos::Te) => store_rec_proj(&rec, season, week, Pos::Te, &conn),
            Ok(Pos::Wr) => store_rec_proj(&rec, season, week, Pos::Wr, &conn),
            Err(_) => println!("Pos missing {:?}", pos),
        }
    }
}

fn get_id_ownership(
    name: &String,
    team: &String,
    pos: &Pos,
    week: i8,
    season: i16,
    conn: &Connection,
) -> Option<IdAndOwnership> {
    let id: i32 = get_player_id_create_if_missing(name, team, pos, conn);
    let own_per: Option<f32> = query_own_per(id, week, season, conn);
    if own_per.is_none() {
        println!("Player has no ownership: {}", name);
        return None;
    }
    Some(IdAndOwnership {
        id: id,
        own_per: own_per.unwrap(),
    })
}

fn store_qb_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let id_and_ownership: Option<IdAndOwnership> = get_id_ownership(
        &rec.player_name,
        &rec.team_name,
        &Pos::Qb,
        week,
        season,
        conn,
    );
    if id_and_ownership.is_none() {
        return;
    }
    let id_and_ownership: IdAndOwnership = id_and_ownership.unwrap();
    let qb_in: &str =
        "INSERT INTO qb_proj (id, season, week, name, team, opp, pts, atts, comps, ints, pass_yds, 
        pass_tds, rush_yds, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)";
    conn.execute(
        qb_in,
        (
            id_and_ownership.id,
            season,
            week,
            &rec.player_name,
            &rec.team_name,
            &rec.opp,
            rec.fantasy_points,
            rec.pass_att,
            rec.pass_comp,
            rec.pass_int,
            rec.pass_yds,
            rec.pass_td,
            rec.rush_yds,
            rec.salary,
            id_and_ownership.own_per,
        ),
    )
    .expect("Failed to insert Quater Back into database");
}

fn store_rb_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let id_and_ownership: Option<IdAndOwnership> = get_id_ownership(
        &rec.player_name,
        &rec.team_name,
        &Pos::Rb,
        week,
        season,
        conn,
    );
    if id_and_ownership.is_none() {
        return;
    }
    let id_and_ownership: IdAndOwnership = id_and_ownership.unwrap();
    let rb_in: &str =
        "INSERT INTO rb_proj (id, season, week, name, team, opp, pts, atts, tds, rec_yds,
        rush_yds, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,?13)";

    conn.execute(
        rb_in,
        (
            id_and_ownership.id,
            season,
            week,
            &rec.player_name,
            &rec.team_name,
            &rec.opp,
            rec.fantasy_points,
            rec.rush_att,
            rec.rush_td,
            rec.recv_yds,
            rec.rush_yds,
            rec.salary,
            id_and_ownership.own_per,
        ),
    )
    .expect("Failed to insert Rb into database");
}

fn store_rec_proj(rec: &ProjRecord, season: i16, week: i8, pos: Pos, conn: &Connection) {
    let id_and_ownership: Option<IdAndOwnership> =
        get_id_ownership(&rec.player_name, &rec.team_name, &pos, week, season, conn);
    if id_and_ownership.is_none() {
        return;
    }
    let id_and_ownership: IdAndOwnership = id_and_ownership.unwrap();
    let table: &str = if pos == Pos::Wr { "wr_proj" } else { "te_proj" };
    let rec_in: String = format!(
        "INSERT INTO {} (id, season, week, name, team, opp, pts, rec, tgts, tds, 
        rec_yds, rush_yds, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        table
    );
    conn.execute(
        &rec_in,
        (
            id_and_ownership.id,
            season,
            week,
            &rec.player_name,
            &rec.team_name,
            &rec.opp,
            rec.fantasy_points,
            rec.recv_receptions,
            rec.recv_targets,
            rec.recv_td,
            rec.recv_yds,
            rec.rush_yds,
            rec.salary,
            id_and_ownership.own_per,
        ),
    )
    .expect("Failed to insert Wide Reciever into database");
}

fn store_dst_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let id_and_ownership: Option<IdAndOwnership> = get_id_ownership(
        &rec.player_name,
        &rec.team_name,
        &Pos::D,
        week,
        season,
        conn,
    );
    if id_and_ownership.is_none() {
        return;
    }
    let id_and_ownership: IdAndOwnership = id_and_ownership.unwrap();
    let dst_in: &str = "INSERT INTO dst_proj (id, season, week, name, team, opp, pts,
        salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
    conn.execute(
        dst_in,
        (
            id_and_ownership.id,
            season,
            week,
            &rec.player_name,
            &rec.team_name,
            &rec.opp,
            rec.fantasy_points,
            rec.salary,
            id_and_ownership.own_per,
        ),
    )
    .expect("Failed to insert Defense into database");
}

// Could change ownership to serde?
fn create_ownership(
    rec: StringRecord,
    season: i16,
    week: i8,
    conn: &Connection,
) -> Option<Ownership> {
    let name = rec[1].to_string();
    let team = rec[2].to_string();
    let pos = Pos::from_str(&rec[4].to_string()).expect("Couldnt get pos");
    let id: i32 = get_player_id_create_if_missing(&name, &team, &pos, conn);
    Some(Ownership {
        id: id as i16,
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

// Load ownership stats
pub fn load_ownership_stats(path: &str, season: i16, week: i8) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let ownership_in: &str =
        "INSERT INTO ownership (id, season, week, name, team, opp, pos, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";
    let contents: String = fs::read_to_string(path).expect("Failed to read file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    for rec in rdr.records() {
        let record: StringRecord = rec.unwrap();
        let opt_ownership: Option<Ownership> = create_ownership(record, season, week, &conn);
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

fn get_player_from_record(record: ProjRecord, pos: Pos) -> Player {
    if pos == Pos::D {
        return Player {
            id: 0,
            name: record.player_name,
            team: record.team_name,
            pos: pos,
        };
    }
    Player {
        id: 0,
        name: record.player_name,
        team: record.team_name,
        pos: pos,
    }
}

// Create player Id Record
pub fn load_player_id(player: Player, conn: &Connection) -> i32 {
    let player_in: &str = "INSERT INTO player (name, team , pos) VALUES (?1, ?2, ?3)";
    let player_clone = player.clone();
    conn.execute(
        player_in,
        (
            player.name,
            player.team,
            player.pos.to_str().expect("Failed to convert Pos to Str"),
        ),
    )
    .expect("Failed to insert Player into database");
    return get_player_id(
        &player_clone.name,
        &player_clone.team,
        &player_clone.pos,
        conn,
    )
    .expect("Just loaded player but cannot find him.");
}

// Iterate through all projections and add player Ids if missing
pub fn load_all_player_ids(path: &str) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let contents: String = fs::read_to_string(path).expect("Failed to read file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    for rec in rdr.deserialize() {
        let record: ProjRecord = rec.unwrap();
        let pos: Pos = Pos::from_str(&record.position).unwrap();
        let player: Player = get_player_from_record(record, pos);
        if get_player_id(&player.name, &player.team, &pos, &conn).is_some() {
            // Player already exists
            continue;
        }
        load_player_id(player, &conn);
    }
}

#[cfg(test)]
mod tests {

    use num_bigint::ToBigUint;

    use super::*;

    #[test]
    fn test_get_player_id() {
        let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
        let id: i32 = get_player_id(
            &String::from("Isaiah Hodgins"),
            &String::from("NYG"),
            &Pos::Wr,
            &conn,
        )
        .unwrap();
        assert_eq!(id, 154)
    }
}
