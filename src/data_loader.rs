use crate::{player::*, DATABASE_FILE};

use lazy_static::lazy_static;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::{collections::HashMap, fs};

lazy_static! {
    pub static ref NON_OFF_TO_OFF_ABBR: HashMap<&'static str, &'static str> = HashMap::from([
        // ("ARI", "ARZ"),
        // ("BAL", "BLT"),
        // ("CLE", "CLV"),
        // ("HOU", "HST"),
        // ("JAC", "JAX"),
        // ("LAR", "LA")
    ]);
}

// There is more fields we can grab if needed
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjRecord {
    rating: f32,
    player: String,
    pos: Option<String>,
    salary: i16,
    team: String,
    opp: String,
    /// Exposure
    exposure_min: f32,
    exposure_max: f32,
    pts_proj: f32,
    ceiling_proj: f32,
    floor_proj: f32,
    pts_plus_minus_proj: f32,
    pts_sal_proj: f32,
    own_proj: f32,
    imp_pts_proj: f32,
    vegas_team_total: f32,
    vegas_opp_total: f32,
    vegas_spread: f32,
    vegas_total: f32,
    #[serde(default)]
    rec_tgt_share: f32,
    #[serde(default)]
    rec_td_tgt_share: f32,
    #[serde(default)]
    rec_yds_share: f32,
    #[serde(default)]
    rush_td_share: f32,
    #[serde(default)]
    rush_yds_share: f32,
    #[serde(default)]
    snaps_share: f32,
    #[serde(default)]
    avg_pass_comp: f32,
    #[serde(default)]
    avg_pass_atts: f32,
    #[serde(default)]
    avg_pass_yds: f32,
    // pass_comp_per: f32,
    #[serde(default)]
    yds_per_pass_att: f32,
    #[serde(default)]
    proj_rush_yds: f32,
    #[serde(default)]
    avg_pass_td: f32,
    #[serde(default)]
    avg_long_pass_yds: f32,
    #[serde(default)]
    pass_to_rb_per: f32,
    #[serde(default)]
    pass_to_wr_per: f32,
    #[serde(default)]
    pass_to_te_per: f32,
    #[serde(default)]
    avg_rush_att: f32,
    #[serde(default)]
    yds_per_carry: f32,
    #[serde(default)]
    avg_rush_yds: f32,
    #[serde(default)]
    avg_rush_td: f32,
    #[serde(default)]
    avg_tgts: f32,
    #[serde(default)]
    avg_recp: f32,
    #[serde(default)]
    avg_rec_yds: f32,
    #[serde(default)]
    avg_long_rec_yds: f32,
    #[serde(default)]
    yds_per_rec: f32,
    #[serde(default)]
    avg_rec_td: f32,
    #[serde(default)]
    yds_per_tgt: f32,
    #[serde(default)]
    red_zone_opp_pg: f32,
    tempature: f32,
    wind_speed: f32,
    precip_per: f32,
    year_consistency: f32,
    year_upside: f32,
    // Load in month year stats when they exist...
}

pub fn load_in_proj(path: &str, season: i16, week: i8, pos: &Pos) {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut reader: csv::Reader<&[u8]> = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(contents.as_bytes());
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    for res in reader.deserialize() {
        let mut rec: ProjRecord = res.unwrap();
        // let pos: Result<Pos, ()> = Pos::from_str(&rec.pos);
        rec.pos = Some(pos.to_str().unwrap().to_owned());
        match pos {
            Pos::Qb => store_qb_proj(&rec, season, week, &conn),
            Pos::D => store_dst_proj(&rec, season, week, &conn),
            Pos::Rb => store_rb_proj(&rec, season, week, &conn),
            Pos::Te => store_rec_proj(&rec, season, week, Pos::Te, &conn),
            Pos::Wr => store_rec_proj(&rec, season, week, Pos::Wr, &conn),
            Pos::K => store_kick_proj(&rec, season, week, &conn),
            _ => println!("Pos missing {:?}", pos),
        }
    }
}

pub fn load_in_anyflex(path: &str, season: i16, week: i8) {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut reader: csv::Reader<&[u8]> = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(contents.as_bytes());
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    for res in reader.deserialize() {
        let rec: ProjRecord = res.unwrap();
        let pos: Pos = Pos::from_string_ref(rec.pos.as_ref().unwrap()).unwrap();
        match pos {
            Pos::Qb => store_qb_proj(&rec, season, week, &conn),
            Pos::D => store_dst_proj(&rec, season, week, &conn),
            Pos::Rb => store_rb_proj(&rec, season, week, &conn),
            Pos::Te => store_rec_proj(&rec, season, week, Pos::Te, &conn),
            Pos::Wr => store_rec_proj(&rec, season, week, Pos::Wr, &conn),
            Pos::K => store_kick_proj(&rec, season, week, &conn),
            _ => println!("Pos missing"),
        }
    }
}

