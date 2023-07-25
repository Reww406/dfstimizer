use serde::{Deserialize, Serialize};

use crate::{mean, optimizer::calculate_lineup_score};
use crate::{player::*, return_if_field_exits};

pub const SALARY_CAP: i32 = 59994;
pub const MAX_AVG_OWNERHSIP: f32 = 60.0;
pub const MIN_AVG_OWNERSHIP: f32 = 1.0;
pub const MAX_POINTS: f32 = 40.0;
pub const MIN_POINTS: f32 = 10.0;

#[derive(Clone)]
pub struct LineupBuilder<'a> {
    pub qb: Option<&'a Player>,
    pub rb1: Option<&'a Player>,
    pub rb2: Option<&'a Player>,
    pub wr1: Option<&'a Player>,
    pub wr2: Option<&'a Player>,
    pub wr3: Option<&'a Player>,
    pub te: Option<&'a Player>,
    pub flex: Option<&'a Player>,
    pub dst: Option<&'a Player>,
    pub total_price: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Lineup {
    pub qb: Player,
    pub rb1: Player,
    pub rb2: Player,
    pub wr1: Player,
    pub wr2: Player,
    pub wr3: Player,
    pub te: Player,
    pub flex: Player,
    pub def: Player,
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

    pub fn array_of_players(&self) -> [&Player; 9] {
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

    pub fn get_ownership_score(&self) -> f32 {
        let averge_ownership: f32 = self.averge_ownership();
        -1.0 * (averge_ownership - 1.0) / (MAX_AVG_OWNERHSIP - MIN_AVG_OWNERSHIP)
    }

    pub fn get_points_score(&self) -> f32 {
        let average_points: f32 = self.averge_projected_points();
        (average_points - MIN_POINTS) / (MAX_POINTS - MIN_POINTS)
    }

    pub fn total_amount_spent(&self) -> i32 {
        let line_up_array: [&Player; 9] = self.array_of_players();
        line_up_array.into_iter().map(|x| x.price as i32).sum()
    }

    pub fn averge_ownership(&self) -> f32 {
        let line_up_array: [&Player; 9] = self.array_of_players();
        let ownerships: Vec<f32> = line_up_array.into_iter().map(|p| p.ownership).collect();
        mean(&ownerships).unwrap()
    }

    pub fn averge_projected_points(&self) -> f32 {
        let line_up_array: [&Player; 9] = self.array_of_players();
        let points: Vec<f32> = line_up_array
            .into_iter()
            .map(|p: &Player| p.points)
            .collect();
        mean(&points).unwrap()
    }

    pub fn set_qb(mut self, qb: &'a Player) -> LineupBuilder<'a> {
        self.qb = Some(return_if_field_exits(self.qb, qb));
        self.total_price += qb.price as i32;
        self
    }

    pub fn set_rb1(mut self, rb1: &'a Player) -> LineupBuilder<'a> {
        self.rb1 = Some(return_if_field_exits(self.rb1, rb1));
        self.total_price += rb1.price as i32;
        self
    }

    pub fn set_rb2(mut self, rb2: &'a Player) -> LineupBuilder<'a> {
        self.rb2 = Some(return_if_field_exits(self.rb2, rb2));
        self.total_price += rb2.price as i32;
        self
    }

    pub fn set_wr1(mut self, wr1: &'a Player) -> LineupBuilder<'a> {
        self.wr1 = Some(return_if_field_exits(self.wr1, wr1));
        self.total_price += wr1.price as i32;
        self
    }

    pub fn set_wr2(mut self, wr2: &'a Player) -> LineupBuilder<'a> {
        self.wr2 = Some(return_if_field_exits(self.wr2, wr2));
        self.total_price += wr2.price as i32;
        self
    }

    pub fn set_wr3(mut self, wr3: &'a Player) -> LineupBuilder<'a> {
        self.wr3 = Some(return_if_field_exits(self.wr3, wr3));
        self.total_price += wr3.price as i32;
        self
    }

    pub fn set_te(mut self, te: &'a Player) -> LineupBuilder<'a> {
        self.te = Some(return_if_field_exits(self.te, te));
        self.total_price += te.price as i32;
        self
    }

    pub fn set_flex(mut self, flex: &'a Player) -> LineupBuilder<'a> {
        self.flex = Some(return_if_field_exits(self.flex, flex));
        self.total_price += flex.price as i32;
        self
    }

    pub fn set_def(mut self, def: &'a Player) -> LineupBuilder<'a> {
        self.dst = Some(return_if_field_exits(self.dst, def));
        self.total_price += def.price as i32;
        self
    }

    pub fn build(self) -> Lineup {
        Lineup {
            qb: self.qb.unwrap().clone(),
            rb1: self.rb1.unwrap().clone(),
            rb2: self.rb2.unwrap().clone(),
            wr1: self.wr1.unwrap().clone(),
            wr2: self.wr2.unwrap().clone(),
            wr3: self.wr3.unwrap().clone(),
            te: self.te.unwrap().clone(),
            flex: self.flex.unwrap().clone(),
            def: self.dst.unwrap().clone(),
            total_price: self.total_price,
            score: calculate_lineup_score(&self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO the lineupBuilder init could probably be done with a macro?
    fn create_test_player(points: f32, price: i16, ownership: f32) -> Player {
        Player {
            name: String::from("test"),
            team: String::from("test"),
            opp: String::from("test"),
            points_dollar: 1.0,
            pos_rank: Some(1),
            price: price,
            points: points,
            pos: String::from("RB"),
            ownership: ownership,
        }
    }

    fn create_lineup_vec(
        points: f32,
        price: i16,
        ownership: f32,
        double: bool,
    ) -> Vec<Option<Player>> {
        let mut players: Vec<Option<Player>> = Vec::new();
        if double {
            for _ in 0..4 {
                let player: Player = create_test_player(points, price, ownership);
                players.push(Some(player));
            }
            for _ in 4..9 {
                let player: Player = create_test_player(2.0 * points, 2 * price, 2.0 * ownership);
                players.push(Some(player));
            }
        } else {
            for _ in 0..9 {
                let player: Player = create_test_player(points, price, ownership);
                players.push(Some(player));
            }
        }
        players
    }

    #[test]
    fn test_calculate_total_salary() {
        let p: Vec<Option<Player>> = create_lineup_vec(1.0, 1, 1.0, false);
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
        let test_player: &Player = &create_test_player(1.0, 1, 1.0);
        let empty_lineup = LineupBuilder::new();
        let qb_lineup = empty_lineup.set_qb(&test_player);
        let rb_lineup = qb_lineup.set_rb2(&test_player);
        assert_eq!(rb_lineup.total_price, 2)
    }
    #[test]
    fn test_lineup_averge_functions() {
        let p: Vec<Option<Player>> = create_lineup_vec(6.0, 1, 4.0, true);
        let line_up: LineupBuilder = LineupBuilder {
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
        assert_eq!(line_up.averge_ownership(), 6.2222223);
        assert_eq!(line_up.averge_projected_points(), 9.333333);
    }

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
        assert_eq!(max_line_up.get_points_score(), 1.0);
        assert_eq!(min_line_up.get_points_score(), 0.0);
        assert_eq!(max_line_up.get_ownership_score(), -1.0);
        assert_eq!(min_line_up.get_ownership_score(), 0.0);
        assert_eq!(max_line_up.get_salary_spent_score(), 1.0);
        assert_eq!(min_line_up.get_salary_spent_score(), 0.0);
    }
}