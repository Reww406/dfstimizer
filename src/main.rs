use csv::Error;
use dfstimizer::data_loader::load_in_anyflex;
use dfstimizer::data_loader::load_in_def_vs_pos;
use dfstimizer::data_loader::load_in_proj;
use dfstimizer::get_slate;
use dfstimizer::lineup::*;
use dfstimizer::optimizer::*;
use dfstimizer::player::*;
use dfstimizer::tables::init_tables;
use dfstimizer::total_comb;
use dfstimizer::Day;
use dfstimizer::DATABASE_FILE;
use dfstimizer::GAME_DAY;
use dfstimizer::SEASON;
use dfstimizer::WEEK;
use rusqlite::Connection;

use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

fn count_player_type(players: &Vec<LitePlayer>, pos: Pos) -> i32 {
    let mut count: i32 = 0;
    for player in players {
        if player.pos == pos {
            count += 1;
        }
    }
    count
}

fn load_in_stats() {
    init_tables();
    load_in_anyflex("flex/sun-night-2.csv", 2023, 22, &Day::Sun);
    load_in_anyflex("flex/flex-1.csv", 2023, 1, &Day::Thu);
    load_in_anyflex("flex/flex-3-thu.csv", 2023, 3, &Day::Thu);
    load_in_anyflex("flex/monday-1.csv", 2023, 1, &Day::Mon);
    load_in_proj("sun-proj/d-1.csv", 2023, 1, &Pos::D, &Day::Sun);
    load_in_proj("sun-proj/qb-1.csv", 2023, 1, &Pos::Qb, &Day::Sun);
    load_in_proj("sun-proj/rb-1.csv", 2023, 1, &Pos::Rb, &Day::Sun);
    load_in_proj("sun-proj/te-1.csv", 2023, 1, &Pos::Te, &Day::Sun);
    load_in_proj("sun-proj/wr-1.csv", 2023, 1, &Pos::Wr, &Day::Sun);
    load_in_proj("sun-proj/d-2-mon.csv", 2023, 2, &Pos::D, &Day::Mon);
    load_in_proj("sun-proj/qb-2-mon.csv", 2023, 2, &Pos::Qb, &Day::Mon);
    load_in_proj("sun-proj/rb-2-mon.csv", 2023, 2, &Pos::Rb, &Day::Mon);
    load_in_proj("sun-proj/te-2-mon.csv", 2023, 2, &Pos::Te, &Day::Mon);
    load_in_proj("sun-proj/wr-2-mon.csv", 2023, 2, &Pos::Wr, &Day::Mon);
    load_in_proj("sun-proj/d-2.csv", 2023, 2, &Pos::D, &Day::Sun);
    load_in_proj("sun-proj/qb-2.csv", 2023, 2, &Pos::Qb, &Day::Sun);
    load_in_proj("sun-proj/rb-2.csv", 2023, 2, &Pos::Rb, &Day::Sun);
    load_in_proj("sun-proj/te-2.csv", 2023, 2, &Pos::Te, &Day::Sun);
    load_in_proj("sun-proj/wr-2.csv", 2023, 2, &Pos::Wr, &Day::Sun);
    load_in_proj("sun-proj/d-3.csv", 2023, 3, &Pos::D, &Day::Sun);
    load_in_proj("sun-proj/qb-3.csv", 2023, 3, &Pos::Qb, &Day::Sun);
    load_in_proj("sun-proj/rb-3.csv", 2023, 3, &Pos::Rb, &Day::Sun);
    load_in_proj("sun-proj/te-3.csv", 2023, 3, &Pos::Te, &Day::Sun);
    load_in_proj("sun-proj/wr-3.csv", 2023, 3, &Pos::Wr, &Day::Sun);
    load_in_def_vs_pos("def/def-vs-qb.csv", "def_vs_qb");
    load_in_def_vs_pos("def/def-vs-rb.csv", "def_vs_rb");
    load_in_def_vs_pos("def/def-vs-te.csv", "def_vs_te");
    load_in_def_vs_pos("def/def-vs-wr.csv", "def_vs_wr");
}

