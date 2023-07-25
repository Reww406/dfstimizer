use crate::lineup::*;
use crate::player::*;

const GOOD_SALARY_USAGE: i32 = 45000;

// TODO try to refactor, hopefully can take up less space
// TODO Optimizer and Linup Builder maybe should be split into seperate classes
// TODO should do comparsions with player ID will make this faster
pub fn build_all_possible_lineups(players: &Vec<PlayerOwn>) -> Vec<Lineup> {
    let mut lineups: Vec<LineupBuilder> = Vec::new();
    players
        .iter()
        .filter(|player: &&PlayerOwn| player.pos.to_lowercase() == "qb")
        .for_each(|qb: &PlayerOwn| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            lineups.push(lineup_builder.set_qb(qb))
        });
    let wrs_lineups: Vec<LineupBuilder> = add_wrs_to_lineups(&players, lineups);
    let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(&players, wrs_lineups);
    let te_linesups: Vec<LineupBuilder> = add_te_to_lineups(&players, rbs_lineups);
    let dst_lineups: Vec<LineupBuilder> = add_dst_to_lineups(&players, te_linesups);
    let filterd_lineups = filter_low_salary_cap(dst_lineups, 48000);
    let flex_lineups: Vec<Lineup> = add_flex_find_top_num(&players, filterd_lineups, 300);
    flex_lineups
}

pub fn filter_low_salary_cap(
    mut lineups: Vec<LineupBuilder>,
    filter_cap: i32,
) -> Vec<LineupBuilder<'_>> {
    lineups.retain(|l| l.total_price > filter_cap);
    lineups
}

// Needs to barrow players so it can be passed to the rest of the functions
pub fn add_wrs_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut iterations: i64 = 0;
    // The two vectors should be dereferenced once the function ends
    let mut lineups_with_wr1: Vec<LineupBuilder> = Vec::with_capacity(lineups.len());
    for lineup in &lineups {
        players
            .iter()
            .filter(|player: &&PlayerOwn| player.pos.to_lowercase() == "wr")
            .for_each(|wr: &PlayerOwn| {
                lineups_with_wr1.push(lineup.clone().set_wr1(wr));
                iterations += 1
            });
    }

    let mut lineups_with_wr2: Vec<LineupBuilder> = Vec::with_capacity(lineups_with_wr1.len());
    for lineup in &lineups_with_wr1 {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos.to_lowercase() == "wr")
            .filter(|wr2: &&PlayerOwn| wr2.name != lineup.wr1.as_ref().unwrap().name)
            .for_each(|wr2: &PlayerOwn| {
                lineups_with_wr2.push(lineup.clone().set_wr2(wr2));
                iterations += 1;
            });
    }

    let mut lineup_with_wr3: Vec<LineupBuilder> = Vec::with_capacity(lineups_with_wr2.len());
    drop(lineups_with_wr1);
    for lineup in &lineups_with_wr2 {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos.to_lowercase() == "wr")
            .filter(|wr3: &&PlayerOwn| wr3.name != lineup.wr1.as_ref().unwrap().name)
            .filter(|wr3: &&PlayerOwn| wr3.name != lineup.wr2.as_ref().unwrap().name)
            .for_each(|wr3: &PlayerOwn| {
                lineup_with_wr3.push(lineup.clone().set_wr3(wr3));
                iterations += 1;
            });
    }
    println!("WR Iterated: {} times", iterations);
    drop(lineups_with_wr2);
    lineup_with_wr3
}

pub fn add_rbs_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut lineups_with_rb1: Vec<LineupBuilder> = Vec::with_capacity(lineups.len());
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos.to_lowercase() == "rb")
            .for_each(|rb: &PlayerOwn| {
                lineups_with_rb1.push(lineup.clone().set_rb1(rb));
                iterations += 1
            });
    }

    let mut lineups_with_rb2: Vec<LineupBuilder> = Vec::with_capacity(lineups_with_rb1.len());
    for lineup in &lineups_with_rb1 {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos.to_lowercase() == "rb")
            .filter(|p: &&PlayerOwn| p.name != lineup.rb1.unwrap().name)
            .for_each(|rb2: &PlayerOwn| {
                lineups_with_rb2.push(lineup.clone().set_rb2(rb2));
                iterations += 1;
            })
    }
    println!("RB Iterated: {} times", iterations);
    drop(lineups_with_rb1);
    lineups_with_rb2
}

pub fn add_te_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut lineups_with_te: Vec<LineupBuilder> = Vec::with_capacity(lineups.len());
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos.to_lowercase() == "te")
            .for_each(|te: &PlayerOwn| {
                lineups_with_te.push(lineup.clone().set_te(te));
                iterations += 1
            })
    }
    println!("TE Iterated: {} times", iterations);
    lineups_with_te
}

