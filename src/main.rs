use csv::Error;
use dfstimizer::lineup::*;
use dfstimizer::load_in_fd_csv;
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

fn main() -> Result<(), Error> {
    let players: Vec<Player> = load_in_fd_csv(
        "fanduel.csv",
        &[String::from("PIT"), String::from("TEN")],
        1,
    );

    println!(
        "{:?}",
        players
            .into_iter()
            .filter(|p| p.pos == "QB")
            .collect::<Vec<Player>>()
    );

    // let lineups: Vec<Lineup> = build_all_possible_lineups(&players);
    // for lineup in lineups {
    //     println!("{:?}", lineup.score);
    // }
    Ok(())
}
