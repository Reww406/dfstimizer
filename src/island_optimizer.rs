use std::sync::Arc;

use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::future::join_all;
use futures::StreamExt;
use rusqlite::Connection;

use crate::get_slate;
use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use crate::GAME_DAY;
use crate::MIN_SAL;
use crate::SALARY_CAP;
use crate::SEASON;
use crate::WEEK;
use itertools::Itertools;
// use std::sync::Rc;

fn get_mvp_ids(players: Vec<LitePlayer>) -> Vec<Arc<i16>> {
    let mvp_pos: &[Pos; 1] = &[Pos::Qb];
    players
        .into_iter()
        .filter(|p| mvp_pos.contains(&p.pos))
        .map(|p| Arc::new(p.id))
        .collect::<Vec<Arc<i16>>>()
}

pub fn build_island_lineups(week: i8, season: i16) -> Vec<IslandLineup> {
    let pool: ThreadPool = ThreadPool::new().unwrap();
    let mut finished_lineups: Vec<IslandLineup> = Vec::new();
    let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
    let players: Vec<LitePlayer> = get_slate(week, season, &GAME_DAY, &conn);
    let ids: Vec<Arc<i16>> = get_mvp_ids(players);
    let mut futures: Vec<_> = Vec::new();
    for id in ids {
        let (tx, rx) = mpsc::unbounded::<IslandLineup>();
        let future = async {
            let fut_tx_result = async move {
                let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
                let mut mvp_lineup: IslandLB = IslandLB::new();
                let thread_players: Vec<LitePlayer> =
                    get_slate(week, season, &GAME_DAY, &conn);
                for player in &thread_players {
                    if player.id == *id {
                        mvp_lineup = mvp_lineup.set_slot(player, Slot::Mvp);
                    }
                }
                build_and_score_combos(&mvp_lineup, &thread_players)
                    .into_iter()
                    .for_each(|l| tx.unbounded_send(l).expect("Failed to send result"));
            };

            pool.spawn_ok(fut_tx_result);

            let future = rx.collect::<Vec<IslandLineup>>();
            future.await
        };
        futures.push(future);
    }
    let futures_join = join_all(futures);
    let complete_futures: Vec<Vec<IslandLineup>> = executor::block_on(futures_join);
    for future in complete_futures {
        finished_lineups.extend(future);
    }
    finished_lineups.sort_by(|a, b: &IslandLineup| b.score.partial_cmp(&a.score).unwrap());
    finished_lineups
}

fn build_and_score_combos(mvp_lineup: &IslandLB, players: &Vec<LitePlayer>) -> Vec<IslandLineup> {
    let amount: usize = 20;
    let conn: Connection = Connection::open(DATABASE_FILE).expect("Couldn't Open DB File");
    let mut best_lineups: Vec<IslandLineup> = Vec::new();
    let mut lowest_score: f32 = 0.0;
    let mut sorted: bool = false;
    for combo in players
        .iter()
        .filter(|p| p.id != mvp_lineup.mvp.as_ref().unwrap().id)
        .combinations(4)
    {
        let island_lb: IslandLB = mvp_lineup
            .clone()
            .set_slot(combo[0], Slot::First)
            .set_slot(combo[1], Slot::Second)
            .set_slot(combo[2], Slot::Third)
            .set_slot(combo[3], Slot::Fourth);
        if island_lb.salary_used > SALARY_CAP || island_lb.salary_used < MIN_SAL {
            continue;
        }
        let new_lineup: IslandLineup = island_lb.build(WEEK, SEASON, &conn);
        let score: f32 = new_lineup.score;
        if best_lineups.len() == amount && sorted == false {
            best_lineups.sort_by(|a, b: &IslandLineup| b.score.partial_cmp(&a.score).unwrap());
            sorted = true;
        }
        if best_lineups.len() < amount {
            if score < lowest_score {
                lowest_score = score;
            }
            best_lineups.push(new_lineup);
        } else if score > lowest_score {
            for i in 0..best_lineups.len() {
                if score > best_lineups[i].score {
                    best_lineups[i] = new_lineup;
                    if i == (best_lineups.len() - 1) {
                        lowest_score = score;
                    }
                    break;
                }
            }
        }
    }
    best_lineups
}
