use std::io::Error;
use std::sync::Arc;
use std::vec;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    get_recent_stat_ceiling_all, mean, DATABASE_FILE, DST_PTS_MAX_MIN, OWN_PER_MAX_MIN,
    QB_ATT_MAX_MIN, QB_AVG_DEPTH_MAX_MIN, QB_PTS_MAX_MIN, QB_RECENT_PTS_CIELING,
    QB_RUSH_YDS_MAX_MIN, RB_ATT_MAX_MIN, RB_AVG_EZ_ATT_MAX_MIN, RB_PTS_MAX_MIN, STAT_SEASON,
    STAT_WEEK, TE_PTS_MAX_MIN, TE_TGTS_MAX_MIN, WEEK, WR_PTS_MAX_MIN, WR_RECENT_PTS_CIELING,
    WR_TDS_MAX_MIN, WR_TGTS_MAX_MIN,
};
use crate::{player::*, return_if_field_exits};

pub const SALARY_CAP: i32 = 59994;

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
fn get_normalized_score(value: f32, max_min: (f32, f32)) -> f32 {
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
        match proj {
            Proj::QbProj(qb_proj) => Self::score_qb(qb_proj, &conn),
            Proj::RecProj(rec_proj) => match rec_proj.pos {
                Pos::Wr => Self::score_wr(rec_proj, &conn),
                Pos::Te => Self::score_te(rec_proj, &conn),
                _ => panic!("Rec Proj had wrong POS."),
            },
            Proj::RbProj(rb_proj) => Self::score_rb(rb_proj, &conn),
            Proj::DefProj(def_proj) => Self::score_dst(def_proj, &conn),
            Proj::KickProj(kick_proj) => panic!("Kicker scoring not implemented yet"),
        }
    }

    pub fn score(mvp_proj: &Proj, projs: &[&Proj; 4], salary_used: i32, conn: &Connection) -> f32 {
        let mut total_score = 0.0;
        projs.into_iter().for_each(|p: &&Proj| {
            total_score += Self::get_proj_score(p, conn);
        });
        total_score += Self::get_proj_score(mvp_proj, conn) * 1.5;
        total_score += get_normalized_score(salary_used as f32, (SALARY_CAP as f32, 45000.0));
        total_score += Self::get_ownership_score(mvp_proj, projs);
        total_score += Self::score_statcking(mvp_proj, projs);
        total_score
    }

    fn score_rb(proj: &RbProj, conn: &Connection) -> f32 {
        let att_score = get_normalized_score(proj.atts, *RB_ATT_MAX_MIN);
        let rb_pts = get_normalized_score(proj.pts, *RB_PTS_MAX_MIN);
        let ez_atts: f32 = get_normalized_score(
            get_recent_avg(
                STAT_SEASON,
                WEEK,
                "rz_atts",
                "rush_rec_stats",
                proj.id,
                &conn,
            ),
            *RB_AVG_EZ_ATT_MAX_MIN,
        );
        return rb_pts + att_score + ez_atts;
    }
    // TODO Could add QB Score
    fn score_wr(proj: &RecProj, conn: &Connection) -> f32 {
        let tgt_score = get_normalized_score(proj.tgts, *WR_TGTS_MAX_MIN);
        let cieling_score = get_normalized_score(
            get_recent_stat_ceiling(
                STAT_SEASON,
                STAT_WEEK,
                "fan_pts",
                "rush_rec_stats",
                proj.id,
                &conn,
            ),
            (*WR_RECENT_PTS_CIELING, 0.0),
        );
        let tds_score = get_normalized_score(proj.td, *WR_TDS_MAX_MIN);
        tgt_score + cieling_score + tds_score
    }

    fn score_te(proj: &RecProj, conn: &Connection) -> f32 {
        let tgt_score = get_normalized_score(proj.tgts, *TE_TGTS_MAX_MIN);
        let proj_score: f32 = get_normalized_score(proj.pts, *TE_PTS_MAX_MIN);
        tgt_score + proj_score
    }

    fn score_qb(proj: &QbProj, conn: &Connection) -> f32 {
        let att_score = get_normalized_score(proj.atts, *QB_ATT_MAX_MIN);
        let rush_yards = get_normalized_score(proj.rush_yds, *QB_RUSH_YDS_MAX_MIN);
        let avg_depth_score: f32 = get_normalized_score(
            get_recent_avg(
                STAT_SEASON,
                STAT_WEEK,
                "avg_depth",
                "qb_stats",
                proj.id,
                &conn,
            ),
            *QB_AVG_DEPTH_MAX_MIN,
        );
        let pts_cieling_score = get_normalized_score(
            get_recent_stat_ceiling(
                STAT_SEASON,
                STAT_WEEK,
                "fan_pts",
                "qb_stats",
                proj.id,
                &conn,
            ),
            (*QB_RECENT_PTS_CIELING, 0.0),
        );
        att_score + rush_yards + avg_depth_score + pts_cieling_score
    }

    fn score_dst(proj: &DefProj, conn: &Connection) -> f32 {
        let pts_score = get_normalized_score(proj.pts, *DST_PTS_MAX_MIN);
        pts_score
    }

    //TODO lets see if we can unnest a couple of these
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
                                score += 1.0;
                                println!("Stacked WR {} {}", rec_proj.name, qb_proj.name)
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
                Proj::QbProj(qb_proj) => ownerships += qb_proj.own_per,
                Proj::RecProj(rec_proj) => ownerships += rec_proj.own_per,
                Proj::RbProj(rb_proj) => ownerships += rb_proj.own_per,
                Proj::DefProj(def_proj) => ownerships += def_proj.own_per,
                Proj::KickProj(_) => todo!(),
            }
        }
        let averge_ownership: f32 = ownerships / 5.0;
        // TODO is this too much
        -3.0 * get_normalized_score(averge_ownership, *OWN_PER_MAX_MIN)
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

