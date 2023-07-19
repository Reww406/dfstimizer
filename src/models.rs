use serde::{Deserialize, Serialize};

use crate::{mean, optimizer::calculate_lineup_score};

pub const SALARY_CAP: i32 = 60000;
pub const MAX_AVG_OWNERHSIP: f32 = 60.0;
pub const MIN_AVG_OWNERSHIP: f32 = 1.0;
pub const MAX_POINTS: f32 = 40.0;
pub const MIN_POINTS: f32 = 10.0;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points_dollar: f32,
    // Higher the better
    pub pos_rank: Option<i16>,
    pub price: i16,
    pub points: f32,
    pub pos: String,
    pub ownership: f32,
}

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

// Builder Helper function
fn return_if_field_exits<'a>(field: Option<&'a Player>, set_to: &'a Player) -> &'a Player {
    if field.is_some() {
        panic!("Tried to set {} when one already exits", set_to.pos);
    }
    set_to
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

// Rank 0, Name 1, Team 2, Position 3, Week 4, Opp 5, Opp Rank 6, Opp Pos Rank 7, Proj Points Fanduel 8,
// Points per dollar 9, Project Ownership 10, Operator (Fanduel) 11, Operator Salary 12
impl Player {
    pub fn new_from_fd(record: csv::StringRecord) -> Self {
        Player {
            name: record[1].to_string(),
            team: record[2].to_string(),
            opp: record[5].to_string(),
            // TODO what is causing this to fail
            points_dollar: record[9].parse::<f32>().unwrap_or_default(),
            pos_rank: if record[7].to_string() == "null" {
                None
            } else {
                Some(record[7].parse::<i16>().expect("Failed to get pos_rank"))
            },
            price: record[12].parse::<i16>().expect("Failed to get price"),
            points: record[8].parse::<f32>().unwrap_or_default(),
            pos: record[3].to_string(),
            ownership: record[10].parse::<f32>().expect("Failed to get ownership"),
        }
    }
}
// TODO Test points score
// TODO Test salary score
// TODO Test ownership score

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;
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
    #[test]
    fn test_new_from_fd() {
        let test_record: StringRecord = StringRecord::from(vec![
            "1",               //0
            "Jonathan Taylor", //1
            "IND",             //2
            "RB",              //3
            "1",               //4
            "HOU",             //5
            "29",              //6
            "28",              //7
            "19.57",           //8
            "1.92",            //9
            "16",              //10
            "FanDuel",         //11
            "10200",           //12
        ]);
        let player: Player = Player::new_from_fd(test_record.clone());
        assert_eq!(player.name, test_record[1].to_string());
        assert_eq!(player.opp, test_record[5].to_string());
        assert_eq!(player.ownership, test_record[10].parse::<f32>().unwrap());
        assert_eq!(player.points, test_record[8].parse::<f32>().unwrap());
        assert_eq!(player.points_dollar, test_record[9].parse::<f32>().unwrap());
        assert_eq!(player.pos, test_record[3].to_string());
        assert_eq!(
            player.pos_rank,
            Some(test_record[7].parse::<i16>().unwrap())
        );
        assert_eq!(player.price, test_record[12].parse::<i16>().unwrap());
        assert_eq!(player.team, test_record[2].to_string())
    }

    #[test]
    fn test_calculate_total_salary() {
        let test_player: Player = create_test_player(1.0, 1, 1.0);
        let test_lineup: LineupBuilder<'_> = LineupBuilder {
            qb: Some(&test_player),
            rb1: Some(&test_player),
            rb2: Some(&test_player),
            wr1: Some(&test_player),
            wr2: Some(&test_player),
            wr3: Some(&test_player),
            te: Some(&test_player),
            flex: Some(&test_player),
            dst: Some(&test_player),
            total_price: 2,
        };
        assert_eq!(test_lineup.total_amount_spent(), 9);
    }

    #[test]
    fn test_lineup_builder_set_functions() {
        let test_player: Player = create_test_player(1.0, 1, 1.0);
        let empty_lineup = LineupBuilder::new();
        let qb_lineup = empty_lineup.set_qb(&test_player);
        let rb_lineup = qb_lineup.set_rb2(&test_player);
        assert_eq!(rb_lineup.total_price, 2)
    }
    #[test]
    fn test_lineup_averge_functions() {
        let mut p: Vec<Option<Player>> = Vec::new();
        for _ in 0..4 {
            p.push(Some(create_test_player(2.0, 1, 4.0)));
        }
        for _ in 0..5 {
            p.push(Some(create_test_player(12.0, 1, 8.0)));
        }
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
        assert_eq!(line_up.averge_projected_points(), 7.5555556);
    }
}