pub fn add_flex_find_top_num<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
    lineup_cap: usize,
) -> Vec<Lineup> {
    let flex_pos: [String; 2] = [String::from("wr"), String::from("rb")];
    let mut best_lineups: Vec<Lineup> = Vec::with_capacity(lineup_cap);
    let mut lowest_score: f32 = 0.0;
    let mut sorted: bool = false;
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        let running_backs: [&String; 2] = [&lineup.rb1.unwrap().name, &lineup.rb2.unwrap().name];
        let wide_recievers: [&String; 3] = [
            &lineup.wr1.unwrap().name,
            &lineup.wr2.unwrap().name,
            &lineup.wr3.unwrap().name,
        ];
        players
            .iter()
            .filter(|p: &&PlayerOwn| flex_pos.contains(&p.pos.to_lowercase()))
            .filter(|p: &&PlayerOwn| !running_backs.contains(&&p.name.to_lowercase()))
            .filter(|p: &&PlayerOwn| !wide_recievers.contains(&&p.name.to_lowercase()))
            .filter(|p: &&PlayerOwn| (p.salary as i32 + lineup.total_price) < SALARY_CAP)
            .filter(|p: &&PlayerOwn| (p.salary as i32 + lineup.total_price) > GOOD_SALARY_USAGE)
            .for_each(|flex: &PlayerOwn| {
                iterations += 1;
                let finished_lineup = lineup.clone().set_flex(flex);
                let score = calculate_lineup_score(&finished_lineup);
                if best_lineups.len() == lineup_cap && sorted == false {
                    best_lineups.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                    sorted = true;
                    for i in &best_lineups {
                        print!("{}, ", i.score)
                    }
                }
                if best_lineups.len() < lineup_cap {
                    if score < lowest_score {
                        lowest_score = score;
                    }
                    best_lineups.push(finished_lineup.clone().build());
                } else if score > lowest_score {
                    for i in 0..best_lineups.len() {
                        if score > best_lineups[i].score {
                            best_lineups[i] = finished_lineup.clone().build();
                            if i == (best_lineups.len() - 1) {
                                lowest_score = score;
                            }
                            break;
                        }
                    }
                }
            });
    }
    println!("Flex iterated {} times", iterations);
    best_lineups
}

pub fn add_dst_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut lineups_with_def: Vec<LineupBuilder> = Vec::new();
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos.to_lowercase() == "d")
            .for_each(|def: &PlayerOwn| {
                lineups_with_def.push(lineup.clone().set_def(def));
                iterations += 1;
            });
    }
    println!("Def iterated: {} times", iterations);
    lineups_with_def
}

// TODO their will need to be mutiple line up scores
pub fn calculate_lineup_score(lineup: &LineupBuilder) -> f32 {
    let salary_spent_score: f32 = lineup.get_salary_spent_score();
    let ownership_score: f32 = lineup.get_ownership_score();
    ownership_score + salary_spent_score
}

// TODO Test build lineup should be sorted and under salary cap.

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for creating line ups
    fn create_test_lineup(ownership: f32, price: i32) -> Lineup {
        let mut players: Vec<PlayerOwn> = Vec::with_capacity(9);
        for _ in 0..9 {
            players.push(PlayerOwn {
                id: 1,
                opp: String::from("PIT"),
                team: String::from("TEN"),
                name: String::from("John bob"),
                own_per: ownership,
                salary: price,
                pos: String::from("QB"),
            })
        }

        let lineup: LineupBuilder = LineupBuilder {
            dst: Some(&players[0]),
            qb: Some(&players[1]),
            rb1: Some(&players[2]),
            rb2: Some(&players[3]),
            wr1: Some(&players[4]),
            wr2: Some(&players[5]),
            wr3: Some(&players[6]),
            te: Some(&players[7]),
            flex: Some(&players[8]),
            total_price: 60000,
        };
        lineup.build()
    }

    #[test]
    fn test_max_score() {
        let lineup: Lineup = create_test_lineup(MIN_AVG_OWNERSHIP, 6666);
        assert_eq!(lineup.score, 1.0)
    }

    #[test]
    fn test_min_score() {
        let lineup: Lineup = create_test_lineup(MAX_AVG_OWNERHSIP, 0);
        assert_eq!(lineup.score, -1.0);
    }

    #[test]
    fn test_scoring() {
        let lineup: Lineup = create_test_lineup(20.0, 4000);
        let lineup1: Lineup = create_test_lineup(19.8, 4005);
        assert!(lineup.score < lineup1.score);
    }
}