// This can be way more DRY
impl Lineup {
    // TODO These scoring criteria is shitty lets think a lot more into this..
    pub fn get_salary_spent_score(&self) -> f32 {
        let spent = self.salary_used as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    pub fn get_ownership_score(&self) -> f32 {
        let averge_ownership: f32 = self.averge_ownership();
        -1.0 * (averge_ownership - OWN_PER_MAX_MIN.1) / (OWN_PER_MAX_MIN.0 - OWN_PER_MAX_MIN.1)
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
            score += get_normalized_score(rb.atts, *RB_ATT_MAX_MIN);
            score += get_normalized_score(rb.pts, *RB_ATT_MAX_MIN)
        });
        score
    }

    // Top 5 Stats once we have it
    pub fn qb_score(&self) -> f32 {
        let atts_score: f32 = get_normalized_score(self.qb.pts, *QB_ATT_MAX_MIN);
        let pts_score: f32 = get_normalized_score(self.qb.pts, *QB_PTS_MAX_MIN);
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
            score += get_normalized_score(wr.tgts, *WR_TGTS_MAX_MIN);
            score += get_normalized_score(wr.pts, *WR_PTS_MAX_MIN);
        });
        score
    }

    fn flex_score(&self) -> f32 {
        match self.flex.pos {
            Pos::Wr => {
                let wr: &RecProj = self.flex.rec_proj.as_ref().unwrap();
                let wr_tgts = get_normalized_score(wr.tgts, *WR_TGTS_MAX_MIN);
                let wr_pts = get_normalized_score(wr.pts, *WR_PTS_MAX_MIN);
                return wr_pts + wr_tgts;
            }
            Pos::Rb => {
                let rb: &RbProj = self.flex.rb_proj.as_ref().unwrap();
                let rb_atts = get_normalized_score(rb.atts, *RB_ATT_MAX_MIN);
                let rb_pts = get_normalized_score(rb.pts, *RB_PTS_MAX_MIN);
                return rb_pts + rb_atts;
            }
            _ => {
                panic!("Wrong Flex Pos..");
            }
        }
    }

    pub fn te_score(&self) -> f32 {
        return get_normalized_score(self.te.tgts, *TE_TGTS_MAX_MIN);
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
