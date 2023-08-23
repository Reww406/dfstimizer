use std::io::Error;
use std::sync::Arc;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    mean, QB_ATT_MAX, QB_ATT_MIN, QB_PTS_MAX, QB_PTS_MIN, RB_ATT_MAX, RB_ATT_MIN, RB_PTS_MAX,
    RB_PTS_MIN, TE_TGTS_MAX, TE_TGTS_MIN, WR_PTS_MAX, WR_PTS_MIN, WR_TGTS_MAX, WR_TGTS_MIN,
};
use crate::{player::*, return_if_field_exits};

pub const SALARY_CAP: i32 = 59994;
pub const MAX_AVG_OWNERHSIP: f32 = 25.0;
pub const MIN_AVG_OWNERSHIP: f32 = 1.0;
pub const MAX_POINTS: f32 = 35.0;
pub const MIN_POINTS: f32 = 10.0;

// Thursday kicker, 5 flex spots
// maybe dont care about kicker

pub struct IslandLB {
    pub mvp: Option<Arc<LitePlayer>>,
    pub first: Option<Arc<LitePlayer>>,
    pub second: Option<Arc<LitePlayer>>,
    pub third: Option<Arc<LitePlayer>>,
    pub fourth: Option<Arc<LitePlayer>>,
}

#[derive(Clone, Debug)]
pub struct LineupBuilder {
    pub qb: Option<Arc<LitePlayer>>,
    pub rb1: Option<Arc<LitePlayer>>,
    pub rb2: Option<Arc<LitePlayer>>,
    pub wr1: Option<Arc<LitePlayer>>,
    pub wr2: Option<Arc<LitePlayer>>,
    pub wr3: Option<Arc<LitePlayer>>,
    pub te: Option<Arc<LitePlayer>>,
    pub flex: Option<Arc<LitePlayer>>,
    pub dst: Option<Arc<LitePlayer>>,
    pub total_price: i32,
}

// Will be converted to typed positions instead of generic playerown
#[derive(Debug, Clone, Default)]
pub struct Lineup {
    pub qb: QbProj,
    pub rb1: RbProj,
    pub rb2: RbProj,
    pub wr1: RecProj,
    pub wr2: RecProj,
    pub wr3: RecProj,
    pub te: RecProj,
    pub flex: FlexProj,
    pub def: DefProj,
    pub total_price: i32,
}

pub struct IslandLineup {
    pub mvp: Proj,
    pub first: Proj,
    pub second: Proj,
    pub third: Proj,
    pub fourth: Proj,
}

fn get_normalized_score(value: f32, max: f32, min: f32) -> f32 {
    (value - max) / (max - min)
}

