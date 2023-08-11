use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{mean, optimizer::calculate_lineup_score};
use crate::{player::*, return_if_field_exits};

pub const SALARY_CAP: i32 = 59994;
pub const MAX_AVG_OWNERHSIP: f32 = 25.0;
pub const MIN_AVG_OWNERSHIP: f32 = 1.0;
pub const MAX_POINTS: f32 = 35.0;
pub const MIN_POINTS: f32 = 10.0;

#[derive(Clone)]
pub struct LineupBuilder<'a> {
    pub qb: Option<&'a LitePlayer>,
    pub rb1: Option<&'a LitePlayer>,
    pub rb2: Option<&'a LitePlayer>,
    pub wr1: Option<&'a LitePlayer>,
    pub wr2: Option<&'a LitePlayer>,
    pub wr3: Option<&'a LitePlayer>,
    pub te: Option<&'a LitePlayer>,
    pub flex: Option<&'a LitePlayer>,
    pub dst: Option<&'a LitePlayer>,
    pub total_price: i32,
}

// Will be converted to typed positions instead of generic playerown
#[derive(Debug, Clone)]
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
    pub score: f32,
}

// TODO Would love to find a way to make this more DRY
impl<'a> LineupBuilder<'a> {
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

    pub fn array_of_players(&self) -> [&LitePlayer; 9] {
        [
            &self.qb.expect("Line up missing qb"),
            &self.rb1.expect("Line up missing rb1"),
            &self.rb2.expect("Line up missing rb2"),
            &self.wr1.expect("Line up missing wr1"),
            &self.wr2.expect("Line up missing wr2"),
            &self.wr3.expect("Line up missing wr3"),
            &self.te.expect("Line up missing te"),
            &self.flex.expect("Line up missing flex"),
            &self.dst.expect("Line up missing def"),
        ]
    }

