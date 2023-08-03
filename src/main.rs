use csv::Error;
use dfstimizer::lineup::*;
use dfstimizer::load_in_ownership;
use dfstimizer::optimizer::*;
use dfstimizer::player::*;

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
            String::from("*"),
            // String::from("CLE"),
            // String::from("BUF"),
            // String::from("DET"),
        ],
    );
    // We shouldn't be iterating over line ups like order matters this will reduce
    // lineup amount by a lot
    println!("QB {}", count_player_type(&players, Pos::Qb));
    println!("WR {}", count_player_type(&players, Pos::Wr));
    println!("RB {}", count_player_type(&players, Pos::Rb));
    println!("TE {}", count_player_type(&players, Pos::Te));
    println!("D {}", count_player_type(&players, Pos::D));
    // let lineups: Vec<Lineup> = build_all_possible_lineups(&players);
    // println!("{}", lineups.len());
    // for lineup in lineups {
    //     println!("{:?}", lineup.score);
    // }
    Ok(())
}