// TODO This should use the player object so changes are more obvious
fn store_qb_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i32 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week);
    let qb_in: &str =
        "INSERT INTO qb_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, pts_plus_minus_proj, 
            pts_sal_proj, vegas_total, avg_pass_atts, avg_pass_comps, avg_pass_yds, avg_pass_tds, avg_rush_atts,
            avg_long_pass_yds, pass_to_wr_per, pass_to_te_per, wind_speed, salary, own_proj, rating, red_zone_op_pg) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, 
                ?23, ?24, ?25)";
    conn.execute(
        qb_in,
        params![
            id,
            season,
            week,
            &rec.player,
            &rec.team,
            &rec.opp,
            rec.pts_proj,
            rec.ceiling_proj,
            rec.floor_proj,
            rec.pts_plus_minus_proj,
            rec.pts_sal_proj,
            rec.vegas_total,
            rec.avg_pass_atts,
            rec.avg_pass_comp,
            rec.avg_pass_yds,
            rec.avg_pass_td,
            rec.avg_rush_att,
            rec.avg_long_pass_yds,
            rec.pass_to_wr_per,
            rec.pass_to_te_per,
            rec.wind_speed,
            rec.salary,
            rec.own_proj,
            rec.rating,
            rec.red_zone_opp_pg,
        ],
    )
    .expect("Failed to insert Quarter Back into database");
}

fn store_rb_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos not set")).unwrap();
    let id: i32 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week);
    let rb_in: &str =
        "INSERT INTO rb_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, pts_plus_minus_proj,
            pts_sal_proj, vegas_total, rush_yds_share, avg_atts, avg_td, avg_rush_yds, avg_rec_yds, salary, own_proj,
            rating, snaps_per, year_consistency) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)";

    conn.execute(
        rb_in,
        params![
            id,
            season,
            week,
            &rec.player,
            &rec.team,
            &rec.opp,
            rec.pts_proj,
            rec.ceiling_proj,
            rec.floor_proj,
            rec.pts_plus_minus_proj,
            rec.pts_sal_proj,
            rec.vegas_total,
            rec.rush_yds_share,
            rec.avg_rush_att,
            rec.avg_rush_td,
            rec.avg_rush_yds,
            rec.avg_rec_yds,
            rec.salary,
            rec.own_proj,
            rec.rating,
            rec.snaps_share,
            rec.year_consistency
        ],
    )
    .expect("Failed to insert Rb into database");
}

fn store_rec_proj(rec: &ProjRecord, season: i16, week: i8, pos: Pos, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i32 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week);
    let table: &str = if pos == Pos::Wr { "wr_proj" } else { "te_proj" };
    let rec_in: String = format!(
        "INSERT INTO {} (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, pts_plus_minus_proj, 
            pts_sal_proj, vegas_total, avg_recp, avg_tgts, avg_td, avg_rec_yds, avg_rush_yds, red_zone_op_pg, 
            rec_tgt_share, salary, own_proj, rating, year_consistency, year_upside) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, 
                ?23, ?24)",
        table
    );
    conn.execute(
        &rec_in,
        params![
            id,
            season,
            week,
            &rec.player,
            &rec.team,
            &rec.opp,
            rec.pts_proj,
            rec.ceiling_proj,
            rec.floor_proj,
            rec.pts_plus_minus_proj,
            rec.pts_sal_proj,
            rec.vegas_total,
            rec.avg_recp,
            rec.avg_tgts,
            rec.avg_rec_td,
            rec.avg_rec_yds,
            rec.avg_rush_yds,
            rec.red_zone_opp_pg,
            rec.rec_tgt_share,
            rec.salary,
            rec.own_proj,
            rec.rating,
            rec.year_consistency,
            rec.year_upside
        ],
    )
    .expect("Failed to insert Wide Reciever into database");
}

fn store_kick_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i32 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week);
    let dst_in: &str = "INSERT INTO kick_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj,
         pts_plus_minus_proj, pts_sal_proj, vegas_total, salary, own_proj, rating) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)";
    conn.execute(
        dst_in,
        (
            id,
            season,
            week,
            &rec.player,
            &rec.team,
            &rec.opp,
            rec.pts_proj,
            rec.ceiling_proj,
            rec.floor_proj,
            rec.pts_plus_minus_proj,
            rec.pts_sal_proj,
            rec.vegas_total,
            rec.salary,
            rec.own_proj,
            rec.rating,
        ),
    )
    .expect("Failed to insert Defense into database");
}

fn store_dst_proj(rec: &ProjRecord, season: i16, week: i8, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i32 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week);
    let dst_in: &str = "INSERT INTO dst_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, 
        pts_plus_minus_proj, pts_sal_proj, vegas_total, salary, own_proj, rating, vegas_opp_total) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)";
    conn.execute(
        dst_in,
        (
            id,
            season,
            week,
            &rec.player,
            &rec.team,
            &rec.opp,
            rec.pts_proj,
            rec.ceiling_proj,
            rec.floor_proj,
            rec.pts_plus_minus_proj,
            rec.pts_sal_proj,
            rec.vegas_total,
            rec.salary,
            rec.own_proj,
            rec.rating,
            rec.vegas_opp_total,
        ),
    )
    .expect("Failed to insert Defense into database");
}

// Load ownership stats
pub fn store_ownership(rec: &ProjRecord, id: i32, season: i16, week: i8) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let ownership_in: &str =
        "INSERT INTO ownership (id, season, week, name, team, opp, pos, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";

    conn.execute(
        ownership_in,
        (
            id,
            season,
            week,
            &rec.player,
            &rec.team,
            &rec.opp,
            &rec.pos,
            rec.salary,
            rec.own_proj,
        ),
    )
    .expect("Failed to insert Ownership into database");
}

fn get_player_from_record(record: ProjRecord, pos: Pos) -> Player {
    if pos == Pos::D {
        return Player {
            id: 0,
            name: record.player,
            team: record.team,
            pos: pos,
        };
    }
    Player {
        id: 0,
        name: record.player,
        team: record.team,
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
        let pos: Pos = Pos::from_str(&record.pos.as_ref().expect("Pos missing")).unwrap();
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
