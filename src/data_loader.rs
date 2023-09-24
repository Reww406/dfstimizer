use crate::{player::*, Day, DATABASE_FILE};

use lazy_static::lazy_static;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::{collections::HashMap, fs};

lazy_static! {
    pub static ref TEAM_NAME_TO_ABV: HashMap<&'static str, &'static str> = HashMap::from([
        ("Los Angeles Chargers", "LAC"),
        ("Philadelphia Eagles", "PHI"),
        ("Chicago Bears", "CHI"),
        ("Miami Dolphins", "MIA"),
        ("Jacksonville Jaguars", "JAX"),
        ("Cincinnati Bengals", "CIN"),
        ("Detroit Lions", "DET"),
        ("Indianapolis Colts", "IND"),
        ("Pittsburgh Steelers", "PIT"),
        ("Tampa Bay Buccaneers", "TB"),
        ("Minnesota Vikings", "MIN"),
        ("Arizona Cardinals", "ARI"),
        ("Las Vegas Raiders", "LV"),
        ("Denver Broncos", "DEN"),
        ("Tennessee Titans", "TEN"),
        ("Green Bay Packers", "GB"),
        ("Seattle Seahawks", "SEA"),
        ("Kansas City Chiefs", "KC"),
        ("New England Patriots", "NE"),
        ("Baltimore Ravens", "BAL"),
        ("San Francisco 49ers", "SF"),
        ("Los Angeles Rams", "LA"),
        ("New York Jets", "NYJ"),
        ("Buffalo Bills", "BUF"),
        ("Carolina Panthers", "CAR"),
        ("Atlanta Falcons", "ATL"),
        ("Houston Texans", "HOU"),
        ("New York Giants", "NYG"),
        ("Dallas Cowboys", "DAL"),
        ("Cleveland Browns", "CLE"),
        ("New Orleans Saints", "NO"),
        ("Washington Commanders", "WAS")
    ]);
}

// There is more fields we can grab if needed
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
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
    month_consistency: f32,
    month_upside: f32, // Load in month year stats when they exist...
}
// TODO get specific stats per pos
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecDefVsPos {
    team: String,
    #[serde(alias = "fdPtsPg")]
    pts_pg: Option<f32>,
}

/// Used for the data loader
fn query_def_id(team: &String, conn: &Connection) -> Result<i16, rusqlite::Error> {
    let select_player: &str = "SELECT id FROM player WHERE pos = 'D' AND team = ?1";
    conn.query_row(
        select_player,
        (TEAM_NAME_TO_ABV.get(team.as_str()).unwrap(),),
        |row| row.get(0),
    )
}

/// Load def vs pos stats into sqlite
pub fn load_in_def_vs_pos(path: &str, table: &str) {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut reader: csv::Reader<&[u8]> = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(contents.as_bytes());
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let insert: String = format!(
        "INSERT INTO {} (id, team_name, pts_given_pg) VALUES (?1, ?2, ?3)",
        table
    );
    for res in reader.deserialize() {
        let rec: RecDefVsPos = res.unwrap();
        let def_id: Result<i16, rusqlite::Error> = query_def_id(&rec.team, &conn);
        if def_id.is_err() || rec.pts_pg.is_none() {
            continue;
        }
        conn.execute(insert.as_str(), (def_id.unwrap(), rec.team, rec.pts_pg))
            .expect("Failed to insert D vs Pos");
    }
}

/// Load in proj for Sunday Slate
pub fn load_in_proj(path: &str, season: i16, week: i8, pos: &Pos, day: &Day) {
    let contents: String = fs::read_to_string(path).expect("Failed to read in file");
    let mut reader: csv::Reader<&[u8]> = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(contents.as_bytes());
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    for res in reader.deserialize() {
        let mut rec: ProjRecord = res.unwrap();
        rec.pos = Some(pos.to_str().unwrap().to_owned());
        match pos {
            Pos::Qb => store_qb_proj(&rec, season, week, day, &conn),
            Pos::D => store_dst_proj(&rec, season, week, day, &conn),
            Pos::Rb => store_rb_proj(&rec, season, week, day, &conn),
            Pos::Te => store_rec_proj(&rec, season, week, day, &conn),
            Pos::Wr => store_rec_proj(&rec, season, week, day, &conn),
            Pos::K => store_kick_proj(&rec, season, week, day, &conn),
        }
    }
}

/// Load in any flex projects for Monday, Thu
pub fn load_in_anyflex(path: &str, season: i16, week: i8, day: &Day) {
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
            Pos::Qb => store_qb_proj(&rec, season, week, day, &conn),
            Pos::D => store_dst_proj(&rec, season, week, day, &conn),
            Pos::Rb => store_rb_proj(&rec, season, week, day, &conn),
            Pos::Te => store_rec_proj(&rec, season, week, day, &conn),
            Pos::Wr => store_rec_proj(&rec, season, week, day, &conn),
            Pos::K => store_kick_proj(&rec, season, week, day, &conn),
        }
    }
}

