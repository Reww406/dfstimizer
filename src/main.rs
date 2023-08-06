use csv::Error;
use dfstimizer::gen_comb;
use dfstimizer::lineup::*;
use dfstimizer::load_in_ownership;
use dfstimizer::optimizer::*;
use dfstimizer::player::*;
use dfstimizer::total_comb;
use num_bigint::BigUint;
use num_bigint::ToBigInt;
use num_bigint::ToBigUint;
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

fn count_player_type(players: &Vec<PlayerOwn>, pos: Pos) -> i32 {
    let mut count = 0;
    for player in players {
        if player.pos == pos {
            count += 1;
        }
    }
    count
}

fn main() -> Result<(), Error> {
    let players: Vec<PlayerOwn> = load_in_ownership(
        "fd-ownership.csv",
        &[
            String::from("PIT"),
            String::from("CIN"),
            String::from("TEN"),
            String::from("DET"),
            String::from("SEA"),
            String::from("ATL"),
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
    let lineups = build_all_possible_lineups(&players);
    println!("Total Line ups: {}", lineups.len());
    Ok(())
}
