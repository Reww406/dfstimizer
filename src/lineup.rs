use std::rc::Rc;
use std::time::Instant;
use std::vec;

use rusqlite::Connection;

use crate::{
    player::*, return_if_field_exits, ALL_CIELING_MAX_MIN, ALL_PTS_MAX_MIN,
    ALL_PTS_PLUS_MINS_MAX_MIN, ALL_PTS_SAL_MAX_MIN, ALL_VEGAS_TOTAL, DATABASE_FILE,
    DST_PTS_PLUS_MINUS, DST_VEGAS_OPP_TOTAL, QB_AVG_RZ_OP, QB_AVG_RZ_OP_FILLER, QB_CIELING,
    QB_COUNT, QB_OPP_DEF, QB_PTS_PER_SAL, QB_RUSH_ATT, QB_RUSH_ATT_FILLER, QB_WR_PASS_PER, RB_ATTS,
    RB_ATTS_FILLER, RB_AVG_REC_YDS, RB_AVG_REC_YDS_FILLER, RB_OPP_DEF, RB_PTS_PLUS_MINUS,
    RB_WR_FLEX_PTS_PLUS, RB_YEAR_CONSISTENCY, RB_YEAR_CONSISTENCY_FILLER, TE_OPP_DEF, TE_PTS_SAL,
    TE_REC_TGT_FILLER, WR_CIELING, WR_OPP_DEF, WR_RED_ZONE, WR_RED_ZONE_FILLER, WR_SALARY_MEDIAN,
    WR_TGT_SHARE, WR_TGT_SHARE_FILLER, WR_YEAR_CONSISTENCY, WR_YEAR_CONSISTENCY_FILLER,
    WR_YEAR_UPSIDE, WR_YEAR_UPSIDE_FILLER,
};
use crate::{RB_WR_FLEX_CIELING, TE_REC_TGT};

pub const SALARY_CAP: i32 = 60000;
pub const MIN_SAL: i32 = 59500;

// first name is min, next number is max
pub const OWN_COUNT_RANGE_3: OwnBracket = OwnBracket {
    own: 3.0,
    max_amount: 3,
    min_amount: 0,
};
pub const OWN_COUNT_RANGE_6: OwnBracket = OwnBracket {
    own: 6.5,
    max_amount: 7,
    min_amount: 1,
};
pub const OWN_COUNT_RANGE_12: OwnBracket = OwnBracket {
    own: 12.0,
    max_amount: 8,
    min_amount: 3,
};
pub const OWN_COUNT_RANGE_22: OwnBracket = OwnBracket {
    own: 22.0,
    max_amount: 9,
    min_amount: 4,
};
pub const OWN_COUNT_RANGE_30: OwnBracket = OwnBracket {
    own: 30.0,
    max_amount: 9,
    min_amount: 6,
};

pub const OWN_BRACKETS: [OwnBracket; 5] = [
    OWN_COUNT_RANGE_3,
    OWN_COUNT_RANGE_6,
    OWN_COUNT_RANGE_12,
    OWN_COUNT_RANGE_22,
    OWN_COUNT_RANGE_30,
];
pub enum Slot {
    Mvp,
    First,
    Second,
    Third,
    Fourth,
    None,
    Flex,
}

pub struct OwnBracket {
    own: f32,
    max_amount: i8,
    min_amount: i8,
}

impl Slot {
    pub fn int_to_slot(int: i8) -> Self {
        match int {
            1 => Slot::First,
            2 => Slot::Second,
            3 => Slot::Third,
            4 => Slot::Fourth,
            _ => panic!("Not a slot"),
        }
    }
}

