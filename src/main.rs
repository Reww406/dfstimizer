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
use std::sync::Arc;
use std::time::Instant;

// TODO Stacking should be scored
// TODO Points per Dollar
// TODO Opp Pos Rank
// TODO remove all negative correlations when building line ups page 57 of book
// TODO Stacking for turnaments
// TODO Get player consitensy numbers and pick the max ?
// TODO Load QB stats, WR stats, RB stats, TE stats and DST stats into Sqlite
// TODO Seperate table for Targets
// TODO load in rolling salary averge
// TODO calculate plus minus
// TODO less target seperation good for stacking

// TODO use Sqlite to avoid doing all iterations in memory

fn count_player_type(players: &Vec<Arc<LitePlayer>>, pos: Pos) -> i32 {
    let mut count: i32 = 0;
    for player in players {
        if player.pos == pos {
            count += 1;
        }
    }
    count
}

fn main() -> Result<(), Error> {
    let start: Instant = Instant::now();
    let players: Vec<Arc<LitePlayer>> = load_in_ownership(
        "fd-ownership.csv",
        18,
        2022,
        &[
            // String::from("*"),
            String::from("PIT"),
            String::from("CIN"),
            String::from("TEN"),
            String::from("DET"),
            String::from("SEA"),
            // String::from("ATL"),
            // String::from("WAS"),
            // String::from("SF"),
        ],
    );
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
    println!("Max Iterations: {}", total);

    println!("Totals: {} {} {} {} {} {}", qb, wr, rb, te, d, flex);
    let lineups: Vec<Lineup> = build_all_possible_lineups(players.clone(), 18, 2022);
    println!("Total lineup count {}", lineups.len());
    println!("Elapsed: {:?}", start.elapsed());

    Ok(())
}
