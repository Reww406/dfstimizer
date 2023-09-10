use csv::Error;
use dfstimizer::data_loader::load_in_proj;
use dfstimizer::data_loader::store_ownership;
use dfstimizer::get_all_active_players;
use dfstimizer::get_all_active_players_pos;
use dfstimizer::get_players_by_ids;
use dfstimizer::get_top_players;
use dfstimizer::island_optimizer::*;
use dfstimizer::lineup::*;
use dfstimizer::load_in_ownership;
use dfstimizer::optimizer::*;
use dfstimizer::player::*;
use dfstimizer::tables::init_tables;
use dfstimizer::total_comb;
use dfstimizer::D_COUNT;
use dfstimizer::QB_COUNT;
use dfstimizer::RB_COUNT;
use dfstimizer::TE_COUNT;
use dfstimizer::WR_COUNT;
use itertools::Itertools;

use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

// TODO Points per Dollar
// TODO Opp Pos Rank
// TODO Load QB stats, WR stats, RB stats, TE stats
// TODO load in rolling salary averge to cache?
// TODO calculate plus minus
// TODO less target seperation good for stacking
// TODO Look at premium stats on pff

// TODO Def Vs Pos https://www.pro-football-reference.com/years/2022/fantasy-points-against-RB.htm

fn count_player_type(players: &Vec<Arc<LitePlayer>>, pos: Pos) -> i32 {
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
    load_in_proj("d-1.csv", 2023, 1, &Pos::D);
    load_in_proj("qb-1.csv", 2023, 1, &Pos::Qb);
    load_in_proj("rb-1.csv", 2023, 1, &Pos::Rb);
    load_in_proj("te-1.csv", 2023, 1, &Pos::Te);
    load_in_proj("wr-1.csv", 2023, 1, &Pos::Wr);
}

fn get_sunday_slate() -> Vec<Arc<LitePlayer>> {
    let mut players = Vec::new();
    let top_qb = get_top_players(2023, 1, "qb_proj", QB_COUNT);
    let top_rb = get_top_players(2023, 1, "rb_proj", RB_COUNT);
    let top_wr = get_top_players(2023, 1, "wr_proj", WR_COUNT);
    let top_te = get_top_players(2023, 1, "te_proj", TE_COUNT);
    let top_d = get_top_players(2023, 1, "dst_proj", D_COUNT);
    let top_ids = [top_qb, top_rb, top_d, top_te, top_wr];
    for ids in top_ids {
        players.extend(get_players_by_ids(1, &ids))
    }
    players
}

fn main() -> Result<(), Error> {
    let start: Instant = Instant::now();
    // load_in_stats();
    // let players: Vec<Arc<LitePlayer>> = get_all_active_players(1);
    let players = get_sunday_slate();
    // for play in players {
    //     println!("{:?}", play)
    // }

    // let island_combos = total_comb(players.len(), 5);
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
    // println!("Totals: {} {} {} {} {} {}", qb, wr, rb, te, d, flex);
    let lineups: Vec<Lineup> = build_all_possible_lineups(players.clone(), 1, 2023);
    // println!("Total lineup count {}", lineups.len());
    println!("Elapsed: {:?}", start.elapsed());
    // for lineup in &lineups[0..10] {
    //     print!("{}, {} ", lineup.score, lineup.salary_used);
    //     print!("MVP: ");
    //     lineup.mvp.print_name();
    //     lineup.first.print_name();
    //     lineup.second.print_name();
    //     lineup.third.print_name();
    //     lineup.fourth.print_name();
    //     println!("")
    // }
    let mut file = File::create("Lineups.txt").unwrap();
    for lineup in &lineups[0..120] {
        file.write_all(lineup.lineup_str().as_bytes())?;
    }

    // println!("Max island iterations: {}", island_combos);

    Ok(())
}