// Scorning functions
pub fn rb_score(rbs: &[&RbProj], conn: &Connection, flex_score: bool) -> f32 {
    let mut score = 0.0;
    rbs.iter().for_each(|rb| {
        let def_vs_rb = query_def_vs_pos(query_def_id(&rb.opp, conn).unwrap(), &Pos::Rb, &conn);
        let mut inside_score = 0.0;
        inside_score += get_normalized_score(rb.vegas_total, *ALL_VEGAS_TOTAL) * 1.5;
        inside_score += get_normalized_score(def_vs_rb.pts_given_pg, *RB_OPP_DEF) * 0.5;
        inside_score +=
            get_normalized_score_with_filler(rb.avg_att, *RB_ATTS, *RB_ATTS_FILLER) * 2.0; // Max 1.5
        inside_score += get_normalized_score_with_filler(
            rb.avg_rec_yds,
            *RB_AVG_REC_YDS,
            *RB_AVG_REC_YDS_FILLER,
        ) * 0.50; // Max 0.5
        if !flex_score {
            inside_score += get_normalized_score(rb.pts_plus_minus_proj, *RB_PTS_PLUS_MINUS) * 2.5;
        // Max 2
        } else {
            inside_score +=
                get_normalized_score(rb.pts_plus_minus_proj, *RB_WR_FLEX_PTS_PLUS) * 2.5;
        }
        score += get_normalized_score(inside_score, (7.0, 0.0));
    });
    score
}

pub fn qb_score(qb: &QbProj, conn: &Connection) -> f32 {
    let mut score = 0.0;
    let def_vs_qb = query_def_vs_pos(query_def_id(&qb.opp, conn).unwrap(), &Pos::Qb, &conn);
    score += get_normalized_score(def_vs_qb.pts_given_pg, *QB_OPP_DEF) * 0.5;
    score +=
        get_normalized_score_with_filler(qb.avg_rush_atts, *QB_RUSH_ATT, *QB_RUSH_ATT_FILLER) * 0.5;
    score +=
        get_normalized_score_with_filler(qb.red_zone_op_pg, *QB_AVG_RZ_OP, *QB_AVG_RZ_OP_FILLER)
            * 1.0;
    score += get_normalized_score(qb.vegas_total, *ALL_VEGAS_TOTAL) * 2.5;
    score += get_normalized_score(qb.cieling_proj, *QB_CIELING) * 1.5;
    score += get_normalized_score(qb.pts_sal_proj, *QB_PTS_PER_SAL) * 2.5;

    let new_score = get_normalized_score(score, (8.5, 0.0));
    new_score
}

pub fn wr_stud_score(wr: &RecProj, conn: &Connection, flex_score: bool) -> f32 {
    let mut score = 0.0;
    let def_vs_wr = query_def_vs_pos(query_def_id(&wr.opp, conn).unwrap(), &Pos::Wr, &conn);
    score += get_normalized_score(def_vs_wr.pts_given_pg, *WR_OPP_DEF) * 0.5;
    score +=
        get_normalized_score_with_filler(wr.rec_tgt_share, *WR_TGT_SHARE, *WR_TGT_SHARE_FILLER)
            * 2.0;
    score += get_normalized_score_with_filler(wr.red_zone_op_pg, *WR_RED_ZONE, *WR_RED_ZONE_FILLER)
        * 0.5;
    if flex_score {
        score += get_normalized_score(wr.cieling_proj, *RB_WR_FLEX_CIELING) * 2.5;
    } else {
        score += get_normalized_score(wr.cieling_proj, *WR_CIELING) * 2.5;
    }
    score += get_normalized_score(wr.vegas_total, *ALL_VEGAS_TOTAL) * 2.5;
    get_normalized_score(score, (8.0, 0.0))
}

fn flex_score(flex: &FlexProj, conn: &Connection) -> f32 {
    match flex.pos {
        Pos::Wr => return wr_stud_score(flex.rec_proj.as_ref().unwrap(), conn, true),
        Pos::Rb => return rb_score(&[flex.rb_proj.as_ref().unwrap()], conn, true),
        _ => {
            panic!("Wrong Flex Pos..");
        }
    }
}

