use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::future::join_all;
use futures::StreamExt;
use rusqlite::Connection;

use crate::gen_comb;
use crate::lineup;
use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use std::collections::HashMap;
use std::sync::Arc;

// AYU DARK
const GOOD_SALARY_USAGE: i32 = 45000;

pub fn build_all_possible_lineups(
    players: Vec<Arc<LitePlayer>>,
    week: i8,
    season: i16,
) -> Vec<Lineup> {
    let pool = ThreadPool::new().unwrap();
    let mut qb_lineups: Vec<LineupBuilder> = Vec::new();
    let mut finished_lineups: Vec<Lineup> = Vec::new();
    players
        .iter()
        .filter(|player| player.pos == Pos::Qb)
        .for_each(|qb| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            qb_lineups.push(lineup_builder.set_qb(qb.clone()))
        });
    let mut futures: Vec<_> = Vec::new();
    for qb_lp in qb_lineups {
        let (tx, rx) = mpsc::unbounded::<Lineup>();
        let binding = qb_lp.clone();
        let future = async {
            let player_clone = players.clone();
            let fut_tx_result = async move {
                println!("Start thread {:?}", std::thread::current().id());
                let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(&player_clone, &binding);
                let wrs_lineups: Vec<LineupBuilder> =
                    add_wrs_to_lineups(&player_clone, rbs_lineups);
                let te_lineups: Vec<LineupBuilder> = add_te_to_lineups(&player_clone, wrs_lineups);
                let dst_lineups: Vec<LineupBuilder> = add_dst_to_lineups(&player_clone, te_lineups);
                let filterd_lineups = filter_low_salary_cap(dst_lineups, 42000);
                let no_bad_combinations = filter_bad_lineups(filterd_lineups, week, season);
                let lineups: Vec<Lineup> =
                    add_flex_find_top_num(&player_clone, no_bad_combinations, 500, week, season);
                lineups.iter().for_each(|l: &Lineup| {
                    tx.unbounded_send(l.clone()).expect("Failed to send lineup")
                });
                println!("Stopped thread {:?}", std::thread::current().id());
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
    finished_lineups
}

pub fn filter_low_salary_cap(
    mut lineups: Vec<LineupBuilder>,
    filter_cap: i32,
) -> Vec<LineupBuilder> {
    lineups.retain(|l| l.total_price > filter_cap);
    lineups
}

pub fn get_combos_for_pos<'a>(
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
    let wr_combo: Vec<Vec<Arc<LitePlayer>>> = get_combos_for_pos(players, 3, &[Pos::Wr]);
    for lineup in &lineups {
        for combo in &wr_combo {
            new_lineups.push(
                lineup
                    .clone()
                    .set_wr1((*p_lookup.get(&combo[0].id).expect("Player missing")).clone())
                    .set_wr2((*p_lookup.get(&combo[1].id).expect("Missing Player")).clone())
                    .set_wr3((*p_lookup.get(&combo[2].id).expect("Missing Player")).clone()),
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
    let rb_combos: Vec<Vec<Arc<LitePlayer>>> = get_combos_for_pos(players, 2, &[Pos::Rb]);
    for combo in &rb_combos {
        new_lineups.push(
            lineup
                .clone()
                .set_rb1((*p_lookup.get(&combo[0].id).expect("Player missing")).clone())
                .set_rb2((*p_lookup.get(&combo[1].id).expect("Player Missing")).clone()),
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
            .filter(|p| (p.salary as i32 + lineup.total_price) < SALARY_CAP)
            .filter(|p| (p.salary as i32 + lineup.total_price) > GOOD_SALARY_USAGE)
            .for_each(|flex| {
                // should be refactored to a function
                iterations += 1;
                let finished_lineup = lineup
                    .clone()
                    .set_flex(flex.clone())
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
    println!("Flex iterated {} times", iterations);
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
            lineups_with_def.push(lineup.clone().set_def(def.clone()));
            iterations += 1;
        });
    }
    println!("Def iterated: {} times", iterations);
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
            total_price: price,
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
