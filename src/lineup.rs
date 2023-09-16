use std::rc::Rc;
use std::vec;

use rusqlite::Connection;

use crate::{
    player::*, return_if_field_exits, ALL_CIELING_MAX_MIN, ALL_PTS_MAX_MIN,
    ALL_PTS_PLUS_MINS_MAX_MIN, ALL_PTS_SAL_MAX_MIN, DATABASE_FILE, DST_RATING_MAX_MIN,
    DST_VEGAS_OPP_TOTAL, QB_AVG_RZ_OP_FILLER, QB_AVG_RZ_OP_MAX_MIN, QB_CIELING, QB_COUNT,
    QB_PTS_PER_SAL, QB_RUSH_ATT_FILLER, QB_RUSH_ATT_MAX_MIN, QB_VEGAS_TOTAL_MAX_MIN,
    QB_WR_PASS_PER_MAX_MIN, RB_ATTS_FILLER, RB_ATTS_MAX_MIN, RB_AVG_REC_YDS, RB_AVG_REC_YDS_FILLER,
    RB_OPP_DEF_MAX_MIN, RB_PTS_PLUS_MINUS, RB_SNAPS_PER_FILLER, RB_VEGAS_TOTAL_MAX_MIN,
    RB_YEAR_CONSISTENCY_FILLER, RB_YEAR_CONSISTENCY_MAX_MIN, TE_OPP_DEF_MAX_MIN, TE_REC_TGT_FILLER,
    TE_VEGAS_TOTAL_MAX_MIN, WR_OPP_DEF_MAX_MIN, WR_RED_ZONE_FILLER, WR_RED_ZONE_MAX_MIN,
    WR_SALARY_MEDIAN, WR_TGT_SHARE_FILLER, WR_TGT_SHARE_MAX_MIN, WR_VEGAS_TOTAL_MAX_MIN,
    WR_YEAR_CONSISTENCY_FILLER, WR_YEAR_CONSISTENCY_MAX_MIN, WR_YEAR_UPSIDE_FILLER,
    WR_YEAR_UPSIDE_MAX_MIN,
};
use crate::{RB_SNAPS_PER_MAX_MIN, TE_REC_TGT_MAX_MIN};

pub const SALARY_CAP: i32 = 59000;
pub const MIN_SAL: i32 = 58000;

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
#[derive(Clone)]

