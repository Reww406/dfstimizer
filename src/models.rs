use serde::{Deserialize, Serialize};

use crate::mean;

pub const SALARY_CAP: i32 = 60000;
pub const MAX_AVG_OWNERHSIP: f32 = 60.0;
pub const MIN_AVG_OWNERSHIP: f32 = 1.0;
pub const MAX_POINTS: f32 = 40.0;
pub const MIN_POINTS: f32 = 10.0;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player {
    pub name: String,
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
    pub def: Option<&'a Player>,
    pub total_price: i32,
}

#[derive(Deserialize, Serialize, Debug)]
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
}

impl Lineup {
    pub fn array_of_players(&self) -> [&Player; 9] {
        [
            &self.qb, &self.rb1, &self.rb2, &self.wr1, &self.wr2, &self.wr3, &self.te, &self.flex,
            &self.def,
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
}

// Builder Helper function
fn return_if_field_exits(field: Option<Player>, set_to: Player) -> Player {
    if field.is_some() {
        panic!("Tried to set {} when one already exits", set_to.pos);
    }
    set_to
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
            def: None,
            total_price: 0,
        }
    }

    // Should check for going over slary cap be here?
    pub fn set_qb(mut self, qb: Player) -> LineupBuilder {
        self.qb = Some(return_if_field_exits(self.qb, qb.clone()));
        self.total_price += qb.price as i32;
        self
    }
    pub fn set_rb1(mut self, rb1: Player) -> LineupBuilder {
        self.rb1 = Some(return_if_field_exits(self.rb1, rb1.clone()));
        self.total_price += rb1.price as i32;
        self
    }
    pub fn set_rb2(mut self, rb2: Player) -> LineupBuilder {
        self.rb2 = Some(return_if_field_exits(self.rb2, rb2.clone()));
        self.total_price += rb2.price as i32;
        self
    }
    pub fn set_wr1(mut self, wr1: Player) -> LineupBuilder {
        self.wr1 = Some(return_if_field_exits(self.wr1, wr1.clone()));
        self.total_price += wr1.price as i32;
        self
    }
    pub fn set_wr2(mut self, wr2: Player) -> LineupBuilder {
        self.wr2 = Some(return_if_field_exits(self.wr2, wr2.clone()));
        self.total_price += wr2.price as i32;
        self
    }
    pub fn set_wr3(mut self, wr3: Player) -> LineupBuilder {
        self.wr3 = Some(return_if_field_exits(self.wr3, wr3.clone()));
        self.total_price += wr3.price as i32;
        self
    }
    pub fn set_te(mut self, te: Player) -> LineupBuilder {
        self.te = Some(return_if_field_exits(self.te, te.clone()));
        self.total_price += te.price as i32;
        self
    }
    pub fn set_flex(mut self, flex: Player) -> LineupBuilder {
        self.flex = Some(return_if_field_exits(self.flex, flex.clone()));
        self.total_price += flex.price as i32;
        self
    }
    pub fn set_def(mut self, def: Player) -> LineupBuilder {
        self.def = Some(return_if_field_exits(self.def, def.clone()));
        self.total_price += def.price as i32;
        self
    }
    pub fn build(self) -> Lineup {
        Lineup {
            qb: self.qb.unwrap(),
            rb1: self.rb1.unwrap(),
            rb2: self.rb2.unwrap(),
            wr1: self.wr1.unwrap(),
            wr2: self.wr2.unwrap(),
            wr3: self.wr3.unwrap(),
            te: self.te.unwrap(),
            flex: self.flex.unwrap(),
            def: self.def.unwrap(),
            total_price: self.total_price,
        }
    }
}

impl Player {
    pub fn new(name: String, price: i16, points: f32, pos: String, ownership: f32) -> Self {
        Player {
            name,
            price,
            points,
            pos,
            ownership,
        }
    }
}
