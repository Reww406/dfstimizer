use csv::Error;
use dfstimizer::load_in_fd_csv;
use dfstimizer::models::*;
use dfstimizer::optimizer::*;

// TODO Stacking should be scored
// TODO Points per Dollar
// TODO Opp Pos Rank
// TODO Bring in more stats from play dirt fantasy

fn main() -> Result<(), Error> {
    let players: Vec<Player> =
        load_in_fd_csv("fanduel.csv", &[String::from("PIT"), String::from("TEN")]);

    let lineups: Vec<Lineup> = build_all_possible_lineups(&players);
    for lineup in lineups {
        println!("{:?}", lineup.score);
    }
    Ok(())
}
