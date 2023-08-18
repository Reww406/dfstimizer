use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::try_join;
use futures::Future;
use futures::StreamExt;
use lazy_static::__Deref;
use rusqlite::Connection;

use crate::gen_comb;
use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// AYU DARK
const GOOD_SALARY_USAGE: i32 = 45000;

// TODO Build all combinations of WR's than go through and generate 200 for each one.
// Combine all at the end and sort again.
pub fn build_all_possible_lineups(
    players: Vec<Arc<LitePlayer>>,
    // wr_lineup: Arc<LineupBuilder>,
    week: i8,
    season: i16,
) -> Vec<Lineup> {
    let mut pool_builder = ThreadPool::builder();
    pool_builder.pool_size(32);
    let pool = pool_builder.create().unwrap();
    let mut lineups: Vec<LineupBuilder> = Vec::new();
    let mut finished_lineups: Vec<Lineup> = Vec::new();
    players
        .iter()
        .filter(|player| player.pos == Pos::Qb)
        .for_each(|qb| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            lineups.push(lineup_builder.set_qb(qb.clone()))
        });
    // let players_ref = players.clone();
    let wrs_lineups: Vec<Arc<LineupBuilder>> = add_wrs_to_lineups(&players, lineups);
    let size = wrs_lineups.len();
    let mut current = 0;
    let mut futures: Vec<_> = Vec::new();
    for wr_lp in wrs_lineups {
        current += 1;
        println!("Started {} need {}", current, size);
        let (tx, rx) = mpsc::unbounded::<Lineup>();
        let binding = wr_lp.clone();
        let future = async {
            let player_clone = players.clone();

            let fut_tx_result = async move {
                // println!("Start thread.");
                // let start = SystemTime::now();
                let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(&player_clone, &binding);
                // println!("WR Lineup: {:?}", rbs_lineups);

                let te_linesups: Vec<LineupBuilder> = add_te_to_lineups(&player_clone, rbs_lineups);
                // println!("TE Lineups: ", te_linesups)
                let filterd_lineups = filter_low_salary_cap(te_linesups, 40000);
                let dst_lineups: Vec<LineupBuilder> =
                    add_dst_to_lineups(&player_clone, filterd_lineups);
                let lineups: Vec<Lineup> =
                    add_flex_find_top_num(&player_clone, dst_lineups, 20, week, season);
                lineups.iter().for_each(|l: &Lineup| {
                    tx.unbounded_send(l.clone()).expect("Failed to send lineup")
                });
                // let since_the_epoch = start
                //     .duration_since(UNIX_EPOCH)
                //     .expect("Time went backwards");
                // println!("Finished in {:?}ms", since_the_epoch.as_millis());
            };

            pool.spawn_ok(fut_tx_result);

            let future = rx.collect::<Vec<Lineup>>();
            future.await
        };
        futures.push(future);
        // finished_lineups.extend(result);
    }
    for future in futures {
        finished_lineups.extend(executor::block_on(future));
    }
    finished_lineups
}

pub fn filter_low_salary_cap(
    mut lineups: Vec<LineupBuilder>,
    filter_cap: i32,
) -> Vec<LineupBuilder> {
    lineups.retain(|l| l.total_price > filter_cap);
    lineups
}

pub fn get_pos_combos<'a>(
    players: &Vec<Arc<LitePlayer>>,
    slots: i8,
    positions: &[Pos],
) -> Vec<Vec<Arc<LitePlayer>>> {
    let pos: &Vec<Arc<LitePlayer>> = &players
        .into_iter()
        .filter(|p| positions.contains(&p.pos))
        .map(|p| p.clone())
        .collect::<Vec<Arc<LitePlayer>>>();
    gen_comb(
        pos,
        slots.try_into().expect("Passed a negative slots value"),
    )
}

