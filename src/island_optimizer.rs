use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::future::join_all;
use futures::StreamExt;
use rusqlite::Connection;

use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use crate::SEASON;
use crate::WEEK;
use itertools::Itertools;
use std::sync::Arc;

// We should but every player in the MVP Spot, then generate combos
// minus that player for the rest of the positions
pub fn build_island_lineups(
    players: Vec<Arc<LitePlayer>>,
    week: i8,
    season: i16,
) -> Vec<IslandLineup> {
    let pool = ThreadPool::new().unwrap();
    let mut mvp_lineups: Vec<IslandLB> = Vec::new();
    let mut finished_lineups: Vec<IslandLineup> = Vec::new();
    // TODO Filter down this list Defense and stuff not needed..?
    players.iter().for_each(|p| {
        let mvp_lineup: IslandLB = IslandLB::new();
        mvp_lineups.push(mvp_lineup.set_slot(&p, Slot::Mvp))
    });
    let mut futures: Vec<_> = Vec::new();
    for lineup in mvp_lineups {
        let (tx, rx) = mpsc::unbounded::<IslandLineup>();
        let binding: IslandLB = lineup.clone();
        let future = async {
            let player_clone: Vec<Arc<LitePlayer>> = players.clone();
            let fut_tx_result = async move {
                build_and_score_combos(&binding, &player_clone, 200, week, season)
                    .into_iter()
                    .for_each(|l| tx.unbounded_send(l).expect("Failed to send result"))
            };

            pool.spawn_ok(fut_tx_result);

            let future = rx.collect::<Vec<IslandLineup>>();
            future.await
        };
        futures.push(future);
    }
    let futures_join = join_all(futures);
    let test = executor::block_on(futures_join);
    for future in test {
        finished_lineups.extend(future);
    }
    finished_lineups.sort_by(|a, b: &IslandLineup| b.score.partial_cmp(&a.score).unwrap());
    finished_lineups
}

fn build_and_score_combos(
    mvp_lineup: &IslandLB,
    players: &Vec<Arc<LitePlayer>>,
    amount: usize,
    week: i8,
    season: i16,
) -> Vec<IslandLineup> {
    let conn: Connection = Connection::open(DATABASE_FILE).expect("Couldn't Open DB File");
    let mut best_lineups: Vec<IslandLineup> = Vec::with_capacity(amount);
    let mut sorted: bool = false;
    let mut lowest_score = 0.0;
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

        let new_lineup = island_lb.build(WEEK, SEASON, &conn);
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
            // TODO Dont need to iterate over whole list
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
