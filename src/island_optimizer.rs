use futures::channel::mpsc;
use futures::executor;
use futures::executor::threadpool;
use futures::future::join_all;
use futures::streamext;
use rusqlite::connection;

use crate::database_file;
use crate::gen_comb;
use crate::lineup;
use crate::lineup::*;
use crate::player::*;
use std::collections::hashmap;
use std::sync::arc;

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

pub fn build_all_possible_lineups(
    players: Vec<Arc<LitePlayer>>,
    // wr_lineup: Arc<LineupBuilder>,
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

pub fn build_all_possible_lineups(
    players: Vec<Arc<LitePlayer>>,
    // wr_lineup: Arc<LineupBuilder>,
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