// This can be way more DRY
impl Lineup {
    pub fn get_salary_spent_score(&self) -> f32 {
        let spent = self.total_price as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    pub fn get_ownership_score(&self) -> f32 {
        let averge_ownership: f32 = self.averge_ownership();
        -1.0 * (averge_ownership - 1.0) / (MAX_AVG_OWNERHSIP - MIN_AVG_OWNERSHIP)
    }

    pub fn averge_ownership(&self) -> f32 {
        let line_up_array: [f32; 9] = self.array_of_ownership();
        let ownerships: Vec<f32> = line_up_array.into_iter().collect();
        mean(&ownerships).unwrap()
    }

    pub fn score(&self) -> f32 {
        let scores: Vec<f32> = vec![
            self.get_ownership_score(),
            self.get_salary_spent_score(),
            self.rb_score(),
            self.wr_score(),
            self.te_score(),
            self.dst_score(),
            self.qb_score(),
            self.flex_score(),
        ];
        scores.iter().sum()
    }

    pub fn rb_score(&self) -> f32 {
        let rbs: [&RbProj; 2] = [&self.rb1, &self.rb2];
        let mut score = 0.0;
        rbs.iter().for_each(|rb| {
            score += get_normalized_score(rb.atts, *RB_ATT_MAX, *RB_ATT_MIN);
            score += get_normalized_score(rb.pts, *RB_PTS_MAX, *RB_PTS_MIN)
        });
        score
    }

    // Top 5 Stats once we have it
    pub fn qb_score(&self) -> f32 {
        let atts_score: f32 = (self.qb.atts - *QB_ATT_MIN) / (*QB_ATT_MAX - *QB_ATT_MIN);
        let pts_score: f32 = get_normalized_score(self.qb.pts, *QB_PTS_MAX, *QB_PTS_MIN);
        if self.qb.team == self.wr2.team || self.qb.team == self.wr3.team {
            return atts_score + pts_score + 0.5;
        }
        atts_score + pts_score
    }

    // Max/Min Receptions, Projected Points
    // Top 5 Average, Deep passes?
    pub fn wr_score(&self) -> f32 {
        let wrs: [&RecProj; 3] = [&self.wr1, &self.wr2, &self.wr3];
        let mut score: f32 = 0.0;
        wrs.iter().for_each(|wr| {
            score += get_normalized_score(wr.tgts, *WR_TGTS_MAX, *WR_TGTS_MIN);
            score += get_normalized_score(wr.pts, *WR_PTS_MAX, *WR_PTS_MIN);
        });
        score
    }

    // Max/Min Receptions, Projected Points
    pub fn flex_score(&self) -> f32 {
        match self.flex.pos {
            Pos::Wr => {
                let wr: &RecProj = self.flex.rec_proj.as_ref().unwrap();
                let wr_pts = get_normalized_score(wr.pts, *WR_PTS_MAX, *WR_PTS_MIN);
                let wr_tgts = get_normalized_score(wr.tgts, *WR_TGTS_MAX, *WR_TGTS_MIN);
                return wr_pts + wr_tgts;
            }
            Pos::Rb => {
                let rb: &RbProj = self.flex.rb_proj.as_ref().unwrap();
                let rb_pts = get_normalized_score(rb.pts, *RB_PTS_MAX, *RB_PTS_MIN);
                let rb_atts = get_normalized_score(rb.atts, *RB_ATT_MAX, *RB_ATT_MIN);
                return rb_pts + rb_atts;
            }
            _ => {
                panic!("Wrong Flex Pos..");
            }
        }
    }

    pub fn te_score(&self) -> f32 {
        return get_normalized_score(self.te.tgts, *TE_TGTS_MAX, *TE_TGTS_MIN);
    }

    // Points
    pub fn dst_score(&self) -> f32 {
        0.0
    }

    pub fn array_of_ownership(&self) -> [f32; 9] {
        let mut flex_own: f32 = 0.0;
        match self.flex.pos {
            Pos::Wr => {
                flex_own = self
                    .flex
                    .rec_proj
                    .as_ref()
                    .expect("Stored rec under wrong pos")
                    .own_per
            }
            Pos::Rb => {
                flex_own = self
                    .flex
                    .rb_proj
                    .as_ref()
                    .expect("Stored rb under wrong pos")
                    .own_per
            }
            _ => {
                panic!("Wrong POS in Flex..")
            }
        }
        [
            self.qb.own_per,
            self.rb1.own_per,
            self.rb2.own_per,
            self.wr1.own_per,
            self.wr2.own_per,
            self.wr3.own_per,
            self.te.own_per,
            flex_own,
            self.def.own_per,
        ]
    }
}

// TODO Would love to find a way to make this more DRY
impl LineupBuilder {
    pub fn new() -> Self {
        LineupBuilder {
            qb: None,
            rb1: None,
            rb2: None,
            wr1: None,
            wr2: None,
            wr3: None,
            te: None,
            flex: None,
            dst: None,
            total_price: 0,
        }
    }

