use std::fs::File;
use std::io::Error;
use std::sync::Arc;
use std::vec;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    mean, player::*, return_if_field_exits, ALL_CIELING_MAX_MIN, ALL_FLOOR_MAX_MIN,
    ALL_PTS_MAX_MIN, ALL_PTS_SAL_MAX_MIN, DST_RATING_MAX_MIN, QB_AVG_RZ_OP_FILLER,
    QB_AVG_RZ_OP_MAX_MIN, QB_OWN_PROJ_MAX_MIN, QB_RUSH_ATT_FILLER, QB_RUSH_ATT_MAX_MIN,
    QB_TE_PASS_PER_MEDIAN, QB_VEGAS_TOTAL_MAX_MIN, QB_WR_PASS_PER_MAX_MIN, RB_AVG_REC_YDS,
    RB_OWN_PROJ_MAX_MIN, RB_SNAPS_PER_FILLER, RB_VEGAS_TOTAL_MAX_MIN, RB_YEAR_CONSISTENCY_FILLER,
    RB_YEAR_CONSISTENCY_MAX_MIN, TE_REC_TGT_FILLER, TE_RED_ZONE_FILLER, TE_RED_ZONE_MAX_MIN,
    WR_OWN_PROJ_MAX_MIN, WR_RED_ZONE_FILLER, WR_RED_ZONE_MAX_MIN, WR_SALARY_MEDIAN,
    WR_TGT_SHARE_FILLER, WR_TGT_SHARE_MAX_MIN, WR_VEGAS_TOTAL_MAX_MIN, WR_YEAR_CONSISTENCY_FILLER,
    WR_YEAR_CONSISTENCY_MAX_MIN, WR_YEAR_UPSIDE_FILLER, WR_YEAR_UPSIDE_MAX_MIN,
};
use crate::{RB_ATTS_MAX_MIN, RB_SNAPS_PER_MAX_MIN, TE_REC_TGT_MAX_MIN};

pub const SALARY_CAP: i32 = 60000;
pub const MIN_SAL: i32 = 58700;

// Thursday kicker, 5 flex spots
// maybe dont care about kicker
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
    pub mvp: Option<Arc<LitePlayer>>,
    pub first: Option<Arc<LitePlayer>>,
    pub second: Option<Arc<LitePlayer>>,
    pub third: Option<Arc<LitePlayer>>,
    pub fourth: Option<Arc<LitePlayer>>,
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