    pub fn get_salary_spent_score(&self) -> f32 {
        let spent = self.total_amount_spent() as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    // pub fn get_ownership_score(&self) -> f32 {
    //     let averge_ownership: f32 = self.averge_ownership();
    //     -1.0 * (averge_ownership - 1.0) / (MAX_AVG_OWNERHSIP - MIN_AVG_OWNERSHIP)
    // }

    pub fn total_amount_spent(&self) -> i32 {
        let line_up_array: [&LitePlayer; 9] = self.array_of_players();
        line_up_array.into_iter().map(|x| x.salary as i32).sum()
    }

    // pub fn averge_ownership(&self) -> f32 {
    //     let line_up_array: [&LitePlayer; 9] = self.array_of_players();
    //     let ownerships: Vec<f32> = line_up_array.into_iter().map(|p| p.own_per).collect();
    //     mean(&ownerships).unwrap()
    // }

    pub fn set_qb(mut self, qb: &'a LitePlayer) -> LineupBuilder<'a> {
        self.qb = Some(return_if_field_exits(self.qb, qb));
        self.total_price += qb.salary as i32;
        self
    }

    pub fn set_rb1(mut self, rb1: &'a LitePlayer) -> LineupBuilder<'a> {
        self.rb1 = Some(return_if_field_exits(self.rb1, rb1));
        self.total_price += rb1.salary as i32;
        self
    }

    pub fn set_rb2(mut self, rb2: &'a LitePlayer) -> LineupBuilder<'a> {
        self.rb2 = Some(return_if_field_exits(self.rb2, rb2));
        self.total_price += rb2.salary as i32;
        self
    }

    pub fn set_wr1(mut self, wr1: &'a LitePlayer) -> LineupBuilder<'a> {
        self.wr1 = Some(return_if_field_exits(self.wr1, wr1));
        self.total_price += wr1.salary as i32;
        self
    }

    pub fn set_wr2(mut self, wr2: &'a LitePlayer) -> LineupBuilder<'a> {
        self.wr2 = Some(return_if_field_exits(self.wr2, wr2));
        self.total_price += wr2.salary as i32;
        self
    }

    pub fn set_wr3(mut self, wr3: &'a LitePlayer) -> LineupBuilder<'a> {
        self.wr3 = Some(return_if_field_exits(self.wr3, wr3));
        self.total_price += wr3.salary as i32;
        self
    }

    pub fn set_te(mut self, te: &'a LitePlayer) -> LineupBuilder<'a> {
        self.te = Some(return_if_field_exits(self.te, te));
        self.total_price += te.salary as i32;
        self
    }

    pub fn set_flex(mut self, flex: &'a LitePlayer) -> LineupBuilder<'a> {
        self.flex = Some(return_if_field_exits(self.flex, flex));
        self.total_price += flex.salary as i32;
        self
    }

    pub fn set_def(mut self, def: &'a LitePlayer) -> LineupBuilder<'a> {
        self.dst = Some(return_if_field_exits(self.dst, def));
        self.total_price += def.salary as i32;
        self
    }
    // Will pull actual data from Sqlite
    pub fn build(self, week: i8, season: i16, conn: &Connection) -> Lineup {
        let flex: FlexProj = if self.flex.unwrap().pos == Pos::Wr {
            FlexProj {
                pos: Pos::Wr,
                rec_proj: Some(
                    query_rec_proj(self.flex.unwrap().id, week, season, &Pos::Wr, conn).unwrap(),
                ),
                rb_proj: None,
            }
        } else {
            FlexProj {
                pos: Pos::Rb,
                rec_proj: None,
                rb_proj: Some(query_rb_proj(self.flex.unwrap().id, week, season, conn).unwrap()),
            }
        };

        Lineup {
            qb: query_qb_proj(self.qb.unwrap().id, week, season, conn).unwrap(),
            rb1: query_rb_proj(self.rb1.unwrap().id, week, season, conn).unwrap(),
            rb2: query_rb_proj(self.rb2.unwrap().id, week, season, conn).unwrap(),
            wr1: query_rec_proj(self.wr1.unwrap().id, week, season, &Pos::Wr, conn).unwrap(),
            wr2: query_rec_proj(self.wr2.unwrap().id, week, season, &Pos::Wr, conn).unwrap(),
            wr3: query_rec_proj(self.wr3.unwrap().id, week, season, &Pos::Wr, conn).unwrap(),
            te: query_rec_proj(self.te.unwrap().id, week, season, &Pos::Wr, conn).unwrap(),
            flex: flex,
            def: query_def_proj(self.dst.unwrap().id, week, season, conn).unwrap(),
            total_price: self.total_price,
            score: calculate_lineup_score(&self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO the lineupBuilder init could probably be done with a macro?
    fn create_test_player(salary: i16, ownership: f32) -> LitePlayer {
        LitePlayer {
            id: 1,
            salary,
            pos: Pos::Rb,
        }
    }

    fn create_lineup_vec(
        points: f32,
        price: i16,
        ownership: f32,
        double: bool,
    ) -> Vec<Option<LitePlayer>> {
        let mut players: Vec<Option<LitePlayer>> = Vec::new();
        if double {
            for _ in 0..4 {
                let player: LitePlayer = create_test_player(price, ownership);
                players.push(Some(player));
            }
            for _ in 4..9 {
                let player: LitePlayer = create_test_player(2 * price, 2.0 * ownership);
                players.push(Some(player));
            }
        } else {
            for _ in 0..9 {
                let player: LitePlayer = create_test_player(price, ownership);
                players.push(Some(player));
            }
        }
        players
    }

    #[test]
    fn test_calculate_total_salary() {
        let p: Vec<Option<LitePlayer>> = create_lineup_vec(1.0, 1, 1.0, false);
        let test_lineup: LineupBuilder = LineupBuilder {
            qb: p[0].as_ref(),
            rb1: p[1].as_ref(),
            rb2: p[2].as_ref(),
            wr1: p[3].as_ref(),
            wr2: p[4].as_ref(),
            wr3: p[5].as_ref(),
            te: p[6].as_ref(),
            flex: p[7].as_ref(),
            dst: p[8].as_ref(),
            total_price: 10,
        };
        assert_eq!(test_lineup.total_amount_spent(), 9);
    }

    #[test]
    fn test_lineup_builder_set_functions() {
        let test_player: &LitePlayer = &create_test_player(1, 1.0);
        let empty_lineup = LineupBuilder::new();
        let qb_lineup = empty_lineup.set_qb(&test_player);
        let rb_lineup = qb_lineup.set_rb2(&test_player);
        assert_eq!(rb_lineup.total_price, 2)
    }
    #[test]
    // fn test_lineup_averge_functions() {
    //     let p: Vec<Option<LitePlayer>> = create_lineup_vec(6.0, 1, 4.0, true);
    //     let line_up: LineupBuilder = LineupBuilder {
    //         qb: p[0].as_ref(),
    //         rb1: p[1].as_ref(),
    //         rb2: p[2].as_ref(),
    //         wr1: p[3].as_ref(),
    //         wr2: p[4].as_ref(),
    //         wr3: p[5].as_ref(),
    //         te: p[6].as_ref(),
    //         flex: p[7].as_ref(),
    //         dst: p[8].as_ref(),
    //         total_price: 10,
    //     };
    //     assert_eq!(line_up.averge_ownership(), 6.2222223);
    // }
    #[test]
    fn test_score_functions() {
        let max_value_players = create_lineup_vec(MAX_POINTS, 6666, MAX_AVG_OWNERHSIP, false);
        let min_value_players = create_lineup_vec(MIN_POINTS, 0, MIN_AVG_OWNERSHIP, false);
        let max_line_up: LineupBuilder = LineupBuilder {
            qb: max_value_players[0].as_ref(),
            rb1: max_value_players[1].as_ref(),
            rb2: max_value_players[2].as_ref(),
            wr1: max_value_players[3].as_ref(),
            wr2: max_value_players[4].as_ref(),
            wr3: max_value_players[5].as_ref(),
            te: max_value_players[6].as_ref(),
            flex: max_value_players[7].as_ref(),
            dst: max_value_players[8].as_ref(),
            total_price: 10,
        };
        let min_line_up: LineupBuilder = LineupBuilder {
            qb: min_value_players[0].as_ref(),
            rb1: min_value_players[1].as_ref(),
            rb2: min_value_players[2].as_ref(),
            wr1: min_value_players[3].as_ref(),
            wr2: min_value_players[4].as_ref(),
            wr3: min_value_players[5].as_ref(),
            te: min_value_players[6].as_ref(),
            flex: min_value_players[7].as_ref(),
            dst: min_value_players[8].as_ref(),
            total_price: 10,
        };
        // assert_eq!(max_line_up.get_ownership_score(), -1.0);
        // assert_eq!(min_line_up.get_ownership_score(), 0.0);
        assert_eq!(max_line_up.get_salary_spent_score(), 1.0);
        assert_eq!(min_line_up.get_salary_spent_score(), 0.0);
    }
}