pub fn score_kicker(proj: &KickProj) -> f32 {
    let pts_score = get_normalized_score(proj.pts_plus_minus_proj, *ALL_PTS_MAX_MIN) * 1.0;
    get_normalized_score(pts_score, (1.0, 0.0))
}

pub fn te_score(te: &RecProj, conn: &Connection) -> f32 {
    let mut score = 0.0;
    let def_vs_te = query_def_vs_pos(query_def_id(&te.opp, conn).unwrap(), &Pos::Te, &conn);
    score += get_normalized_score(def_vs_te.pts_given_pg, *TE_OPP_DEF) * 1.0;
    score += get_normalized_score(te.vegas_total, *ALL_VEGAS_TOTAL) * 2.0;
    score +=
        get_normalized_score_with_filler(te.rec_tgt_share, *TE_REC_TGT, *TE_REC_TGT_FILLER) * 2.0;
    score += get_normalized_score(te.pts_sal_proj, *TE_PTS_SAL) * 3.0;
    get_normalized_score(score, (8.0, 0.0))
}

// Points
pub fn dst_score(def: &DefProj) -> f32 {
    let mut score = 0.0;
    score += get_normalized_score(def.vegas_opp_total * -1.0, *DST_VEGAS_OPP_TOTAL) * 2.0;
    score += get_normalized_score(def.pts_plus_minus_proj, *DST_PTS_PLUS_MINUS) * 1.0;
    get_normalized_score(score, (3.0, 0.0))
}

pub fn score_stacking(wrs: &[&RecProj], qb: &QbProj) -> f32 {
    for wr in wrs {
        if wr.team == qb.team {
            return 0.6 + get_normalized_score(qb.pass_to_wr_per, *QB_WR_PASS_PER);
        }
    }
    return 0.0;
}

/// Takes tuple of max: f32, min: f32
pub fn get_normalized_score(value: f32, max_min: (f32, f32)) -> f32 {
    if value > max_min.0 {
        panic!("Value is greater than max: {}", value)
    }
    if value < max_min.1 {
        panic!("Value is less than min: {}", value)
    }
    (value - max_min.1) / (max_min.0 - max_min.1)
}

/// Takes tuple of max: f32, min: f32
fn get_normalized_score_with_filler(value: f32, max_min: (f32, f32), filler: f32) -> f32 {
    if value > max_min.0 {
        panic!("Value is greater than max: {}", value)
    }
    if value < max_min.1 {
        panic!("Value is less than min: {}", value)
    }
    if value == 0.0 {
        return (filler - max_min.1) / (max_min.0 - max_min.1);
    }
    (value - max_min.1) / (max_min.0 - max_min.1)
}

#[derive(Clone)]
pub struct IslandLB {
    pub mvp: Option<Rc<LitePlayer>>,
    pub first: Option<Rc<LitePlayer>>,
    pub second: Option<Rc<LitePlayer>>,
    pub third: Option<Rc<LitePlayer>>,
    pub fourth: Option<Rc<LitePlayer>>,
    pub salary_used: i32,
}

// TODO Find a better way to score when there is two players basically get one and zero
impl IslandLB {
    pub fn new() -> IslandLB {
        IslandLB {
            mvp: None,
            first: None,
            second: None,
            third: None,
            fourth: None,
            salary_used: 0,
        }
    }

    pub fn set_slot(mut self, lite_player: &Rc<LitePlayer>, slot: Slot) -> IslandLB {
        match slot {
            Slot::Mvp => self.mvp = Some(return_if_field_exits(self.mvp, lite_player)),
            Slot::First => self.first = Some(return_if_field_exits(self.first, lite_player)),
            Slot::Second => self.second = Some(return_if_field_exits(self.second, lite_player)),
            Slot::Third => self.third = Some(return_if_field_exits(self.third, lite_player)),
            Slot::Fourth => self.fourth = Some(return_if_field_exits(self.fourth, lite_player)),
            _ => panic!("Not a valid Island Slot"),
        }
        self.salary_used += lite_player.salary as i32;
        self
    }