pub struct IslandLB {
    pub mvp: Option<Rc<LitePlayer>>,
    pub first: Option<Rc<LitePlayer>>,
    pub second: Option<Rc<LitePlayer>>,
    pub third: Option<Rc<LitePlayer>>,
    pub fourth: Option<Rc<LitePlayer>>,
    pub salary_used: i32,
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

pub enum Slot {
    Mvp,
    First,
    Second,
    Third,
    Fourth,
    None,
    Flex,
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

pub fn rb_score(rbs: &[&RbProj], conn: &Connection) -> f32 {
    let mut score = 0.0;
    rbs.iter().for_each(|rb| {
        let def_vs_rb = query_def_vs_pos(query_def_id(&rb.opp, conn).unwrap(), &Pos::Rb, &conn);
        let mut inside_score = 0.0;
        inside_score += get_normalized_score(rb.vegas_total, *RB_VEGAS_TOTAL_MAX_MIN) * 2.0;
        inside_score += get_normalized_score(def_vs_rb.pts_given_pg, *RB_OPP_DEF_MAX_MIN) * 1.0;
        inside_score +=
            get_normalized_score_with_filler(rb.avg_att, *RB_ATTS_MAX_MIN, *RB_ATTS_FILLER) * 2.0; // Max 1.5
        inside_score += get_normalized_score_with_filler(
            rb.avg_rec_yds,
            *RB_AVG_REC_YDS,
            *RB_AVG_REC_YDS_FILLER,
        ) * 0.50; // Max 0.5

        inside_score += get_normalized_score(rb.pts_plus_minus_proj, *RB_PTS_PLUS_MINUS) * 2.5; // Max 2
        score += get_normalized_score(inside_score, (8.0, 0.0));
    });
    score
}

pub fn qb_score(qb: &QbProj, conn: &Connection) -> f32 {
    let mut score = 0.0;
    let def_vs_qb = query_def_vs_pos(query_def_id(&qb.opp, conn).unwrap(), &Pos::Qb, &conn);
    score += get_normalized_score(def_vs_qb.pts_given_pg, *RB_OPP_DEF_MAX_MIN) * 1.0;
    score += get_normalized_score_with_filler(
        qb.avg_rush_atts,
        *QB_RUSH_ATT_MAX_MIN,
        *QB_RUSH_ATT_FILLER,
    ) * 0.5;
    score += get_normalized_score_with_filler(
        qb.red_zone_op_pg,
        *QB_AVG_RZ_OP_MAX_MIN,
        *QB_AVG_RZ_OP_FILLER,
    ) * 0.5;
    score += get_normalized_score(qb.vegas_total, *QB_VEGAS_TOTAL_MAX_MIN) * 2.0;
    score += get_normalized_score(qb.cieling_proj, *QB_CIELING) * 1.5;
    score += get_normalized_score(qb.pts_sal_proj, *QB_PTS_PER_SAL) * 2.5;

    let new_score = get_normalized_score(score, (8.0, 0.0));
    new_score
}

pub fn wr_stud_score(wr: &RecProj, conn: &Connection) -> f32 {
    let mut score = 0.0;
    let def_vs_wr = query_def_vs_pos(query_def_id(&wr.opp, conn).unwrap(), &Pos::Wr, &conn);
    score += get_normalized_score(def_vs_wr.pts_given_pg, *WR_OPP_DEF_MAX_MIN) * 1.0;
    score += get_normalized_score_with_filler(
        wr.rec_tgt_share,
        *WR_TGT_SHARE_MAX_MIN,
        *WR_TGT_SHARE_FILLER,
    ) * 2.0;
    score += get_normalized_score_with_filler(
        wr.red_zone_op_pg,
        *WR_RED_ZONE_MAX_MIN,
        *WR_RED_ZONE_FILLER,
    ) * 0.5;
    score += get_normalized_score(wr.cieling_proj, *ALL_CIELING_MAX_MIN) * 2.5;
    score += get_normalized_score(wr.vegas_total, *WR_VEGAS_TOTAL_MAX_MIN) * 2.0;
    get_normalized_score(score, (8.0, 0.0))
}

fn flex_score(flex: &FlexProj, conn: &Connection) -> f32 {
    match flex.pos {
        Pos::Wr => return wr_stud_score(flex.rec_proj.as_ref().unwrap(), conn),
        Pos::Rb => return rb_score(&[flex.rb_proj.as_ref().unwrap()], conn),
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
    score += get_normalized_score(def_vs_te.pts_given_pg, *TE_OPP_DEF_MAX_MIN) * 1.0;
    score += get_normalized_score(te.vegas_total, *TE_VEGAS_TOTAL_MAX_MIN) * 1.0;
    score +=
        get_normalized_score_with_filler(te.rec_tgt_share, *TE_REC_TGT_MAX_MIN, *TE_REC_TGT_FILLER)
            * 2.0;
    score += get_normalized_score(te.pts_sal_proj, *ALL_PTS_SAL_MAX_MIN) * 3.0;
    get_normalized_score(score, (7.0, 0.0))
}

// Points
pub fn dst_score(def: &DefProj) -> f32 {
    let mut score = 0.0;
    score += get_normalized_score(def.vegas_opp_total * -1.0, *DST_VEGAS_OPP_TOTAL) * 1.0;
    score += get_normalized_score(def.pts_plus_minus_proj, *ALL_PTS_PLUS_MINS_MAX_MIN) * 2.0;
    get_normalized_score(score, (3.0, 0.0))
}

pub fn score_stacking(wrs: &[&RecProj], qb: &QbProj) -> f32 {
    for wr in wrs {
        if wr.team == qb.team {
            return 1.0;
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
                Pos::Wr => wr_stud_score(rec_proj, &conn),
                Pos::Te => te_score(rec_proj, &conn),
                _ => panic!("Rec Proj had wrong POS."),
            },
            Proj::RbProj(rb_proj) => rb_score(&[&rb_proj], &conn),
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

impl Lineup {
    pub fn score(&self) -> f32 {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        let scores: Vec<f32> = vec![
            rb_score(&[&self.rb1, &self.rb2], &conn),
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

    pub fn get_cum_ownership(&self) -> f32 {
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
        .iter()
        .sum()
    }

    pub fn wr_scores(wrs: &[&RecProj]) -> f32 {
        let mut score: f32 = 0.0;
        let conn = Connection::open(DATABASE_FILE).unwrap();
        wrs.iter().for_each(|wr| {
            // if wr.salary as f32 > *WR_SALARY_MEDIAN {
            score += wr_stud_score(wr, &conn);
            // } else {
            //     score += wr_value_score(wr);
            // }
        });
        score
    }

    pub fn lineup_str(&self) -> String {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        format!(
            "\nSalary: {} Score: {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            self.salary_used,
            self.score(),
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
                rb_score(&[&self.rb1], &conn)
            ),
            format!(
                "RB2: {} Team: {} Score: {}",
                self.rb2.name,
                self.rb2.team,
                rb_score(&[&self.rb2], &conn)
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

    pub fn print_lineup(&self) {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        println!("Salary: {} Score: {}", self.salary_used, self.score());
        println!(
            "QB: {} Team: {} Score: {}",
            self.qb.name,
            self.qb.team,
            qb_score(&self.qb, &conn)
        );
        println!(
            "RB1: {} Team: {} Score: {}",
            self.rb1.name,
            self.rb1.team,
            rb_score(&[&self.rb1], &conn)
        );
        println!(
            "RB2: {} Team: {} Score: {}",
            self.rb2.name,
            self.rb2.team,
            rb_score(&[&self.rb2], &conn)
        );
        println!(
            "WR1: {} Team: {} Score: {}",
            self.wr1.name,
            self.wr1.team,
            Self::wr_scores(&[&self.wr1])
        );
        println!(
            "WR2: {} Team: {} Score: {}",
            self.wr2.name,
            self.wr2.team,
            Self::wr_scores(&[&self.wr2])
        );
        println!(
            "WR3: {} Team: {} Score: {}",
            self.wr3.name,
            self.wr3.team,
            Self::wr_scores(&[&self.wr3])
        );
        println!(
            "DST: {} Team: {} Score: {}",
            self.def.name,
            self.def.team,
            dst_score(&self.def)
        );
        println!(
            "TE: {} Team: {} Score: {}",
            self.te.name,
            self.te.team,
            te_score(&self.te, &conn)
        );
        match self.flex.pos {
            Pos::Wr => println!(
                "FWR: {} Team: {} Score: {}",
                self.flex.rec_proj.as_ref().expect("").name,
                self.flex.rec_proj.as_ref().expect("").team,
                flex_score(&self.flex, &conn)
            ),
            Pos::Rb => println!(
                "FRB: {} Team: {} Score: {}",
                self.flex.rb_proj.as_ref().expect("").name,
                self.flex.rb_proj.as_ref().expect("").team,
                flex_score(&self.flex, &conn)
            ),
            _ => panic!("Wrong POS For Flex"),
        }
        println!("")
    }
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