// Needs to barrow players so it can be passed to the rest of the functions
pub fn add_wrs_to_lineups(
    players: &Vec<Arc<LitePlayer>>,
    lineups: Vec<LineupBuilder>,
) -> Vec<Arc<LineupBuilder>> {
    let mut new_lineups: Vec<Arc<LineupBuilder>> = Vec::new();
    let p_lookup: HashMap<i16, &Arc<LitePlayer>> = LitePlayer::player_lookup_map(players);
    let wr_combo: Vec<Vec<Arc<LitePlayer>>> = get_pos_combos(players, 3, &[Pos::Wr]);
    for lineup in &lineups {
        for combo in &wr_combo {
            //TODO WTF?!?! clone clone
            new_lineups.push(Arc::new(
                lineup
                    .clone()
                    .set_wr1(
                        p_lookup
                            .get(&combo[0].id)
                            .expect("Player missing")
                            .clone()
                            .clone(),
                    )
                    .set_wr2(
                        p_lookup
                            .get(&combo[1].id)
                            .expect("Missing Player")
                            .clone()
                            .clone(),
                    )
                    .set_wr3(
                        p_lookup
                            .get(&combo[2].id)
                            .expect("Missing Player")
                            .clone()
                            .clone(),
                    ),
            ))
        }
    }
    new_lineups
}

pub fn add_rbs_to_lineups(
    players: &Vec<Arc<LitePlayer>>,
    lineup: &LineupBuilder,
) -> Vec<LineupBuilder> {
    let mut new_lineups: Vec<LineupBuilder> = Vec::new();
    let p_lookup: HashMap<i16, &Arc<LitePlayer>> = LitePlayer::player_lookup_map(players);
    let rb_combos: Vec<Vec<Arc<LitePlayer>>> = get_pos_combos(players, 2, &[Pos::Rb]);

    for combo in &rb_combos {
        new_lineups.push(
            lineup
                .clone()
                .set_rb1(
                    p_lookup
                        .get(&combo[0].id)
                        .expect("Player missing")
                        .clone()
                        .clone(),
                )
                .set_rb2(
                    p_lookup
                        .get(&combo[1].id)
                        .expect("Player Missing")
                        .clone()
                        .clone(),
                ),
        );
    }

    new_lineups
}

pub fn add_te_to_lineups(
    players: &Vec<Arc<LitePlayer>>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineups_with_te: Vec<LineupBuilder> = Vec::with_capacity(lineups.len());
    for lineup in &lineups {
        players.iter().filter(|p| p.pos == Pos::Te).for_each(|te| {
            lineups_with_te.push(lineup.clone().set_te(te.clone()));
        })
    }
    lineups_with_te
}