    fn get_proj_score(proj: &Proj) -> f32 {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let score = match proj {
            Proj::QbProj(qb_proj) => qb_score(qb_proj, &conn),
            Proj::RecProj(rec_proj) => match rec_proj.pos {
                Pos::Wr => wr_stud_score(rec_proj, &conn, true),
                Pos::Te => te_score(rec_proj, &conn),
                _ => panic!("Rec Proj had wrong POS."),
            },
            Proj::RbProj(rb_proj) => rb_score(&[&rb_proj], &conn, true),
            Proj::DefProj(def_proj) => dst_score(def_proj),
            Proj::KickProj(kick_proj) => score_kicker(kick_proj),
        };
        score
    }

    pub fn score(mvp_proj: &Proj, projs: &[&Proj; 4]) -> f32 {
        let mut total_score = 0.0;
        let mut players: Vec<&Proj> = Vec::new();
        players.push(mvp_proj);
        players.extend(projs);

        projs.into_iter().for_each(|p: &&Proj| {
            total_score += Self::get_proj_score(p);
        });
        total_score += Self::get_proj_score(mvp_proj) * 1.5;
        // TODO score stacking
        total_score
    }

    pub fn build(self, week: i8, season: i16, conn: &Connection) -> IslandLineup {
        let mvp_proj = query_proj(&self.mvp, week, season, conn);
        let first = query_proj(&self.first, week, season, conn);
        let second = query_proj(&self.second, week, season, conn);
        let third = query_proj(&self.third, week, season, conn);
        let fourth = query_proj(&self.fourth, week, season, conn);

        let score = Self::score(&mvp_proj, &[&first, &second, &third, &fourth]);

        IslandLineup {
            mvp: mvp_proj,
            first: first,
            second: second,
            third: third,
            fourth: fourth,
            salary_used: self.salary_used,
            score: score,
        }
    }
}

#[derive(Debug)]
pub struct IslandLineup {
    pub mvp: Proj,
    pub first: Proj,
    pub second: Proj,
    pub third: Proj,
    pub fourth: Proj,
    pub salary_used: i32,
    pub score: f32,
}

impl IslandLineup {
    pub fn lineup_str(&self) -> String {
        format!(
            "Sal: {}, Score: {}\nMVP: {}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n\n",
            self.salary_used,
            self.score,
            self.mvp.get_name(),
            self.mvp.get_proj_pos().to_str().expect(""),
            self.first.get_name(),
            self.first.get_proj_pos().to_str().expect(""),
            self.second.get_name(),
            self.second.get_proj_pos().to_str().expect(""),
            self.third.get_name(),
            self.third.get_proj_pos().to_str().expect(""),
            self.fourth.get_name(),
            self.fourth.get_proj_pos().to_str().expect(""),
        )
    }
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
    pub salary_used: i32,
}

impl Lineup {
    pub fn score(&self) -> f32 {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let scores: Vec<f32> = vec![
            rb_score(&[&self.rb1, &self.rb2], &conn, false),
            Self::wr_scores(&[&self.wr1, &self.wr2, &self.wr3]),
            te_score(&self.te, &conn),
            dst_score(&self.def),
            qb_score(&self.qb, &conn),
            flex_score(&self.flex, &conn),
            score_stacking(&[&self.wr1, &self.wr2, &self.wr3], &self.qb),
        ];
        // TODO stacking
        scores.iter().sum()
    }

    pub fn fits_own_brackets(&self) -> bool {
        let ownerships = self.get_ownership_arr();
        for bracket in &OWN_BRACKETS {
            if !Self::fits_own_bracket(bracket, &ownerships) {
                return false;
            }
        }
        true
    }

    fn fits_own_bracket(bracket: &OwnBracket, ownerships: &[f32; 9]) -> bool {
        let mut count = 0;
        for own in ownerships {
            if own < &bracket.own {
                count += 1;
            }
        }
        if count > bracket.max_amount {
            return false;
        }
        if count < bracket.min_amount {
            return false;
        }
        true
    }

