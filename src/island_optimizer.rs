use std::rc::Rc;
use std::sync::Arc;

use futures::channel::mpsc;
use futures::executor;
use futures::executor::ThreadPool;
use futures::future::join_all;
use futures::StreamExt;
use rusqlite::Connection;

use crate::get_sunday_slate;
use crate::lineup::*;
use crate::player::*;
use crate::DATABASE_FILE;
use crate::SEASON;
use crate::WEEK;
use itertools::Itertools;
// use std::sync::Rc;

fn get_mvp_ids(players: Vec<Rc<LitePlayer>>) -> Vec<Arc<i16>> {
    let mvp_pos = &[Pos::Qb, Pos::Rb, Pos::Wr];
    players
        .into_iter()
        .filter(|p| mvp_pos.contains(&p.pos))
        .map(|p| Arc::new(p.id))
        .collect::<Vec<Arc<i16>>>()
}

pub fn build_island_lineups(week: i8, season: i16) -> Vec<IslandLineup> {
    let pool: ThreadPool = ThreadPool::new().unwrap();
    let mut finished_lineups: Vec<IslandLineup> = Vec::new();
    let players: Vec<Rc<LitePlayer>> = get_sunday_slate(week, season);
    let ids: Vec<Arc<i16>> = get_mvp_ids(players);
    let mut futures: Vec<_> = Vec::new();
    for id in ids {
        let (tx, rx) = mpsc::unbounded::<IslandLineup>();
        let future = async {
            let fut_tx_result = async move {
                let mut mvp_lineup: IslandLB = IslandLB::new();
                let thread_players: Vec<Rc<LitePlayer>> = get_sunday_slate(week, season);
                for player in &thread_players {
                    if player.id == *id {
                        mvp_lineup = mvp_lineup.set_slot(player, Slot::Mvp);
                    }
                }
                let lineup = build_and_score_combos(&mvp_lineup, &thread_players);
                tx.unbounded_send(lineup).expect("Failed to send result");
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
    players: &Vec<Rc<LitePlayer>>,
) -> IslandLineup {
    let conn: Connection = Connection::open(DATABASE_FILE).expect("Couldn't Open DB File");
    let mut best_lineup: Option<IslandLineup> = None;
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
        if best_lineup.is_none() {
            best_lineup = Some(new_lineup)
        } else if score < lowest_score {
            lowest_score = score;
            best_lineup = Some(new_lineup);
        }
    }
    best_lineup.expect("Found no lineups")
}
//  if best_lineups.len() < amount {
//             if score < lowest_score {
//                 lowest_score = score;
//             }
//             best_lineups.push(new_lineup);
//         } else if score > lowest_score {
//             // TODO Dont need to iterate over whole list
//             for i in 0..best_lineups.len() {
//                 if score > best_lineups[i].score {
//                     best_lineups[i] = new_lineup;
//                     if i == (best_lineups.len() - 1) {
//                         lowest_score = score;
//                     }
//                     break;
//                 }
//             }
//         }
