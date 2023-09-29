use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::future::join_all;
use futures::StreamExt;
use itertools::Itertools;
use rusqlite::Connection;

use crate::filter_top_salary_players;
use crate::get_slate;
use crate::get_top_players_by_pos;
use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use crate::GAME_DAY;
use crate::MIN_SAL;
use crate::SALARY_CAP;
use crate::WR_COUNT;
use std::collections::HashMap;
use std::time::Instant;

pub fn build_all_possible_lineups(week: i8, season: i16) -> Vec<Lineup> {
    let pool: ThreadPool = ThreadPool::new().unwrap();
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let mut finished_lineups: Vec<Lineup> = Vec::new();
    let wr_ids: Vec<i16> =
        get_top_players_by_pos(season, week, &Pos::Wr, WR_COUNT, &GAME_DAY, &conn);
    println!("Cooking up LINEUPS!! {} WRs", wr_ids.len());
    let mut futures: Vec<_> = Vec::new();
    for wr_id in wr_ids.into_iter().combinations(3) {
        let (tx, rx) = mpsc::unbounded::<Lineup>();
        let future = async {
            let fut_tx_result = async move {
                let start: Instant = Instant::now();
                let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
                let thread_players: Vec<LitePlayer> =
                    get_slate(week, season, &GAME_DAY, true, &conn);
                drop(conn);
                let mut qb_lineups: Vec<LineupBuilder> = Vec::new();
                thread_players
                    .iter()
                    .filter(|player: &&LitePlayer| player.pos == Pos::Qb)
                    .for_each(|qb: &LitePlayer| {
                        let lineup_builder: LineupBuilder = LineupBuilder::new();
                        qb_lineups.push(lineup_builder.set_pos(&qb, Slot::None))
                    });
                let wr_lineups: Vec<LineupBuilder> =
                    add_wrs_to_lineups(wr_id, &thread_players, qb_lineups);
                let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(
                    &thread_players
                        .clone()
                        .into_iter()
                        .filter(|p| p.pos == Pos::Rb)
                        .collect_vec(),
                    wr_lineups,
                );
                let te_lineups: Vec<LineupBuilder> = add_te_to_lineups(
                    &thread_players
                        .clone()
                        .into_iter()
                        .filter(|p| p.pos == Pos::Te)
                        .collect_vec(),
                    rbs_lineups,
                );
                let dst_lineups: Vec<LineupBuilder> = add_dst_to_lineups(
                    &thread_players
                        .clone()
                        .into_iter()
                        .filter(|p| p.pos == Pos::D)
                        .collect_vec(),
                    te_lineups,
                );
                // let filterd_lineups: Vec<LineupBuilder> = filter_low_salary_cap(dst_lineups, 39500);
                let flex_pos: [Pos; 2] = [Pos::Wr, Pos::Rb];
                let lineup: Option<Lineup> = add_flex_find_top_num(
                    &thread_players
                        .into_iter()
                        .filter(|p| flex_pos.contains(&p.pos))
                        .collect_vec(),
                    dst_lineups,
                    week,
                    season,
                );
                if lineup.is_some() {
                    tx.unbounded_send(lineup.unwrap())
                        .expect("Failed to send lineup")
                }
                println!("Finished Thread {:?}", start.elapsed());
            };
            pool.spawn_ok(fut_tx_result);

            let future = rx.collect::<Vec<Lineup>>();
            future.await
        };
        futures.push(future);
    }
    let futures_join = join_all(futures);
    let done_futures = executor::block_on(futures_join);
    for future in done_futures {
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

// Needs to barrow players so it can be passed to the rest of the functions
pub fn add_wrs_to_lineups(
    player_ids: Vec<i16>,
    players: &Vec<LitePlayer>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut new_lineups: Vec<LineupBuilder> = Vec::new();
    let p_lookup: HashMap<i16, &LitePlayer> = LitePlayer::player_lookup_map(players);
    for lineup in &lineups {
        new_lineups.push(
            lineup
                .clone()
                .set_pos(
                    *p_lookup.get(&player_ids[0]).expect("Player missing"),
                    Slot::First,
                )
                .set_pos(
                    *p_lookup.get(&player_ids[1]).expect("Missing Player"),
                    Slot::Second,
                )
                .set_pos(
                    *p_lookup.get(&player_ids[2]).expect("Missing Player"),
                    Slot::Third,
                ),
        )
    }
    new_lineups
}

pub fn add_rbs_to_lineups(
    rbs: &Vec<LitePlayer>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut new_lineups: Vec<LineupBuilder> = Vec::new();
    let p_lookup: HashMap<i16, &LitePlayer> = LitePlayer::player_lookup_map(&rbs);
    for lineup in lineups {
        for combo in rbs.iter().combinations(2) {
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
    }
    new_lineups
}

pub fn add_te_to_lineups(tes: &Vec<LitePlayer>, lineups: Vec<LineupBuilder>) -> Vec<LineupBuilder> {
    let mut lineups_with_te: Vec<LineupBuilder> = Vec::with_capacity(lineups.len());
    for lineup in &lineups {
        tes.iter().for_each(|te| {
            lineups_with_te.push(lineup.clone().set_pos(&te, Slot::None));
        })
    }
    lineups_with_te
}

pub fn add_flex_find_top_num(
    players: &Vec<LitePlayer>,
    lineups: Vec<LineupBuilder>,
    week: i8,
    season: i16,
) -> Option<Lineup> {
    let mut best_lineup: Option<Lineup> = None;
    let mut highest_score: f32 = 0.0;
    for lineup in &lineups {
        let exiting_players: [&i16; 5] = [
            &lineup.rb1.as_ref().unwrap().id,
            &lineup.rb2.as_ref().unwrap().id,
            &lineup.wr1.as_ref().unwrap().id,
            &lineup.wr2.as_ref().unwrap().id,
            &lineup.wr3.as_ref().unwrap().id,
        ];
        players
            .iter()
            .filter(|p| !exiting_players.contains(&&p.id))
            .filter(|p| {
                (p.salary as i32 + lineup.salary_used) < SALARY_CAP
                    && (p.salary as i32 + lineup.salary_used) > MIN_SAL
            })
            .for_each(|flex| {
                let finished_lineup = lineup
                    .set_pos(&flex, Slot::Flex)
                    .build(week, season)
                    .expect("Failed to build lineup..");
                if finished_lineup.fits_own_brackets() {
                    let score: f32 = finished_lineup.score();
                    if best_lineup.is_none() {
                        best_lineup = Some(finished_lineup);
                        highest_score = score;
                    } else if score > highest_score {
                        highest_score = score;
                        best_lineup = Some(finished_lineup)
                    }
                }
            });
    }
    best_lineup
}

pub fn add_dst_to_lineups(
    players: &Vec<LitePlayer>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineups_with_def: Vec<LineupBuilder> = Vec::new();
    let mut iterations: i64 = 0;
    for lineup in &lineups {
        players.iter().for_each(|def| {
            lineups_with_def.push(lineup.clone().set_pos(&def, Slot::None));
            iterations += 1;
        });
    }
    lineups_with_def
}

// TODO Test build lineup should be sorted and under salary cap.
#[cfg(test)]
mod tests {

    // use super::*;
    // Helper function for creating line ups
    // fn create_test_lineup(price: i32) -> Lineup {
    //     let conn: Connection = Connection::open(DATABASE_FILE).expect("Failed to open DB");
    //     let week = 18;
    //     let season = 2022;
    //     Lineup {
    //         qb: query_qb_proj(26, week, season, &conn).unwrap(),
    //         rb1: query_rb_proj(1, week, season, &conn).unwrap(),
    //         rb2: query_rb_proj(2, week, season, &conn).unwrap(),
    //         wr1: query_rec_proj(3, week, season, &Pos::Wr, &conn).unwrap(),
    //         wr2: query_rec_proj(8, week, season, &Pos::Wr, &conn).unwrap(),
    //         wr3: query_rec_proj(8, week, season, &Pos::Wr, &conn).unwrap(),
    //         te: query_rec_proj(56, week, season, &Pos::Te, &conn).unwrap(),
    //         flex: FlexProj {
    //             rb_proj: Some(query_rb_proj(2, week, season, &conn).unwrap()),
    //             pos: Pos::Rb,
    //             rec_proj: None,
    //         },
    //         def: query_def_proj(17, week, season, &conn).unwrap(),
    //         salary_used: price,
    //     }
    // }
}