    pub fn get_cum_ownership(&self) -> f32 {
        self.get_ownership_arr().iter().sum()
    }

    pub fn get_ownership_arr(&self) -> [f32; 9] {
        let flex_own = match self.flex.pos {
            Pos::Wr => {
                self.flex
                    .rec_proj
                    .as_ref()
                    .expect("Stored rec under wrong pos")
                    .own_proj
            }
            Pos::Rb => {
                self.flex
                    .rb_proj
                    .as_ref()
                    .expect("Stored rb under wrong pos")
                    .own_proj
            }
            _ => {
                panic!("Wrong POS in Flex..")
            }
        };
        [
            self.qb.own_proj,
            self.rb1.own_proj,
            self.rb2.own_proj,
            self.wr1.own_proj,
            self.wr2.own_proj,
            self.wr3.own_proj,
            self.te.own_proj,
            flex_own,
            self.def.own_proj,
        ]
    }

    pub fn wr_scores(wrs: &[&RecProj]) -> f32 {
        let mut score: f32 = 0.0;
        let conn = Connection::open(DATABASE_FILE).unwrap();
        wrs.iter().for_each(|wr| {
            score += wr_stud_score(wr, &conn, false);
        });
        score
    }

    pub fn lineup_str(&self) -> String {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        format!(
            "\nSalary: {} Score: {} Cum Own: {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            self.salary_used,
            self.score(),
            self.get_cum_ownership(),
            format!(
                "QB: {} Team: {} Score: {}",
                self.qb.name,
                self.qb.team,
                qb_score(&self.qb, &conn)
            ),
            format!(
                "RB1: {} Team: {} Score: {}",
                self.rb1.name,
                self.rb1.team,
                rb_score(&[&self.rb1], &conn, false)
            ),
            format!(
                "RB2: {} Team: {} Score: {}",
                self.rb2.name,
                self.rb2.team,
                rb_score(&[&self.rb2], &conn, false)
            ),
            format!(
                "WR1: {} Team: {} Score: {}",
                self.wr1.name,
                self.wr1.team,
                Self::wr_scores(&[&self.wr1])
            ),
            format!(
                "WR2: {} Team: {} Score: {}",
                self.wr2.name,
                self.wr2.team,
                Self::wr_scores(&[&self.wr2])
            ),
            format!(
                "WR3: {} Team: {} Score: {}",
                self.wr3.name,
                self.wr3.team,
                Self::wr_scores(&[&self.wr3])
            ),
            format!(
                "DST: {} Team: {} Score: {}",
                self.def.name,
                self.def.team,
                dst_score(&self.def)
            ),
            format!(
                "TE: {} Team: {} Score: {}",
                self.te.name,
                self.te.team,
                te_score(&self.te, &conn)
            ),
            match self.flex.pos {
                Pos::Wr => format!(
                    "FWR: {} Team: {} Score: {}",
                    self.flex.rec_proj.as_ref().expect("").name,
                    self.flex.rec_proj.as_ref().expect("").team,
                    flex_score(&self.flex, &conn)
                ),
                Pos::Rb => format!(
                    "FRB: {} Team: {} Score: {}",
                    self.flex.rb_proj.as_ref().expect("").name,
                    self.flex.rb_proj.as_ref().expect("").team,
                    flex_score(&self.flex, &conn)
                ),
                _ => panic!("Wrong POS For Flex"),
            }
        )
    }
}

