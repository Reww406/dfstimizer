use csv::Error;
use dfstimizer::load_in_csv;
use dfstimizer::load_in_csv_buff_test;
use dfstimizer::models::*;
use dfstimizer::optimizer::*;

//TODO need to get standard deviation and mean for price, points and ownership
// these values can be used to do Zscore normalization
// need to figure out how to use 'a
// once we can normalize each variable using zscore we can calculate a score
// in testing 0 ownership, max salary and point should be the higest
// high ownerhsip, 0 salary and points should be the lowest

fn main() -> Result<(), Error> {
    // 4 * 4080 * 56 * 2 * 8
    let players: Vec<Player> = load_in_csv_buff_test("with_ownership.csv");
    // println!("{:?}", players);
    // println!(
    //     "{}",
    //     players
    //         .iter()
    //         .filter(|x| x.pos.to_lowercase() == "te")
    //         .count()
    // );

    let lineups: Vec<Lineup> = build_all_possible_lineups(&players);
    for lineup in lineups {
        println!("{:?}", lineup.score);
    }
    //TODO add score to lineup
    Ok(())
}
