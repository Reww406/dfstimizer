use csv::Error;
use dfstimizer::data_loader::load_in_proj;
use dfstimizer::data_loader::load_in_qb_stats;
use dfstimizer::data_loader::load_in_rec_rush_stats;
use dfstimizer::data_loader::load_ownership_stats;
use dfstimizer::island_optimizer::*;
use dfstimizer::lineup::*;
use dfstimizer::load_in_ownership;
use dfstimizer::optimizer::*;
use dfstimizer::player::*;
use dfstimizer::tables::init_tables;
use dfstimizer::total_comb;
use itertools::Itertools;

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
    load_ownership_stats("week-1-ownership.csv", 2023, 1);
    load_in_proj("week-1-proj.csv", 2023, 1);
    for i in 14..19 {
        let qb_file: String = format!("qb-2022-{}-stats.csv", i);
        let rec_rush_file: String = format!("rush-rec-2022-{}.csv", i);
        load_in_qb_stats(&qb_file, 2022, i);
        load_in_rec_rush_stats(&rec_rush_file, 2022, i);
    }
}

fn main() -> Result<(), Error> {
    let start: Instant = Instant::now();
    // load_in_stats();
    let players: Vec<Arc<LitePlayer>> = load_in_ownership(
        "week-1-ownership.csv",
        1,
        2023,
        &[String::from("DET"), String::from("KC")],
    );
    let island_combos = total_comb(players.len(), 5);
    let qb: u32 = count_player_type(&players, Pos::Qb) as u32;
    let wr_count: u32 = count_player_type(&players, Pos::Wr) as u32;
    let wr: u32 = total_comb(wr_count.try_into().unwrap(), 3);
    let rb_count: u32 = count_player_type(&players, Pos::Rb) as u32;
    let rb: u32 = total_comb(rb_count.try_into().unwrap(), 2);
    let te: u32 = count_player_type(&players, Pos::Te) as u32;
    let d: u32 = count_player_type(&players, Pos::D) as u32;
    let flex: u32 = wr_count + rb_count;
    let total: u128 = qb as u128 * wr as u128 * rb as u128 * te as u128 * d as u128 * flex as u128;
    println!("Total Players: {}", players.len());

    println!("Totals: {} {} {} {} {} {}", qb, wr, rb, te, d, flex);
    let lineups: Vec<IslandLineup> = build_island_lineups(players.clone(), 1, 2023);
    println!("Total lineup count {}", lineups.len());
    println!("Elapsed: {:?}", start.elapsed());
    println!("Max Iterations: {}", total);

    for lineup in &lineups[0..2] {
        println!("{:?}", lineup)
    }

    println!("Max island iterations: {}", island_combos);

    Ok(())
}