// TODO try implementing copy instead of RC
#[derive(Clone, Debug)]
pub struct LineupBuilder {
    pub qb: Option<Rc<LitePlayer>>,
    pub rb1: Option<Rc<LitePlayer>>,
    pub rb2: Option<Rc<LitePlayer>>,
    pub wr1: Option<Rc<LitePlayer>>,
    pub wr2: Option<Rc<LitePlayer>>,
    pub wr3: Option<Rc<LitePlayer>>,
    pub te: Option<Rc<LitePlayer>>,
    pub flex: Option<Rc<LitePlayer>>,
    pub def: Option<Rc<LitePlayer>>,
    pub salary_used: i32,
}

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
            salary_used: 0,
        }
    }

    pub fn array_of_players(&self) -> [&Rc<LitePlayer>; 9] {
        [
            self.qb.as_ref().expect("Line up missing qb"),
            self.rb1.as_ref().expect("Line up missing rb1"),
            self.rb2.as_ref().expect("Line up missing rb2"),
            self.wr1.as_ref().expect("Line up missing wr1"),
            self.wr2.as_ref().expect("Line up missing wr2"),
            self.wr3.as_ref().expect("Line up missing wr3"),
            self.te.as_ref().expect("Line up missing te"),
            self.flex.as_ref().expect("Line up missing flex"),
            self.def.as_ref().expect("Line up missing def"),
        ]
    }

    pub fn get_salary_spent_score(&self) -> f32 {
        let spent = self.total_amount_spent() as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    pub fn total_amount_spent(&self) -> i32 {
        let line_up_array: [&Rc<LitePlayer>; 9] = self.array_of_players();
        line_up_array.into_iter().map(|x| x.salary as i32).sum()
    }
    pub fn set_pos(mut self, lp: &Rc<LitePlayer>, slot: Slot) -> LineupBuilder {
        match lp.pos {
            Pos::Qb => self.qb = Some(return_if_field_exits(self.qb, &lp)),
            Pos::Rb => match slot {
                Slot::First => self.rb1 = Some(return_if_field_exits(self.rb1, &lp)),
                Slot::Second => self.rb2 = Some(return_if_field_exits(self.rb2, &lp)),
                Slot::Flex => self.flex = Some(return_if_field_exits(self.flex, &lp)),
                _ => panic!("Bad RB Slot"),
            },
            Pos::Wr => match slot {
                Slot::First => self.wr1 = Some(return_if_field_exits(self.wr1, &lp)),
                Slot::Second => self.wr2 = Some(return_if_field_exits(self.wr2, &lp)),
                Slot::Third => self.wr3 = Some(return_if_field_exits(self.wr3, &lp)),
                Slot::Flex => self.flex = Some(return_if_field_exits(self.flex, &lp)),
                _ => panic!("Bad WR Slot"),
            },
            Pos::Te => self.te = Some(return_if_field_exits(self.te, &lp)),
            Pos::D => self.def = Some(return_if_field_exits(self.def, &lp)),
            Pos::K => panic!("No kicker in regular optimizer."),
        }
        self.salary_used += lp.salary as i32;
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

        let qb: QbProj = query_qb_proj_helper(&self.qb, week, season, conn);
        let rb1: RbProj = query_rb_proj_helper(&self.rb1, week, season, conn);
        let rb2: RbProj = query_rb_proj_helper(&self.rb2, week, season, conn);
        let wr1: RecProj = query_rec_proj_helper(&self.wr1, week, season, &Pos::Wr, conn);
        let wr2: RecProj = query_rec_proj_helper(&self.wr2, week, season, &Pos::Wr, conn);
        let wr3: RecProj = query_rec_proj_helper(&self.wr3, week, season, &Pos::Wr, conn);
        let te: RecProj = query_rec_proj_helper(&self.te, week, season, &Pos::Te, conn);
        let flex: FlexProj = flex;
        let def: DefProj = query_def_proj_helper(&self.def, week, season, conn);
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
            salary_used: self.salary_used,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::WR_CIELING;

    use super::*;

    #[test]
    fn test_score_rb() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        // Def vs_rb will change this every time.
        if let Some(mut rb) = query_rb_proj(129, 1, 2023, &conn) {
            rb.vegas_total = ALL_VEGAS_TOTAL.0;
            rb.avg_att = RB_ATTS.0;
            rb.avg_rec_yds = RB_AVG_REC_YDS.0;
            rb.pts_plus_minus_proj = RB_PTS_PLUS_MINUS.0;
            println!("High Score {}", rb_score(&[&rb], &conn, false));
        }
    }

    #[test]
    fn test_score_wr() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        // Def vs_rb will change this every time.
        if let Some(mut wr) = query_rec_proj(1, 1, 2023, &Pos::Wr, &conn) {
            wr.vegas_total = ALL_VEGAS_TOTAL.0;
            wr.rec_tgt_share = WR_TGT_SHARE.0;
            wr.red_zone_op_pg = WR_RED_ZONE.0;
            wr.cieling_proj = WR_CIELING.0;
            println!("High Score {}", wr_stud_score(&wr, &conn, false));
        }
    }

    #[test]
    fn test_score_te() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        // Def vs_rb will change this every time.
        if let Some(mut te) = query_rec_proj(30, 1, 2023, &Pos::Te, &conn) {
            te.vegas_total = ALL_VEGAS_TOTAL.0;
            te.rec_tgt_share = TE_REC_TGT.0;
            te.pts_sal_proj = TE_PTS_SAL.0;
            println!("High Score {}", te_score(&te, &conn));
        }
    }

    #[test]
    fn test_score_dst() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        if let Some(mut dst) = query_def_proj(15, 1, 2023, &conn) {
            dst.vegas_opp_total = 14.0;
            dst.pts_plus_minus_proj = DST_PTS_PLUS_MINUS.1;
            println!("High Score {} {:?}", dst_score(&dst), *DST_VEGAS_OPP_TOTAL);
        }
    }
    #[test]
    fn test_score_qb() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        if let Some(mut qb) = query_qb_proj(26, 1, 2023, &conn) {
            qb.vegas_total = ALL_VEGAS_TOTAL.0;
            qb.avg_rush_atts = QB_RUSH_ATT.0;
            qb.red_zone_op_pg = QB_AVG_RZ_OP.0;
            qb.cieling_proj = QB_CIELING.0;
            qb.pts_sal_proj = QB_PTS_PER_SAL.0;
            println!("High Score {}", qb_score(&qb, &conn));
        }
    }

    #[test]
    fn test_score_stacking() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let qb = query_qb_proj(26, 1, 2023, &conn).unwrap();
        let on_team = query_rec_proj(29, 1, 2023, &Pos::Wr, &conn).unwrap();
        let off_team = query_rec_proj(1, 1, 2023, &Pos::Wr, &conn).unwrap();

        assert!(score_stacking(&[&on_team, &on_team, &on_team], &qb) > 1.0);
        assert!(score_stacking(&[&off_team, &off_team, &off_team], &qb) == 0.0);
    }

    #[test]
    fn test_ownership_bracket() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let week: i8 = 1;
        let season: i16 = 2023;
        let lineup = Lineup {
            qb: query_qb_proj(115, 1, 2023, &conn).expect(""), // 0.3
            rb1: query_rb_proj(156, week, season, &conn).unwrap(), //
            rb2: query_rb_proj(43, week, season, &conn).unwrap(),
            wr1: query_rec_proj(228, week, season, &Pos::Wr, &conn).unwrap(),
            wr2: query_rec_proj(204, week, season, &Pos::Wr, &conn).unwrap(),
            wr3: query_rec_proj(16, week, season, &Pos::Wr, &conn).unwrap(),
            te: query_rec_proj(23, week, season, &Pos::Te, &conn).unwrap(),
            flex: FlexProj {
                pos: Pos::Wr,
                rec_proj: Some(query_rec_proj(35, week, season, &Pos::Wr, &conn).unwrap()),
                rb_proj: None,
            },
            def: query_def_proj(17, week, season, &conn).unwrap(),
            salary_used: 60000,
        };
        println!("{:?}", lineup.get_ownership_arr());
        println!("{}", lineup.fits_own_brackets());
    }
}
