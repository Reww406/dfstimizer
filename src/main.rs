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
    let players: Vec<Player> = load_in_csv_buff_test("with_ownership.csv");
    println!(
        "{}",
        players
            .iter()
            .filter(|x| x.pos.to_lowercase() == "qb")
            .count()
    );
    println!(
        "{}",
        players
            .iter()
            .filter(|x| x.pos.to_lowercase() == "wr")
            .count()
    );
    let lineups: Vec<LineupBuilder> = build_all_possible_lineups(players);
    println!("{}", lineups.len());
    Ok(())
}
