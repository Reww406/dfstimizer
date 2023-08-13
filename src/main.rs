use csv::Error;
use dfstimizer::data_loader::*;
use dfstimizer::gen_comb;
use dfstimizer::lineup::*;
use dfstimizer::load_in_ownership;
use dfstimizer::optimizer::*;
use dfstimizer::player::*;
use dfstimizer::total_comb;
use dfstimizer::DATABASE_FILE;
use num_bigint::BigUint;
use num_bigint::ToBigInt;
use num_bigint::ToBigUint;
use rusqlite::Connection;
use std::mem::size_of_val;

// TODO Stacking should be scored
// TODO Points per Dollar
// TODO Opp Pos Rank
// TODO Bring in more stats from play dirt fantasy
// TODO remove all negative correlations when building line ups page 57 of book
// TODO Stacking for turnaments
// TODO Get player consitensy numbers and pick the max ?
// TODO Load QB stats, WR stats, RB stats, TE stats and DST stats into Sqlite
// TODO Seperate table for Targets
// TODO load in rolling salary averge
// TODO calculate plus minus
// TODO less target seperation good for stacking

// TODO use Sqlite to avoid doing all iterations in memory

fn count_player_type(players: &Vec<LitePlayer>, pos: Pos) -> i32 {
    let mut count = 0;
    for player in players {
        if player.pos == pos {
            count += 1;
        }
    }
    count
}

fn init_tables() {
    let conn: Connection = Connection::open(DATABASE_FILE).expect("Can't open DB File");
    let player: &str = "
        CREATE TABLE IF NOT EXISTS player (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            pos TEXT NOT NULL,
            UNIQUE(name, team, pos) on CONFLICT REPLACE
        )
    ";

    let rb_proj: &str = "
        CREATE TABLE IF NOT EXISTS rb_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            atts REAL NOT NULL,
            tds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            rec_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let dst_proj: &str = "
        CREATE TABLE IF NOT EXISTS dst_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let qb_proj: &str = "
        CREATE TABLE IF NOT EXISTS qb_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            atts REAL NOT NULL,
            comps REAL NOT NULL,
            ints REAL NOT NULL,
            pass_yds REAL NOT NULL,
            pass_tds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let wr_proj: &str = "
        CREATE TABLE IF NOT EXISTS wr_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            rec REAL NOT NULL,
            tgts REAL NOT NULL,
            tds REAL NOT NULL,
            rec_yds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let te_proj: &str = "
        CREATE TABLE IF NOT EXISTS te_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            rec REAL NOT NULL,
            tgts REAL NOT NULL,
            tds REAL NOT NULL,
            rec_yds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    // id,player,team,opponent,position,salary,ownership
    let ownership: &str = "
        CREATE TABLE IF NOT EXISTS ownership (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pos REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";
    let tables: [&str; 7] = [
        player, qb_proj, wr_proj, dst_proj, te_proj, rb_proj, ownership,
    ];
    for table in tables {
        conn.execute(table, ()).expect("Could not create table");
    }
}
fn main() -> Result<(), Error> {
    let players: Vec<LitePlayer> = load_in_ownership(
        "fd-ownership.csv",
        &[
            // String::from("PIT"),
            String::from("CIN"),
            // String::from("TEN"),
            // String::from("DET"),
            // String::from("SEA"),
            // String::from("ATL"),
            // String::from("WAS"),
        ],
    );
    // We shouldn't be iterating over line ups like order matters this will reduce
    // lineup amount by a lot

    let qb = count_player_type(&players, Pos::Qb);
    let wr = count_player_type(&players, Pos::Wr);
    let rb = count_player_type(&players, Pos::Rb);
    let te = count_player_type(&players, Pos::Te);
    let d = count_player_type(&players, Pos::D);
    let flex = wr + rb;
    println!(
        "{} {} {} {} {} {}",
        total_comb(qb.try_into().unwrap(), 1),
        total_comb(wr.try_into().unwrap(), 3),
        total_comb(rb.try_into().unwrap(), 2),
        total_comb(te.try_into().unwrap(), 1),
        total_comb(d.try_into().unwrap(), 1),
        total_comb(flex.try_into().unwrap(), 1)
    );
    let lineups = build_all_possible_lineups(&players, 18, 2022);
    for lineup in lineups {
        println!("{:?}\n", lineup)
    }
    // println!("Total Line ups: {}", lineups.len());

    Ok(())
}