pub enum Slot {
    Mvp,
    First,
    Second,
    Third,
    Fourth,
    None,
    Flex,
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
    // TODO Change pts to pts per dollar
    // return if field exits already
    pub fn set_slot(mut self, lite_player: &Arc<LitePlayer>, slot: Slot) -> IslandLB {
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

    fn get_proj_score(proj: &Proj, conn: &Connection) -> f32 {
        let score = match proj {
            Proj::QbProj(qb_proj) => Self::score_qb(qb_proj, &conn),
            Proj::RecProj(rec_proj) => match rec_proj.pos {
                Pos::Wr => Self::score_wr(rec_proj, &conn),
                Pos::Te => Self::score_te(rec_proj, &conn),
                _ => panic!("Rec Proj had wrong POS."),
            },
            Proj::RbProj(rb_proj) => Self::score_rb(rb_proj, &conn),
            Proj::DefProj(def_proj) => Self::score_dst(def_proj, &conn),
            Proj::KickProj(kick_proj) => Self::score_kicker(kick_proj, &conn),
        };
        // if score == 0.0 {
        //     // println!("{} got a zero score", proj.get_name())
        // }
        score
    }

    pub fn score(mvp_proj: &Proj, projs: &[&Proj; 4], salary_used: i32, conn: &Connection) -> f32 {
        let mut total_score = 0.0;
        projs.into_iter().for_each(|p: &&Proj| {
            total_score += Self::get_proj_score(p, conn);
        });
        total_score += Self::get_proj_score(mvp_proj, conn) * 1.5;
        // total_score +=
        //     get_normalized_score(salary_used as f32, (SALARY_CAP as f32, MIN_SAL as f32));
        total_score += Self::get_ownership_score(mvp_proj, projs);
        total_score += Self::score_statcking(mvp_proj, projs);
        total_score
    }

    fn score_rb(proj: &RbProj, conn: &Connection) -> f32 {
        // TODO REC YDS
        let att_score: f32 = get_normalized_score(proj.avg_att, *RB_ATTS_MAX_MIN);
        let rb_floor_pts: f32 = get_normalized_score(proj.floor_proj, *ALL_FLOOR_MAX_MIN);
        let rb_yds_share: f32 = get_normalized_score(proj.snaps_per, *RB_SNAPS_PER_MAX_MIN);
        let rb_rec_yds: f32 = get_normalized_score(proj.avg_rec_yds, *RB_AVG_REC_YDS);

        get_normalized_score(
            att_score + rb_floor_pts + rb_yds_share + rb_rec_yds,
            (4.0, 0.0),
        )
    }
    // TODO Could add QB Score
    fn score_wr(proj: &RecProj, conn: &Connection) -> f32 {
        let tgt_share_score = get_normalized_score(proj.rec_tgt_share, *WR_TGT_SHARE_MAX_MIN);
        let cieling_score = get_normalized_score(proj.cieling_proj, *ALL_CIELING_MAX_MIN);
        let red_zone_score = get_normalized_score(proj.red_zone_op_pg, *WR_RED_ZONE_MAX_MIN);
        get_normalized_score(tgt_share_score + cieling_score + red_zone_score, (3.0, 0.0))
    }

    fn score_te(proj: &RecProj, conn: &Connection) -> f32 {
        let avg_tgt = get_normalized_score(proj.rec_tgt_share, *TE_REC_TGT_MAX_MIN);
        let floor_score: f32 = get_normalized_score(proj.floor_proj, *ALL_FLOOR_MAX_MIN);
        get_normalized_score(avg_tgt + floor_score, (2.0, 0.0))
    }

    fn score_qb(proj: &QbProj, conn: &Connection) -> f32 {
        // let cieling_score = get_normalized_score(proj.cieling_proj, *ALL_CIELING_MAX_MIN);
        // let rush_yds_score = get_normalized_score(proj.avg_rush_yds, *QB_RUSH_ATT_MAX_MIN);
        // let avg_pass_tds = get_normalized_score(proj.avg_pass_tds, *QB_AVG_PASS_TDS_MAX_MIN);
        // let score = get_normalized_score(cieling_score + rush_yds_score + avg_pass_tds, (3.0, 0.0));
        0.0
    }

    fn score_dst(proj: &DefProj, conn: &Connection) -> f32 {
        0.0
    }

    fn score_kicker(proj: &KickProj, conn: &Connection) -> f32 {
        let pts_score = get_normalized_score(proj.pts_proj, *ALL_PTS_MAX_MIN);
        pts_score
    }

    // TODO lets see if we can unnest a couple of these
    fn score_statcking(mvp_proj: &Proj, projs: &[&Proj; 4]) -> f32 {
        let mut score = 0.0;
        let mut all_proj = vec![mvp_proj];
        all_proj.extend(projs);
        projs
            .iter()
            .filter(|p| matches!(p, Proj::QbProj(_)))
            .for_each(|qb: &&Proj| {
                let qb_proj = qb.get_qb_proj();
                projs
                    .iter()
                    .filter(|p| matches!(p, Proj::RecProj(_)))
                    .for_each(|rec| {
                        let rec_proj = rec.get_rec_proj();
                        if rec_proj.pos == Pos::Wr {
                            if rec_proj.team == qb_proj.team {
                                score = 0.8
                            }
                        };
                    })
            });
        score
    }

    fn get_ownership_score(mvp_proj: &Proj, projs: &[&Proj; 4]) -> f32 {
        let mut ownerships = 0.0;
        let mut all_proj = vec![mvp_proj];
        all_proj.extend(projs);
        for proj in all_proj {
            match proj {
                Proj::QbProj(qb_proj) => ownerships += qb_proj.own_proj,
                Proj::RecProj(rec_proj) => ownerships += rec_proj.own_proj,
                Proj::RbProj(rb_proj) => ownerships += rb_proj.own_proj,
                Proj::DefProj(def_proj) => ownerships += def_proj.own_proj,
                Proj::KickProj(k_proj) => ownerships += k_proj.own_proj,
            }
        }
        let averge_ownership: f32 = ownerships / 5.0;
        // TODO is this too much
        0.0
    }

    pub fn build(self, week: i8, season: i16, conn: &Connection) -> IslandLineup {
        let mvp_proj = query_proj(&self.mvp, week, season, conn);
        let first = query_proj(&self.first, week, season, conn);
        let second = query_proj(&self.second, week, season, conn);
        let third = query_proj(&self.third, week, season, conn);
        let fourth = query_proj(&self.fourth, week, season, conn);

        let score = Self::score(
            &mvp_proj,
            &[&first, &second, &third, &fourth],
            self.salary_used,
            &conn,
        );
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

impl IslandLineup {}

impl Lineup {
    pub fn score(&self) -> f32 {
        let scores: Vec<f32> = vec![
            Self::rb_score(&[&self.rb1, &self.rb2]),
            Self::wr_scores(&[&self.wr1, &self.wr2, &self.wr3]),
            self.te_score(),
            self.dst_score(),
            self.qb_score(),
            self.flex_score(),
        ];
        scores.iter().sum()
    }

    //TODO QB pas to rb boost
    pub fn rb_score(rbs: &[&RbProj]) -> f32 {
        let mut score = 0.0;
        rbs.iter().for_each(|rb| {
            let mut inside_score = 0.0;
            inside_score += get_normalized_score(rb.vegas_total, *RB_VEGAS_TOTAL_MAX_MIN);
            inside_score += get_normalized_score_with_filler(
                rb.snaps_per,
                *RB_SNAPS_PER_MAX_MIN,
                *RB_SNAPS_PER_FILLER,
            );
            inside_score += get_normalized_score(rb.avg_rec_yds, *RB_AVG_REC_YDS);
            inside_score += get_normalized_score(rb.pts_sal_proj, *ALL_PTS_SAL_MAX_MIN);
            inside_score += get_normalized_score(rb.floor_proj, *ALL_FLOOR_MAX_MIN);
            inside_score += get_normalized_score_with_filler(
                rb.year_consistency,
                *RB_YEAR_CONSISTENCY_MAX_MIN,
                *RB_YEAR_CONSISTENCY_FILLER,
            );
            inside_score += -0.6 * get_normalized_score(rb.own_proj, *RB_OWN_PROJ_MAX_MIN);
            score += get_normalized_score(inside_score, (6.0, 0.0))
        });
        score
    }

    // Top 5 Stats once we have it
    pub fn qb_score(&self) -> f32 {
        let mut score = 0.0;
        let qb = &self.qb;
        score += get_normalized_score_with_filler(
            qb.avg_rush_atts,
            *QB_RUSH_ATT_MAX_MIN,
            *QB_RUSH_ATT_FILLER,
        );
        score += get_normalized_score_with_filler(
            qb.red_zone_op_pg,
            *QB_AVG_RZ_OP_MAX_MIN,
            *QB_AVG_RZ_OP_FILLER,
        );
        score += get_normalized_score(qb.vegas_total, *QB_VEGAS_TOTAL_MAX_MIN);
        score += get_normalized_score(qb.cieling_proj, *ALL_CIELING_MAX_MIN);
        let wr_passing_score: f32 =
            get_normalized_score(qb.pass_to_wr_per, *QB_WR_PASS_PER_MAX_MIN);

        for wr in [&self.wr1, &self.wr2, &self.wr3] {
            if wr.team == qb.team {
                score += 1.0 + wr_passing_score;
                break;
            }
        }
        score += get_normalized_score(qb.pts_sal_proj, *ALL_PTS_SAL_MAX_MIN);
        score += -1.0 * get_normalized_score(qb.own_proj, *QB_OWN_PROJ_MAX_MIN);
        get_normalized_score(score, (7.0, 0.0))
    }

    pub fn wr_stud_score(wr: &RecProj) -> f32 {
        let mut score = 0.0;
        score += get_normalized_score_with_filler(
            wr.rec_tgt_share,
            *WR_TGT_SHARE_MAX_MIN,
            *WR_TGT_SHARE_FILLER,
        );
        score += get_normalized_score_with_filler(
            wr.red_zone_op_pg,
            *WR_RED_ZONE_MAX_MIN,
            *WR_RED_ZONE_FILLER,
        );
        score += get_normalized_score(wr.cieling_proj, *ALL_CIELING_MAX_MIN);
        score += get_normalized_score(wr.pts_sal_proj, *ALL_PTS_SAL_MAX_MIN);
        score += get_normalized_score(wr.vegas_total, *WR_VEGAS_TOTAL_MAX_MIN);
        score += get_normalized_score_with_filler(
            wr.year_upside,
            *WR_YEAR_UPSIDE_MAX_MIN,
            *WR_YEAR_UPSIDE_FILLER,
        );
        score += -0.6 * get_normalized_score(wr.own_proj, *WR_OWN_PROJ_MAX_MIN);
        get_normalized_score(score, (6.0, 0.0))
    }

    pub fn wr_value_score(wr: &RecProj) -> f32 {
        let mut score = 0.0;
        score += get_normalized_score_with_filler(
            wr.rec_tgt_share,
            *WR_TGT_SHARE_MAX_MIN,
            *WR_TGT_SHARE_FILLER,
        );
        score += get_normalized_score(wr.pts_sal_proj, *ALL_PTS_SAL_MAX_MIN);
        score += get_normalized_score(wr.vegas_total, *WR_VEGAS_TOTAL_MAX_MIN);
        score += get_normalized_score_with_filler(
            wr.year_consistency,
            *WR_YEAR_CONSISTENCY_MAX_MIN,
            *WR_YEAR_CONSISTENCY_FILLER,
        );
        score += -1.0 * get_normalized_score(wr.own_proj, *WR_OWN_PROJ_MAX_MIN);
        get_normalized_score(score, (4.0, 0.0))
    }

    pub fn wr_scores(wrs: &[&RecProj]) -> f32 {
        let mut score: f32 = 0.0;
        wrs.iter().for_each(|wr| {
            if wr.salary as f32 > *WR_SALARY_MEDIAN {
                score += Self::wr_stud_score(wr);
            } else {
                score += Self::wr_value_score(wr);
            }
        });
        score
    }

    fn flex_score(&self) -> f32 {
        match self.flex.pos {
            Pos::Wr => return Self::wr_value_score(&self.flex.rec_proj.as_ref().unwrap()),
            Pos::Rb => return Self::rb_score(&[self.flex.rb_proj.as_ref().unwrap()]),
            _ => {
                panic!("Wrong Flex Pos..");
            }
        }
    }

    pub fn te_score(&self) -> f32 {
        // return get_normalized_score(self.te.tgts, *TE_TGTS_MAX_MIN);
        let mut score = 0.0;
        score += get_normalized_score_with_filler(
            self.te.rec_tgt_share,
            *TE_REC_TGT_MAX_MIN,
            *TE_REC_TGT_FILLER,
        );
        score += get_normalized_score_with_filler(
            self.te.red_zone_op_pg,
            *TE_RED_ZONE_MAX_MIN,
            *TE_RED_ZONE_FILLER,
        );
        if self.qb.pass_to_te_per > *QB_TE_PASS_PER_MEDIAN {
            if self.te.team == self.qb.team {
                score += 0.5;
            }
        }
        score += get_normalized_score(self.te.pts_sal_proj, *ALL_PTS_SAL_MAX_MIN);
        get_normalized_score(score, (3.5, 0.0))
    }

    // Points
    pub fn dst_score(&self) -> f32 {
        let mut score = 0.0;
        score += get_normalized_score(self.def.rating, *DST_RATING_MAX_MIN);
        score += get_normalized_score(self.def.pts_sal_proj, *DST_RATING_MAX_MIN);
        score
    }

    pub fn lineup_str(&self) -> String {
        format!(
            "\nSalary: {} Score: {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            self.salary_used,
            self.score(),
            format!(
                "QB: {} Team: {} Score: {}",
                self.qb.name,
                self.qb.team,
                self.qb_score()
            ),
            format!(
                "RB1: {} Team: {} Score: {}",
                self.rb1.name,
                self.rb1.team,
                Self::rb_score(&[&self.rb1])
            ),
            format!(
                "RB2: {} Team: {} Score: {}",
                self.rb2.name,
                self.rb2.team,
                Self::rb_score(&[&self.rb2])
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
                self.dst_score()
            ),
            format!(
                "TE: {} Team: {} Score: {}",
                self.te.name,
                self.te.team,
                self.te_score()
            ),
            match self.flex.pos {
                Pos::Wr => format!(
                    "FWR: {} Team: {} Score: {}",
                    self.flex.rec_proj.as_ref().expect("").name,
                    self.flex.rec_proj.as_ref().expect("").team,
                    self.flex_score()
                ),
                Pos::Rb => format!(
                    "FRB: {} Team: {} Score: {}",
                    self.flex.rb_proj.as_ref().expect("").name,
                    self.flex.rb_proj.as_ref().expect("").team,
                    self.flex_score()
                ),
                _ => panic!("Wrong POS For Flex"),
            }
        )
    }

    pub fn print_lineup(&self) {
        println!("Salary: {} Score: {}", self.salary_used, self.score());
        println!(
            "QB: {} Team: {} Score: {}",
            self.qb.name,
            self.qb.team,
            self.qb_score()
        );
        println!(
            "RB1: {} Team: {} Score: {}",
            self.rb1.name,
            self.rb1.team,
            Self::rb_score(&[&self.rb1])
        );
        println!(
            "RB2: {} Team: {} Score: {}",
            self.rb2.name,
            self.rb2.team,
            Self::rb_score(&[&self.rb2])
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
            self.dst_score()
        );
        println!(
            "TE: {} Team: {} Score: {}",
            self.te.name,
            self.te.team,
            self.te_score()
        );
        match self.flex.pos {
            Pos::Wr => println!(
                "FWR: {} Team: {} Score: {}",
                self.flex.rec_proj.as_ref().expect("").name,
                self.flex.rec_proj.as_ref().expect("").team,
                self.flex_score()
            ),
            Pos::Rb => println!(
                "FRB: {} Team: {} Score: {}",
                self.flex.rb_proj.as_ref().expect("").name,
                self.flex.rb_proj.as_ref().expect("").team,
                self.flex_score()
            ),
            _ => panic!("Wrong POS For Flex"),
        }
        // println!("FLEX: {} Team: {} Score: {}", self.flex.name, self.flex., self.flex_score());
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
            dst: None,
            salary_used: 0,
        }
    }

    pub fn array_of_players(&self) -> [&Arc<LitePlayer>; 9] {
        [
            self.qb.as_ref().expect("Line up missing qb"),
            self.rb1.as_ref().expect("Line up missing rb1"),
            self.rb2.as_ref().expect("Line up missing rb2"),
            self.wr1.as_ref().expect("Line up missing wr1"),
            self.wr2.as_ref().expect("Line up missing wr2"),
            self.wr3.as_ref().expect("Line up missing wr3"),
            self.te.as_ref().expect("Line up missing te"),
            self.flex.as_ref().expect("Line up missing flex"),
            self.dst.as_ref().expect("Line up missing def"),
        ]
    }

    pub fn get_salary_spent_score(&self) -> f32 {
        let spent = self.total_amount_spent() as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    pub fn total_amount_spent(&self) -> i32 {
        let line_up_array: [&Arc<LitePlayer>; 9] = self.array_of_players();
        line_up_array.into_iter().map(|x| x.salary as i32).sum()
    }
    pub fn set_pos(mut self, lp: &Arc<LitePlayer>, slot: Slot) -> LineupBuilder {
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
            Pos::D => self.dst = Some(return_if_field_exits(self.dst, &lp)),
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
        let def: DefProj = query_def_proj_helper(&self.dst, week, season, conn);
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