fn parse_lineups(lineups: Vec<Lineup>, conn: &Connection) -> Option<Vec<Lineup>> {
    let mut qb_lineups: HashMap<i16, Vec<Lineup>> = HashMap::new();
    let mut best_lines: Vec<Lineup> = Vec::new();
    let amount_of_qb_per = 5;
    lineups.into_iter().for_each(|l| {
        let qb_id: i16 = l.qb.id;
        if qb_lineups.get(&qb_id).is_some() {
            if !qb_lineups.get(&qb_id).unwrap().contains(&l) {
                qb_lineups.get_mut(&qb_id).unwrap().push(l.clone());
            }
        } else {
            qb_lineups.insert(qb_id, vec![l]);
        }
    });
    for k in qb_lineups.keys() {
        let lu = qb_lineups.get(k).expect("");
        let mut clone_lu = lu.clone();
        clone_lu.sort_by(|a, b: &Lineup| b.score(&conn).partial_cmp(&a.score(&conn)).unwrap());
        let max_index = min(clone_lu.len(), amount_of_qb_per);
        clone_lu[0..max_index]
            .iter()
            .for_each(|l| best_lines.push(l.clone()));
    }
    best_lines.sort_by(|a, b: &Lineup| b.score(&conn).partial_cmp(&a.score(&conn)).unwrap());
    Some(best_lines)
}

// keep conn for ease of swapping
fn parse_island_lineups(lineups: Vec<IslandLineup>, _: &Connection) -> Option<Vec<IslandLineup>> {
    let mut qb_lineups: HashMap<i16, Vec<IslandLineup>> = HashMap::new();
    let mut best_lines: Vec<IslandLineup> = Vec::new();
    let amount_of_qb_per = 15;
    lineups.into_iter().for_each(|l| {
        let qb_id: i16 = l.mvp.get_proj_id();
        if qb_lineups.get(&qb_id).is_some() {
            if !qb_lineups.get(&qb_id).unwrap().contains(&l) {
                qb_lineups.get_mut(&qb_id).unwrap().push(l.clone());
            }
        } else {
            qb_lineups.insert(qb_id, vec![l]);
        }
    });
    for k in qb_lineups.keys() {
        let lu = qb_lineups.get(k).expect("");
        let mut clone_lu = lu.clone();
        clone_lu
            .sort_by(|a: &IslandLineup, b: &IslandLineup| b.score.partial_cmp(&a.score).unwrap());
        let max_index = min(clone_lu.len(), amount_of_qb_per);
        clone_lu[0..max_index]
            .iter()
            .for_each(|l| best_lines.push(l.clone()));
    }
    best_lines.sort_by(|a: &IslandLineup, b: &IslandLineup| b.score.partial_cmp(&a.score).unwrap());
    Some(best_lines)
}

fn main() -> Result<(), Error> {
    let start: Instant = Instant::now();
    let conn = Connection::open(DATABASE_FILE).unwrap();
    // load_in_stats();
    let players: Vec<LitePlayer> = get_slate(WEEK, SEASON, &GAME_DAY, true, &conn);
    let qb: u32 = count_player_type(&players, Pos::Qb) as u32;
    let wr_count: u32 = count_player_type(&players, Pos::Wr) as u32;
    let wr: u32 = total_comb(wr_count.try_into().unwrap(), 3);
    let rb_count: u32 = count_player_type(&players, Pos::Rb) as u32;
    let rb: u32 = total_comb(rb_count.try_into().unwrap(), 2);
    let te: u32 = count_player_type(&players, Pos::Te) as u32;
    let d: u32 = count_player_type(&players, Pos::D) as u32;
    let flex: u32 = wr_count + rb_count;
    let total: u128 = qb as u128 * wr as u128 * rb as u128 * te as u128 * d as u128 * flex as u128;
    // println!("Total Players: {}", players.len());
    println!("Max Iterations: {}", total);
    println!("WR Combos: {}", total_comb(wr_count as usize, 3));
    // TODO load in def for sunday
    let lineups: Vec<Lineup> = build_all_possible_lineups(WEEK, SEASON);
    // let lineups: Vec<IslandLineup> = build_island_lineups(WEEK, SEASON);

    let mut file = File::create(format!(
        "lineups/lineups-{}-{}.txt",
        WEEK,
        &GAME_DAY.to_str()
    ))
    .unwrap();

    for lineup in parse_lineups(lineups, &conn).unwrap() {
        file.write_all(lineup.lineup_str(&conn).as_bytes())?;
    }

    println!("Elapsed Time: {:?}", start.elapsed());
    Ok(())
}