// Pull in data per
pub fn add_flex_find_top_num(
    players: &Vec<Arc<LitePlayer>>,
    lineups: Vec<LineupBuilder>,
    lineup_cap: usize,
    week: i8,
    season: i16,
) -> Vec<Lineup> {
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let flex_pos: [Pos; 2] = [Pos::Wr, Pos::Rb];
    let mut best_lineups: Vec<Lineup> = Vec::with_capacity(lineup_cap);
    let mut lowest_score: f32 = 0.0;
    let mut sorted: bool = false;
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        let running_backs: [&i16; 2] = [
            &lineup.rb1.as_ref().unwrap().id,
            &lineup.rb2.as_ref().unwrap().id,
        ];
        let wide_recievers: [&i16; 3] = [
            &lineup.wr1.as_ref().unwrap().id,
            &lineup.wr2.as_ref().unwrap().id,
            &lineup.wr3.as_ref().unwrap().id,
        ];
        players
            .iter()
            .filter(|p| flex_pos.contains(&p.pos))
            .filter(|p| !running_backs.contains(&&p.id))
            .filter(|p| !wide_recievers.contains(&&p.id))
            .filter(|p| (p.salary as i32 + lineup.total_price) < SALARY_CAP)
            .filter(|p| (p.salary as i32 + lineup.total_price) > GOOD_SALARY_USAGE)
            .for_each(|flex| {
                iterations += 1;

                let finished_lineup = lineup.clone().set_flex(flex.clone());
                let score = calculate_lineup_score(&finished_lineup);
                if best_lineups.len() == lineup_cap && sorted == false {
                    best_lineups.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                    sorted = true;
                }
                if best_lineups.len() < lineup_cap {
                    if score < lowest_score {
                        lowest_score = score;
                    }
                    let lineup_res: Result<Lineup, Box<dyn Error>> =
                        finished_lineup.clone().build(week, season, &conn);
                    if lineup_res.is_err() {
                        println!("Missing Proj for Lineup, {:?}", lineup_res.err());
                        return;
                    }
                    best_lineups.push(lineup_res.unwrap());
                } else if score > lowest_score {
                    for i in 0..best_lineups.len() {
                        if score > best_lineups[i].score {
                            let lineup_res: Result<Lineup, Box<dyn Error>> =
                                finished_lineup.clone().build(week, season, &conn);
                            if lineup_res.is_err() {
                                println!("Missing Proj for Lineup, {:?}", lineup_res.err());
                                return;
                            }
                            best_lineups[i] = lineup_res.unwrap();
                            if i == (best_lineups.len() - 1) {
                                lowest_score = score;
                            }
                            break;
                        }
                    }
                }
            });
    }
    // println!("Flex iterated {} times", iterations);
    best_lineups
}

pub fn add_dst_to_lineups(
    players: &Vec<Arc<LitePlayer>>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineups_with_def: Vec<LineupBuilder> = Vec::new();
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        players.iter().filter(|p| p.pos == Pos::D).for_each(|def| {
            lineups_with_def.push(lineup.clone().set_def(def.clone()));
            iterations += 1;
        });
    }
    // println!("Def iterated: {} times", iterations);
    lineups_with_def
}

// TODO their will need to be mutiple line up scores
pub fn calculate_lineup_score(lineup: &LineupBuilder) -> f32 {
    let salary_spent_score: f32 = lineup.get_salary_spent_score();
    // let ownership_score: f32 = lineup.get_ownership_score();
    // ownership_score + salary_spent_score
    salary_spent_score
}

// TODO Test build lineup should be sorted and under salary cap.

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for creating line ups
    // fn create_test_lineup(ownership: f32, price: i16) -> Lineup {
    //     let mut players: Vec<LitePlayer> = Vec::with_capacity(9);
    //     for _ in 0..9 {
    //         players.push(LitePlayer {
    //             id: 1,
    //             salary: price,
    //             pos: Pos::Qb,
    //         })
    //     }

    //     let lineup: LineupBuilder = LineupBuilder {
    //         dst: Some(&players[0]),
    //         qb: Some(&players[1]),
    //         rb1: Some(&players[2]),
    //         rb2: Some(&players[3]),
    //         wr1: Some(&players[4]),
    //         wr2: Some(&players[5]),
    //         wr3: Some(&players[6]),
    //         te: Some(&players[7]),
    //         flex: Some(&players[8]),
    //         total_price: 60000,
    //     };
    //     lineup.build()
    // }

    // #[test]
    // fn test_max_score() {
    //     let lineup: Lineup = create_test_lineup(MIN_AVG_OWNERSHIP, 6666);
    //     assert_eq!(lineup.score, 1.0)
    // }

    // #[test]
    // // fn test_min_score() {
    //     let lineup: Lineup = create_test_lineup(MAX_AVG_OWNERHSIP, 0);
    //     assert_eq!(lineup.score, -1.0);
    // }

    // #[test]
    // fn test_scoring() {
    //     let lineup: Lineup = create_test_lineup(20.0, 4000);
    //     let lineup1: Lineup = create_test_lineup(19.8, 4005);
    //     assert!(lineup.score < lineup1.score);
    // }
}
