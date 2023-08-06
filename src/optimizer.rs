use crate::gen_comb;
use crate::lineup::*;
use crate::player::*;
use num_bigint::BigInt;
use num_bigint::BigUint;
use num_bigint::ToBigInt;
use num_bigint::ToBigUint;
use std::collections::HashMap;
use std::mem::size_of_val;
use std::ops::BitAnd;

// AYU DARK
const GOOD_SALARY_USAGE: i32 = 45000;

// TODO try to refactor, hopefully can take up less space
// TODO Optimizer and Linup Builder maybe should be split into seperate classes
// TODO should do comparsions with player ID will make this faster
pub fn build_all_possible_lineups(players: &Vec<PlayerOwn>) -> Vec<Lineup> {
    let mut lineups: Vec<LineupBuilder> = Vec::new();
    players
        .iter()
        .filter(|player: &&PlayerOwn| player.pos == Pos::Qb)
        .for_each(|qb: &PlayerOwn| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            lineups.push(lineup_builder.set_qb(qb))
        });
    let wrs_lineups: Vec<LineupBuilder> = add_wrs_to_lineups(&players, lineups);
    let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(&players, wrs_lineups);
    let te_linesups: Vec<LineupBuilder> = add_te_to_lineups(&players, rbs_lineups);
    let dst_lineups: Vec<LineupBuilder> = add_dst_to_lineups(&players, te_linesups);
    let filterd_lineups = filter_low_salary_cap(dst_lineups, 0);
    let flex_lineups: Vec<Lineup> = add_flex_find_top_num(&players, filterd_lineups, 250);
    flex_lineups
}

pub fn filter_low_salary_cap(
    mut lineups: Vec<LineupBuilder>,
    filter_cap: i32,
) -> Vec<LineupBuilder<'_>> {
    lineups.retain(|l| l.total_price > filter_cap);
    lineups
}

pub fn get_pos_combos<'a>(
    players: &Vec<PlayerOwn>,
    slots: i8,
    positions: &[Pos],
) -> Vec<Vec<PlayerOwn>> {
    let pos: &Vec<PlayerOwn> = &players
        .into_iter()
        .filter(|p| positions.contains(&p.pos))
        .map(|p| p.clone())
        .collect::<Vec<PlayerOwn>>();
    gen_comb(
        pos,
        slots.try_into().expect("Passed a negative slots value"),
    )
}

// Needs to barrow players so it can be passed to the rest of the functions
pub fn add_wrs_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut new_lineups: Vec<LineupBuilder> = Vec::new();
    let p_lookup: HashMap<i16, &PlayerOwn> = PlayerOwn::player_lookup_map(players);
    let wr_combo: Vec<Vec<PlayerOwn>> = get_pos_combos(players, 3, &[Pos::Wr]);
    for lineup in &lineups {
        for combo in &wr_combo {
            new_lineups.push(
                lineup
                    .clone()
                    .set_wr1(p_lookup.get(&combo[0].id).expect("Player missing"))
                    .set_wr2(p_lookup.get(&combo[1].id).expect("Missing Player"))
                    .set_wr3(p_lookup.get(&combo[2].id).expect("Missing Player")),
            )
        }
    }
    new_lineups
}

pub fn add_rbs_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut new_lineups: Vec<LineupBuilder> = Vec::new();

    let p_lookup: HashMap<i16, &PlayerOwn> = PlayerOwn::player_lookup_map(players);
    let rb_combos: Vec<Vec<PlayerOwn>> = get_pos_combos(players, 2, &[Pos::Rb]);
    for lineup in &lineups {
        for combo in &rb_combos {
            new_lineups.push(
                lineup
                    .clone()
                    .set_rb1(p_lookup.get(&combo[0].id).expect("Player missing"))
                    .set_rb2(p_lookup.get(&combo[1].id).expect("Player Missing")),
            );
        }
    }
    new_lineups
}

pub fn add_te_to_lineups<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
) -> Vec<LineupBuilder<'a>> {
    let mut lineups_with_te: Vec<LineupBuilder> = Vec::with_capacity(lineups.len());
    for lineup in &lineups {
        players
            .iter()
            .filter(|p: &&PlayerOwn| p.pos == Pos::Te)
            .for_each(|te: &PlayerOwn| {
                lineups_with_te.push(lineup.clone().set_te(te));
            })
    }
    lineups_with_te
}

// Pull in data per
pub fn add_flex_find_top_num<'a>(
    players: &'a Vec<PlayerOwn>,
    lineups: Vec<LineupBuilder<'a>>,
    lineup_cap: usize,
) -> Vec<Lineup> {
    let flex_pos: [Pos; 2] = [Pos::Wr, Pos::Rb];
    let mut best_lineups: Vec<Lineup> = Vec::with_capacity(lineup_cap);
    let mut lowest_score: f32 = 0.0;
    let mut sorted: bool = false;
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        let running_backs: [&i16; 2] = [&lineup.rb1.unwrap().id, &lineup.rb2.unwrap().id];
        let wide_recievers: [&i16; 3] = [
            &lineup.wr1.unwrap().id,
            &lineup.wr2.unwrap().id,
            &lineup.wr3.unwrap().id,
        ];
        players
            .iter()
            .filter(|p: &&PlayerOwn| flex_pos.contains(&p.pos))
            .filter(|p: &&PlayerOwn| !running_backs.contains(&&p.id))
            .filter(|p: &&PlayerOwn| !wide_recievers.contains(&&p.id))
            .filter(|p: &&PlayerOwn| (p.salary as i32 + lineup.total_price) < SALARY_CAP)
            .filter(|p: &&PlayerOwn| (p.salary as i32 + lineup.total_price) > GOOD_SALARY_USAGE)
            .for_each(|flex: &PlayerOwn| {
                iterations += 1;

                let finished_lineup = lineup.clone().set_flex(flex);
                let score = calculate_lineup_score(&finished_lineup);
                if best_lineups.len() == lineup_cap && sorted == false {
                    best_lineups.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                    sorted = true;
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
            .filter(|p: &&PlayerOwn| p.pos == Pos::D)
            .for_each(|def: &PlayerOwn| {
                lineups_with_def.push(lineup.clone().set_def(def));
                iterations += 1;
                if iterations > 100_000_000 {
                    println!("Hit 100M stoppping...");
                    std::process::exit(1)
                }
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
                opp_id: 1,
                team_id: 1,
                id: 1,
                own_per: ownership,
                salary: price,
                pos: Pos::Qb,
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