    pub fn array_of_players(&self) -> [Arc<LitePlayer>; 9] {
        [
            self.qb.clone().expect("Line up missing qb"),
            self.rb1.clone().expect("Line up missing rb1"),
            self.rb2.clone().expect("Line up missing rb2"),
            self.wr1.clone().expect("Line up missing wr1"),
            self.wr2.clone().expect("Line up missing wr2"),
            self.wr3.clone().expect("Line up missing wr3"),
            self.te.clone().expect("Line up missing te"),
            self.flex.clone().expect("Line up missing flex"),
            self.dst.clone().expect("Line up missing def"),
        ]
    }

    pub fn get_salary_spent_score(&self) -> f32 {
        let spent = self.total_amount_spent() as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    pub fn total_amount_spent(&self) -> i32 {
        let line_up_array: [Arc<LitePlayer>; 9] = self.array_of_players();
        line_up_array.into_iter().map(|x| x.salary as i32).sum()
    }

    pub fn set_pos(mut self, lp: Arc<LitePlayer>, pos: Pos, slot: i8) -> LineupBuilder {
        match pos {
            Pos::Qb => self.qb = Some(return_if_field_exits(self.qb, &lp)),
            Pos::Rb => match slot {
                1 => self.rb1 = Some(return_if_field_exits(self.rb1, &lp)),
                2 => self.rb2 = Some(return_if_field_exits(self.rb2, &lp)),
                _ => panic!("Bad RB Slot"),
            },
            Pos::Wr => match slot {
                1 => self.wr1 = Some(return_if_field_exits(self.wr1, &lp)),
                2 => self.wr2 = Some(return_if_field_exits(self.wr2, &lp)),
                3 => self.wr3 = Some(return_if_field_exits(self.wr3, &lp)),
                _ => panic!("Bad WR Slot"),
            },
            Pos::Te => self.te = Some(return_if_field_exits(self.te, &lp)),
            Pos::D => self.dst = Some(return_if_field_exits(self.dst, &lp)),
            Pos::K => panic!("No kicker in regular optimizer."),
        }
        self.total_price += lp.salary as i32;
        self
    }

    // TODO Can turn these all into a match on a enum that has the slot
    pub fn set_qb(mut self, qb: Arc<LitePlayer>) -> LineupBuilder {
        self.qb = Some(return_if_field_exits(self.qb, &qb));
        self.total_price += qb.salary as i32;
        self
    }

    pub fn set_rb1(mut self, rb1: Arc<LitePlayer>) -> LineupBuilder {
        self.rb1 = Some(return_if_field_exits(self.rb1, &rb1));
        self.total_price += rb1.salary as i32;
        self
    }

    pub fn set_rb2(mut self, rb2: Arc<LitePlayer>) -> LineupBuilder {
        self.rb2 = Some(return_if_field_exits(self.rb2, &rb2));
        self.total_price += rb2.salary as i32;
        self
    }

    pub fn set_wr1(mut self, wr1: Arc<LitePlayer>) -> LineupBuilder {
        self.wr1 = Some(return_if_field_exits(self.wr1, &wr1));
        self.total_price += wr1.salary as i32;
        self
    }

    pub fn set_wr2(mut self, wr2: Arc<LitePlayer>) -> LineupBuilder {
        self.wr2 = Some(return_if_field_exits(self.wr2, &wr2));
        self.total_price += wr2.salary as i32;
        self
    }

    pub fn set_wr3(mut self, wr3: Arc<LitePlayer>) -> LineupBuilder {
        self.wr3 = Some(return_if_field_exits(self.wr3, &wr3));
        self.total_price += wr3.salary as i32;
        self
    }

    pub fn set_te(mut self, te: Arc<LitePlayer>) -> LineupBuilder {
        self.te = Some(return_if_field_exits(self.te, &te));
        self.total_price += te.salary as i32;
        self
    }

    pub fn set_flex(mut self, flex: Arc<LitePlayer>) -> LineupBuilder {
        self.flex = Some(return_if_field_exits(self.flex, &flex));
        self.total_price += flex.salary as i32;
        self
    }

    pub fn set_def(mut self, def: Arc<LitePlayer>) -> LineupBuilder {
        self.dst = Some(return_if_field_exits(self.dst, &def));
        self.total_price += def.salary as i32;
        self
    }
    // Will pull actual data from Sqlite
    pub fn build(
        self,
        week: i8,
        season: i16,
        conn: &Connection,
    ) -> Result<Lineup, Box<dyn std::error::Error>> {
        let flex: FlexProj = if self.flex.as_ref().unwrap().pos == Pos::Wr {
            FlexProj {
                pos: Pos::Wr,
                rec_proj: Some(
                    query_rec_proj(self.flex.as_ref().unwrap().id, week, season, &Pos::Wr, conn)
                        .ok_or("Could not find flex wr")?,
                ),
                rb_proj: None,
            }
        } else {
            FlexProj {
                pos: Pos::Rb,
                rec_proj: None,
                rb_proj: Some(
                    query_rb_proj(self.flex.as_ref().unwrap().id, week, season, conn)
                        .ok_or("Could not find flex rb")?,
                ),
            }
        };

        let qb: QbProj = query_qb_proj(self.qb.as_ref().unwrap().id, week, season, conn)
            .ok_or("QB Could not be found")?;
        let rb1: RbProj = query_rb_proj(self.rb1.as_ref().unwrap().id, week, season, conn)
            .ok_or("Rb1 Could not be found")?;
        let rb2: RbProj = query_rb_proj(self.rb2.as_ref().unwrap().id, week, season, conn)
            .ok_or("Rb2 could not be found")?;
        let wr1: RecProj =
            query_rec_proj(self.wr1.as_ref().unwrap().id, week, season, &Pos::Wr, conn)
                .ok_or("Wr1 could not be found")?;
        let wr2: RecProj =
            query_rec_proj(self.wr2.as_ref().unwrap().id, week, season, &Pos::Wr, conn)
                .ok_or("Wr2 could not be found")?;
        let wr3: RecProj =
            query_rec_proj(self.wr3.as_ref().unwrap().id, week, season, &Pos::Wr, conn)
                .ok_or("Wr3 could not be found")?;
        let te: RecProj =
            query_rec_proj(self.te.as_ref().unwrap().id, week, season, &Pos::Te, conn)
                .ok_or("Te could not be found")?;
        let flex: FlexProj = flex;
        let def: DefProj = query_def_proj(self.dst.as_ref().unwrap().id, week, season, conn)
            .ok_or("Def could not be found")?;

        Ok(Lineup {
            qb,
            rb1,
            rb2,
            wr1,
            wr2,
            wr3,
            te,
            flex,
            def,
            total_price: self.total_price,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     // TODO the lineupBuilder init could probably be done with a macro?
//     fn create_test_player(salary: i16, ownership: f32) -> LitePlayer {
//         LitePlayer {
//             id: 1,
//             salary,
//             pos: Pos::Rb,
//         }
//     }

//     fn create_lineup_vec(
//         points: f32,
//         price: i16,
//         ownership: f32,
//         double: bool,
//     ) -> Vec<Option<LitePlayer>> {
//         let mut players: Vec<Option<LitePlayer>> = Vec::new();
//         if double {
//             for _ in 0..4 {
//                 let player: LitePlayer = create_test_player(price, ownership);
//                 players.push(Some(player));
//             }
//             for _ in 4..9 {
//                 let player: LitePlayer = create_test_player(2 * price, 2.0 * ownership);
//                 players.push(Some(player));
//             }
//         } else {
//             for _ in 0..9 {
//                 let player: LitePlayer = create_test_player(price, ownership);
//                 players.push(Some(player));
//             }
//         }
//         players
//     }

//     #[test]
//     // fn test_calculate_total_salary() {
//     //     let p: Vec<Option<LitePlayer>> = create_lineup_vec(1.0, 1, 1.0, false);
//     //     let test_lineup: LineupBuilder = LineupBuilder {
//     //         qb: p[0].as_ref(),
//     //         rb1: p[1].as_ref(),
//     //         rb2: p[2].as_ref(),
//     //         wr1: p[3].as_ref(),
//     //         wr2: p[4].as_ref(),
//     //         wr3: p[5].as_ref(),
//     //         te: p[6].as_ref(),
//     //         flex: p[7].as_ref(),
//     //         dst: p[8].as_ref(),
//     //         total_price: 10,
//     //     };
//     //     assert_eq!(test_lineup.total_amount_spent(), 9);
//     // }

//     // #[test]
//     // fn test_lineup_builder_set_functions() {
//     //     let test_player: &LitePlayer = &create_test_player(1, 1.0);
//     //     let empty_lineup = LineupBuilder::new();
//     //     let qb_lineup = empty_lineup.set_qb(&test_player);
//     //     let rb_lineup = qb_lineup.set_rb2(&test_player);
//     //     assert_eq!(rb_lineup.total_price, 2)
//     // }
//     #[test]
//     // fn test_lineup_averge_functions() {
//     //     let p: Vec<Option<LitePlayer>> = create_lineup_vec(6.0, 1, 4.0, true);
//     //     let line_up: LineupBuilder = LineupBuilder {
//     //         qb: p[0].as_ref(),
//     //         rb1: p[1].as_ref(),
//     //         rb2: p[2].as_ref(),
//     //         wr1: p[3].as_ref(),
//     //         wr2: p[4].as_ref(),
//     //         wr3: p[5].as_ref(),
//     //         te: p[6].as_ref(),
//     //         flex: p[7].as_ref(),
//     //         dst: p[8].as_ref(),
//     //         total_price: 10,
//     //     };
//     //     assert_eq!(line_up.averge_ownership(), 6.2222223);
//     // }
//     #[test]
//     // fn test_score_functions() {
//     //     let max_value_players = create_lineup_vec(MAX_POINTS, 6666, MAX_AVG_OWNERHSIP, false);
//     //     let min_value_players = create_lineup_vec(MIN_POINTS, 0, MIN_AVG_OWNERSHIP, false);
//     //     let max_line_up: LineupBuilder = LineupBuilder {
//     //         qb: max_value_players[0].as_ref(),
//     //         rb1: max_value_players[1].as_ref(),
//     //         rb2: max_value_players[2].as_ref(),
//     //         wr1: max_value_players[3].as_ref(),
//     //         wr2: max_value_players[4].as_ref(),
//     //         wr3: max_value_players[5].as_ref(),
//     //         te: max_value_players[6].as_ref(),
//     //         flex: max_value_players[7].as_ref(),
//     //         dst: max_value_players[8].as_ref(),
//     //         total_price: 10,
//     //     };
//     //     let min_line_up: LineupBuilder = LineupBuilder {
//     //         qb: min_value_players[0].as_ref(),
//     //         rb1: min_value_players[1].as_ref(),
//     //         rb2: min_value_players[2].as_ref(),
//     //         wr1: min_value_players[3].as_ref(),
//     //         wr2: min_value_players[4].as_ref(),
//     //         wr3: min_value_players[5].as_ref(),
//     //         te: min_value_players[6].as_ref(),
//     //         flex: min_value_players[7].as_ref(),
//     //         dst: min_value_players[8].as_ref(),
//     //         total_price: 10,
//     //     };
//         // assert_eq!(max_line_up.get_ownership_score(), -1.0);
//         // assert_eq!(min_line_up.get_ownership_score(), 0.0);
//         assert_eq!(max_line_up.get_salary_spent_score(), 1.0);
//         assert_eq!(min_line_up.get_salary_spent_score(), 0.0);
//     }
// }