fn store_qb_proj(rec: &ProjRecord, season: i16, week: i8, day: &Day, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i16 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week, day);
    let qb_in: &str =
        "INSERT INTO qb_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, pts_plus_minus_proj, 
            pts_sal_proj, vegas_total, avg_pass_atts, avg_pass_comps, avg_pass_yds, avg_pass_tds, avg_rush_atts,
            avg_long_pass_yds, pass_to_wr_per, pass_to_te_per, wind_speed, salary, own_proj, rating, red_zone_op_pg,
            vegas_team_total, month_consistency, yds_per_pass_att, day) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, 
                ?23, ?24, ?25, ?26, ?27, ?28, ?29)";
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
            rec.vegas_team_total,
            rec.month_consistency,
            rec.yds_per_pass_att,
            day.to_str()
        ],
    )
    .expect("Failed to insert Quarter Back into database");
}

fn store_rb_proj(rec: &ProjRecord, season: i16, week: i8, day: &Day, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos not set")).unwrap();
    let id: i16 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week, day);
    let rb_in: &str =
        "INSERT INTO rb_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, pts_plus_minus_proj,
            pts_sal_proj, vegas_total, rush_yds_share, avg_atts, avg_td, avg_rush_yds, avg_rec_tgts, salary, own_proj,
            rating, snaps_per, year_consistency, vegas_team_total, month_consistency, day) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, 
                ?22, ?23, ?24, ?25)";

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
            rec.avg_tgts,
            rec.salary,
            rec.own_proj,
            rec.rating,
            rec.snaps_share,
            rec.year_consistency,
            rec.vegas_team_total,
            rec.month_consistency,
            day.to_str()
        ],
    )
    .expect("Failed to insert Rb into database");
}

fn store_rec_proj(rec: &ProjRecord, season: i16, week: i8, day: &Day, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i16 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week, day);
    let table: &str = if pos == Pos::Wr { "wr_proj" } else { "te_proj" };
    let rec_in: String = format!(
        "INSERT INTO {} (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, pts_plus_minus_proj, 
            pts_sal_proj, vegas_total, avg_recp, avg_tgts, avg_td, avg_rec_yds, avg_rush_yds, red_zone_op_pg, 
            rec_tgt_share, salary, own_proj, rating, year_consistency, year_upside, vegas_team_total, 
            month_consistency, month_upside, day) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, 
                ?23, ?24, ?25, ?26, ?27, ?28)",
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
            rec.year_upside,
            rec.vegas_team_total,
            rec.month_consistency,
            rec.month_upside,
            day.to_str()
        ],
    )
    .expect("Failed to insert Wide Reciever into database");
}

fn store_kick_proj(rec: &ProjRecord, season: i16, week: i8, day: &Day, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i16 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week, day);
    let dst_in: &str = "INSERT INTO kick_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj,
         pts_plus_minus_proj, pts_sal_proj, vegas_total, salary, own_proj, rating, day) 
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
            day.to_str(),
        ),
    )
    .expect("Failed to insert Defense into database");
}

fn store_dst_proj(rec: &ProjRecord, season: i16, week: i8, day: &Day, conn: &Connection) {
    let pos: Pos = Pos::from_str(&rec.pos.as_ref().expect("Pos missing")).unwrap();
    let id: i16 = get_player_id_create_if_missing(&rec.player, &rec.team, &pos, conn);
    store_ownership(&rec, id, season, week, day);
    let dst_in: &str = "INSERT INTO dst_proj (id, season, week, name, team, opp, pts_proj, cieling_proj, floor_proj, 
        pts_plus_minus_proj, pts_sal_proj, vegas_total, salary, own_proj, rating, vegas_opp_total, day, 
        vegas_team_total) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)";
    conn.execute(
        dst_in,
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
            rec.salary,
            rec.own_proj,
            rec.rating,
            rec.vegas_opp_total,
            day.to_str(),
            rec.vegas_team_total
        ],
    )
    .expect("Failed to insert Defense into database");
}

/// Load ownership stats
pub fn store_ownership(rec: &ProjRecord, id: i16, season: i16, week: i8, day: &Day) {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let ownership_in: &str =
        "INSERT INTO ownership (id, season, week, day, name, team, opp, pos, salary, own_per) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

    conn.execute(
        ownership_in,
        (
            id,
            season,
            week,
            day.to_str(),
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

// Create player Id Record
pub fn load_player_id(player: &Player, conn: &Connection) -> i16 {
    let player_in: &str = "INSERT INTO player (name, team , pos) VALUES (?1, ?2, ?3)";
    conn.execute(
        player_in,
        (
            &player.name,
            &player.team,
            player.pos.to_str().expect("Failed to convert Pos to Str"),
        ),
    )
    .expect("Failed to insert Player into database");
    return get_player_id(&player.name, &player.team, &player.pos, conn)
        .expect("Just loaded player but cannot find him.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_player_id() {
        let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
        let id: i16 = get_player_id(
            &String::from("Isaiah Hodgins"),
            &String::from("NYG"),
            &Pos::Wr,
            &conn,
        )
        .unwrap();
        assert_eq!(id, 154)
    }
}
