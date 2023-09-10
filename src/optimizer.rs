use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::future::join_all;
use futures::StreamExt;
use itertools::Itertools;
use rusqlite::Connection;

use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use std::collections::HashMap;
use std::sync::Arc;

// AYU DARK
const GOOD_SALARY_USAGE: i32 = 56000;

pub fn build_all_possible_lineups(
    players: Vec<Arc<LitePlayer>>,
    week: i8,
    season: i16,
) -> Vec<Lineup> {
    let mut pool_b = ThreadPool::builder();
    pool_b.pool_size(16);
    let pool = pool_b.create().unwrap();
    let mut qb_lineups: Vec<LineupBuilder> = Vec::new();
    let mut finished_lineups: Vec<Lineup> = Vec::new();
    players
        .iter()
        .filter(|player| player.pos == Pos::Qb)
        .for_each(|qb| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            qb_lineups.push(lineup_builder.set_pos(&qb, Slot::None))
        });

    let wrs_lineups: Vec<LineupBuilder> = add_wrs_to_lineups(&players, qb_lineups);
    println!("Cooking up LINEUPS!!");
    let mut futures: Vec<_> = Vec::new();
    for wr_lp in wrs_lineups {
        let (tx, rx) = mpsc::unbounded::<Lineup>();
        let binding = wr_lp.clone();
        let future = async {
            let player_clone = players.clone();
            let fut_tx_result = async move {
                let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(&player_clone, &binding);
                let te_lineups: Vec<LineupBuilder> = add_te_to_lineups(&player_clone, rbs_lineups);
                let dst_lineups: Vec<LineupBuilder> = add_dst_to_lineups(&player_clone, te_lineups);
                let filterd_lineups = filter_low_salary_cap(dst_lineups, 46500);
                let no_bad_combinations = filter_bad_lineups(filterd_lineups, week, season);
                let lineups: Vec<Lineup> =
                    add_flex_find_top_num(&player_clone, no_bad_combinations, 1, week, season);
                lineups.iter().for_each(|l: &Lineup| {
                    tx.unbounded_send(l.clone()).expect("Failed to send lineup")
                });
            };

            pool.spawn_ok(fut_tx_result);

            let future = rx.collect::<Vec<Lineup>>();
            future.await
        };
        futures.push(future);
    }
    let futures_join = join_all(futures);
    let test = executor::block_on(futures_join);
    for future in test {
        finished_lineups.extend(future);
    }
    finished_lineups.sort_by(|a, b: &Lineup| b.score().partial_cmp(&a.score()).unwrap());
    finished_lineups
}

pub fn filter_low_salary_cap(
    mut lineups: Vec<LineupBuilder>,
    filter_cap: i32,
) -> Vec<LineupBuilder> {
    lineups.retain(|l| l.salary_used > filter_cap);
    lineups
}

pub fn filter_bad_lineups(
    lineups: Vec<LineupBuilder>,
    week: i8,
    season: i16,
) -> Vec<LineupBuilder> {
    let conn = Connection::open(DATABASE_FILE).unwrap();
    let mut filtered_lineups: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups {
        let qb = query_qb_proj_helper(&lineup.qb, week, season, &conn);
        let d = query_def_proj_helper(&lineup.dst, week, season, &conn);
        let rb1 = query_rb_proj_helper(&lineup.rb1, week, season, &conn);
        let rb2 = query_rb_proj_helper(&lineup.rb2, week, season, &conn);
        if qb.opp == d.team {
            continue;
        }
        if rb1.opp == rb2.team {
            continue;
        }
        filtered_lineups.push(lineup)
    }
    filtered_lineups
}
// Needs to barrow players so it can be passed to the rest of the functions
pub fn add_wrs_to_lineups(
    players: &Vec<Arc<LitePlayer>>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut new_lineups: Vec<LineupBuilder> = Vec::new();
    let p_lookup: HashMap<i16, &Arc<LitePlayer>> = LitePlayer::player_lookup_map(players);
    for lineup in &lineups {
        for combo in players.iter().filter(|p| p.pos == Pos::Wr).combinations(3) {
            new_lineups.push(
                lineup
                    .clone()
                    .set_pos(
                        *p_lookup.get(&combo[0].id).expect("Player missing"),
                        Slot::First,
                    )
                    .set_pos(
                        *p_lookup.get(&combo[1].id).expect("Missing Player"),
                        Slot::Second,
                    )
                    .set_pos(
                        *p_lookup.get(&combo[2].id).expect("Missing Player"),
                        Slot::Third,
                    ),
            )
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
    for combo in players.iter().filter(|p| p.pos == Pos::Rb).combinations(2) {
        new_lineups.push(
            lineup
                .clone()
                .set_pos(
                    *p_lookup.get(&combo[0].id).expect("Player missing"),
                    Slot::First,
                )
                .set_pos(
                    *p_lookup.get(&combo[1].id).expect("Player Missing"),
                    Slot::Second,
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
            lineups_with_te.push(lineup.clone().set_pos(&te, Slot::None));
        })
    }
    lineups_with_te
}

// WR2 And QB are most correlated, than 3 than 2
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
            .filter(|p| (p.salary as i32 + lineup.salary_used) < SALARY_CAP)
            .filter(|p| (p.salary as i32 + lineup.salary_used) > MIN_SAL)
            .for_each(|flex| {
                // should be refactored to a function
                iterations += 1;
                let finished_lineup = lineup
                    .clone()
                    .set_pos(&flex, Slot::Flex)
                    .build(week, season, &conn)
                    .expect("Failed to build lineup..");
                let score: f32 = finished_lineup.score();
                if best_lineups.len() == lineup_cap && sorted == false {
                    best_lineups
                        .sort_by(|a, b: &Lineup| b.score().partial_cmp(&a.score()).unwrap());
                    sorted = true;
                }
                if best_lineups.len() < lineup_cap {
                    if score < lowest_score {
                        lowest_score = score;
                    }
                    best_lineups.push(finished_lineup);
                } else if score > lowest_score {
                    for i in 0..best_lineups.len() {
                        if score > best_lineups[i].score() {
                            best_lineups[i] = finished_lineup;
                            if i == (best_lineups.len() - 1) {
                                lowest_score = score;
                            }
                            break;
                        }
                    }
                }
            });
    }
    best_lineups
}

pub fn insert_value_sorted(lineups: &mut Vec<Lineup>, new_lineup: Lineup) -> (&Vec<Lineup>, bool) {
    let pos = lineups
        .binary_search_by(|probe| new_lineup.score().total_cmp(&probe.score()))
        .expect("Failed to find value");
    lineups[pos] = new_lineup;
    if pos == (lineups.len() - 1) {
        return (lineups, true);
    }
    (lineups, false)
}

pub fn add_dst_to_lineups(
    players: &Vec<Arc<LitePlayer>>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineups_with_def: Vec<LineupBuilder> = Vec::new();
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        players.iter().filter(|p| p.pos == Pos::D).for_each(|def| {
            lineups_with_def.push(lineup.clone().set_pos(&def, Slot::None));
            iterations += 1;
        });
    }
    lineups_with_def
}

// TODO Test build lineup should be sorted and under salary cap.
#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    // Helper function for creating line ups
    fn create_test_lineup(price: i32) -> Lineup {
        let conn: Connection = Connection::open(DATABASE_FILE).expect("Failed to open DB");
        let week = 18;
        let season = 2022;
        Lineup {
            qb: query_qb_proj(26, week, season, &conn).unwrap(),
            rb1: query_rb_proj(1, week, season, &conn).unwrap(),
            rb2: query_rb_proj(2, week, season, &conn).unwrap(),
            wr1: query_rec_proj(3, week, season, &Pos::Wr, &conn).unwrap(),
            wr2: query_rec_proj(8, week, season, &Pos::Wr, &conn).unwrap(),
            wr3: query_rec_proj(8, week, season, &Pos::Wr, &conn).unwrap(),
            te: query_rec_proj(56, week, season, &Pos::Te, &conn).unwrap(),
            flex: FlexProj {
                rb_proj: Some(query_rb_proj(2, week, season, &conn).unwrap()),
                pos: Pos::Rb,
                rec_proj: None,
            },
            def: query_def_proj(17, week, season, &conn).unwrap(),
            salary_used: price,
        }
    }

    // #[test]
    // fn test_insert_sorted_val() {
    //     let mut lineups: Vec<Lineup> = vec![
    //         create_test_lineup(1030),
    //         create_test_lineup(1030),
    //         create_test_lineup(1050),
    //     ];
    //     lineups
    //         .iter()
    //         .for_each(|l| println!("Current {:?}", l.score()));
    //     let new_lineup = create_test_lineup(5500);
    //     println!("new score {}", new_lineup);
    //     let value = insert_value_sorted(&mut lineups, new_lineup);
    //     value.0.iter().for_each(|l| println!("New {:?}", l.score()));
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
