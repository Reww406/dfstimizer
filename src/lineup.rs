use std::cmp::max;
use std::vec;

use rand::Rng;
use rusqlite::Connection;

use crate::{
    player::*, return_if_field_exits, ALL_CIELING_MAX_MIN, ALL_FLOOR_MAX_MIN, ALL_PTS_MAX_MIN,
    ALL_PTS_PLUS_MINS_MAX_MIN, ALL_PTS_SAL_MAX_MIN, ALL_TEAM_TOTAL, DATABASE_FILE,
    DST_PTS_PLUS_MINUS, DST_VEGAS_OPP_TOTAL, QB_AVG_RUSH_YDS, QB_AVG_RZ_OP, QB_AVG_TD, QB_CIELING,
    QB_INVERSE_SAL, QB_OPP_DEF, QB_PTS_PLUS_MINUS, QB_PTS_SAL, RB_ATTS, RB_AVG_REC_TGTS,
    RB_CEILING, RB_INVERSE_SAL, RB_OPP_DEF, RB_PTS_SAL, SALARY_CAP, TE_AVG_TD, TE_CIELING,
    TE_INVERSE_SAL, TE_OPP_DEF, TE_PTS_SAL, TE_UPSIDE, WR_AVG_TD, WR_CIELING, WR_MONTH_UPSIDE,
    WR_OPP_DEF, WR_PTS_SAL, WR_RED_ZONE, WR_TGT_SHARE,
};
use crate::{RB_AVG_TD, TE_REC_TGT};

// first name is min, next number is max
pub const OWN_COUNT_RANGE_3: OwnBracket = OwnBracket {
    own: 2.5,
    max_amount: 2,
    min_amount: 0,
};
pub const OWN_COUNT_RANGE_6: OwnBracket = OwnBracket {
    own: 5.0,
    max_amount: 4,
    min_amount: 1,
};
pub const OWN_COUNT_RANGE_12: OwnBracket = OwnBracket {
    own: 12.0,
    max_amount: 6,
    min_amount: 3,
};
pub const OWN_COUNT_RANGE_22: OwnBracket = OwnBracket {
    own: 22.0,
    max_amount: 9,
    min_amount: 6,
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

pub fn score_player(
    id: i16,
    pos: &Pos,
    week: i8,
    season: i16,
    conn: &Connection,
    any_flex: bool,
) -> f32 {
    match pos {
        Pos::Qb => qb_score(&query_qb_proj(id, week, season, conn).unwrap(), any_flex),
        Pos::Rb => rb_score(
            &[&query_rb_proj(id, week, season, conn).unwrap()],
            any_flex,
            false,
        ),
        Pos::Wr => wr_stud_score(
            &query_rec_proj(id, week, season, pos, conn).unwrap(),
            any_flex,
        ),
        Pos::Te => te_score(
            &query_rec_proj(id, week, season, pos, conn).unwrap(),
            any_flex,
        ),
        Pos::D => dst_score(&query_def_proj(id, week, season, conn).unwrap(), any_flex),
        Pos::K => score_kicker(&query_kick_proj(id, week, season, conn).unwrap()),
    }
}

// TODO Maybe else if score on TD or Yds
// TODO Scorning functions
// TODO performance could precaclulate score for each player and store in cache..
pub fn rb_score(rbs: &[&RbProj], any_flex: bool, _: bool) -> f32 {
    let mut score: f32 = 0.0;
    rbs.iter().for_each(|rb| {
        let mut inside_score: f32 = 0.0;
        inside_score += get_normalized_score(rb.opp_def_pts_given, *RB_OPP_DEF) * 0.5;
        inside_score += get_normalized_score(rb.avg_att, *RB_ATTS) * 0.5;
        inside_score += get_normalized_score(rb.avg_rec_tgts, *RB_AVG_REC_TGTS) * 0.5;
        inside_score += get_normalized_score(rb.avg_td, *RB_AVG_TD) * 1.0;
        inside_score += get_normalized_score(rb.salary as f32 * -1.0, *RB_INVERSE_SAL) * 3.0;
        if any_flex {
            inside_score += 2.0;
            inside_score += get_normalized_score(rb.cieling_proj, *ALL_CIELING_MAX_MIN) * 2.0;
            // lower score
        } else {
            inside_score += get_normalized_score(rb.vegas_team_total, *ALL_TEAM_TOTAL) * 2.0;
            // inside_score += get_normalized_score(rb.pts_sal_proj, *RB_PTS_SAL) * 1.0;
            inside_score += get_normalized_score(rb.cieling_proj, *ALL_CIELING_MAX_MIN) * 3.0;
        }
        if rb.own_proj < 5.5 {
            inside_score = 0.0
        }

        score += get_normalized_score(inside_score, (9.25, 0.0));
    });

    score
}

pub fn qb_score(qb: &QbProj, any_flex: bool) -> f32 {
    let mut score: f32 = 0.0;
    // score += get_normalized_score(qb.opp_def_pts_given, *QB_OPP_DEF) * 1.0;
    score += get_normalized_score(qb.red_zone_op_pg, *QB_AVG_RZ_OP) * 1.00;
    score += get_normalized_score(qb.avg_pass_tds, *QB_AVG_TD) * 1.00;
    score += get_normalized_score(qb.avg_rush_yards, *QB_AVG_RUSH_YDS) * 0.5;
    score += get_normalized_score(qb.salary as f32 * -1.0, *QB_INVERSE_SAL) * 1.2;
    // rush yds
    if any_flex {
        // score += get_normalized_score(qb.cieling_proj, *ALL_CIELING_MAX_MIN) * 3.5;
        score += 2.5
    } else {
        score += get_normalized_score(qb.vegas_team_total, *ALL_TEAM_TOTAL) * 2.0;
        score += get_normalized_score(qb.cieling_proj, *QB_CIELING) * 2.0;
        // score += get_normalized_score(qb.pts_sal_proj, *QB_PTS_SAL) * 1.0;
    }
    if qb.own_proj < 2.5 {
        score = 0.0
    }

    let new_score: f32 = get_normalized_score(score, (8.25, 0.0));
    new_score
}

// Most expensive scoring
pub fn wr_stud_score(wr: &RecProj, any_flex: bool) -> f32 {
    let mut score: f32 = 0.0;
    score += get_normalized_score(wr.opp_def_pts_given, *WR_OPP_DEF) * 0.75;
    score += get_normalized_score(wr.rec_tgt_share, *WR_TGT_SHARE) * 0.5;
    score += get_normalized_score(wr.avg_td, *WR_AVG_TD) * 1.5;
    score += get_normalized_score(wr.red_zone_op_pg, *WR_RED_ZONE) * 0.75;
    if any_flex {
        // score += get_normalized_score(wr.cieling_proj, *ALL_CIELING_MAX_MIN) * 2.5;
        score += 2.5;
    } else {
        score += get_normalized_score(wr.vegas_team_total, *ALL_TEAM_TOTAL) * 2.5;
        score += get_normalized_score(wr.pts_sal_proj, *WR_PTS_SAL) * 2.5;
        score += get_normalized_score(wr.cieling_proj, *WR_CIELING) * 3.5;
    }
    score += get_normalized_score(wr.month_upside, *WR_MONTH_UPSIDE) * 0.5;
    if wr.own_proj < 8.0 {
        score += 0.05
    }

    if wr.own_proj < 3.5 {
        score = 0.0
    }
    get_normalized_score(score, (11.35, 0.0)) + 0.05
}

fn flex_score(flex: &FlexProj) -> f32 {
    match flex.pos {
        Pos::Wr => return wr_stud_score(flex.rec_proj.as_ref().unwrap(), false),
        Pos::Rb => return rb_score(&[flex.rb_proj.as_ref().unwrap()], false, true),
        _ => {
            panic!("Wrong Flex Pos..");
        }
    }
}

// Only included in AnyFlex
pub fn score_kicker(proj: &KickProj) -> f32 {
    let pts_score: f32 = get_normalized_score(proj.pts_plus_minus_proj, *ALL_PTS_MAX_MIN) * 1.0;
    get_normalized_score(pts_score, (1.0, 0.0))
}

pub fn te_score(te: &RecProj, any_flex: bool) -> f32 {
    let mut score: f32 = 0.0;
    // score += get_normalized_score(te.opp_def_pts_given, *TE_OPP_DEF) * 0.25;
    // score += get_normalized_score(te.rec_tgt_share, *TE_REC_TGT) * 1.5;
    score += get_normalized_score(te.avg_td, *TE_AVG_TD) * 1.0;
    score += get_normalized_score(te.month_upside, *TE_UPSIDE) * 0.50;
    score += get_normalized_score(-1.0 * te.salary as f32, *TE_INVERSE_SAL) * 0.5;
    if any_flex {
        // score += get_normalized_score(te.pts_sal_proj, *ALL_PTS_MAX_MIN) * 2.0;
        score += 2.0;
    } else {
        score += get_normalized_score(te.vegas_team_total, *ALL_TEAM_TOTAL) * 1.0;
        score += get_normalized_score(te.pts_sal_proj, *TE_PTS_SAL) * 2.0;
    }
    if te.own_proj < 5.0 {
        return 0.0;
    }

    let score = get_normalized_score(score, (8.0, 0.0));
    score
}

// Points
pub fn dst_score(def: &DefProj, any_flex: bool) -> f32 {
    let mut score: f32 = 0.0;
    if def.own_proj < 1.0 {
        return 0.0;
    }

    score += get_normalized_score(def.vegas_opp_total * -1.0, *DST_VEGAS_OPP_TOTAL) * 1.0;
    if any_flex {
        score += get_normalized_score(def.pts_plus_minus_proj, *ALL_PTS_PLUS_MINS_MAX_MIN) * 1.0;
    } else {
        score += get_normalized_score(def.pts_plus_minus_proj, *DST_PTS_PLUS_MINUS) * 1.0;
    }
    get_normalized_score(score, (3.0, 0.0))
}

pub fn score_stacking(wrs: &[&RecProj], qb: &QbProj) -> f32 {
    let mut score: f32 = 0.0;
    for wr in wrs {
        if wr.team == qb.team {
            let bonus = 0.3 + (get_normalized_score(wr.rec_tgt_share, *WR_TGT_SHARE) * 0.50);
            if bonus > score {
                score = bonus;
            }
        }
    }
    return score;
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

#[derive(Clone)]
pub struct IslandLB {
    pub mvp: Option<LitePlayer>,
    pub first: Option<LitePlayer>,
    pub second: Option<LitePlayer>,
    pub third: Option<LitePlayer>,
    pub fourth: Option<LitePlayer>,
    pub salary_used: i32,
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

    pub fn set_slot(mut self, lite_player: &LitePlayer, slot: Slot) -> IslandLB {
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
        let score: f32 = match proj {
            Proj::QbProj(qb_proj) => qb_score(qb_proj, true),
            Proj::RecProj(rec_proj) => match rec_proj.pos {
                Pos::Wr => wr_stud_score(rec_proj, true),
                Pos::Te => te_score(rec_proj, true),
                _ => panic!("Rec Proj had wrong POS."),
            },
            Proj::RbProj(rb_proj) => rb_score(&[&rb_proj], true, false),
            Proj::DefProj(def_proj) => dst_score(def_proj, true),
            Proj::KickProj(kick_proj) => score_kicker(kick_proj),
        };
        score
    }

    pub fn score(mvp_proj: &Proj, projs: &[&Proj; 4]) -> f32 {
        let mut total_score: f32 = 0.0;
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
        let mvp_proj: Proj = query_proj(self.mvp.as_ref(), week, season, conn);
        let first: Proj = query_proj(self.first.as_ref(), week, season, conn);
        let second: Proj = query_proj(self.second.as_ref(), week, season, conn);
        let third: Proj = query_proj(self.third.as_ref(), week, season, conn);
        let fourth: Proj = query_proj(self.fourth.as_ref(), week, season, conn);

        let score: f32 = Self::score(&mvp_proj, &[&first, &second, &third, &fourth]);

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

#[derive(Debug, Clone)]
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
    pub fn lineup_str(&self, conn: &Connection) -> String {
        format!(
            "Sal: {}, Score: {}\nMVP: {}: {} {}\n{}: {} {}\n{}: {} {}\n{}: {} {}\n{}: {} {}\n\n",
            self.salary_used,
            self.score,
            self.mvp.get_name(conn),
            self.mvp.get_pos().to_str().expect(""),
            self.mvp.get_own(),
            self.first.get_name(conn),
            self.first.get_pos().to_str().expect(""),
            self.first.get_own(),
            self.second.get_name(conn),
            self.second.get_pos().to_str().expect(""),
            self.second.get_own(),
            self.third.get_name(conn),
            self.third.get_pos().to_str().expect(""),
            self.third.get_own(),
            self.fourth.get_name(conn),
            self.fourth.get_pos().to_str().expect(""),
            self.fourth.get_own()
        )
    }

    pub fn get_as_arr(&self) -> [&Proj; 5] {
        [
            &self.mvp,
            &self.first,
            &self.second,
            &self.third,
            &self.fourth,
        ]
    }
}

impl PartialEq for IslandLineup {
    fn eq(&self, other: &Self) -> bool {
        let ids: Vec<i16> = self
            .get_as_arr()
            .iter()
            .map(|x| x.get_id())
            .collect::<Vec<i16>>();
        let other_ids: Vec<i16> = other
            .get_as_arr()
            .iter()
            .map(|x| x.get_id())
            .collect::<Vec<i16>>();
        for id in ids {
            if !other_ids.contains(&id) {
                return false;
            }
        }
        return true;
    }
}

impl PartialEq for Lineup {
    fn eq(&self, other: &Self) -> bool {
        let ids: [i16; 9] = self.get_id_array();
        let other_ids: [i16; 9] = other.get_id_array();
        for id in ids {
            if !other_ids.contains(&id) {
                return false;
            }
        }
        return true;
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
    // TODO transfer to lite player need to add team
    pub fn score(&self) -> f32 {
        let scores: Vec<f32> = vec![
            rb_score(&[&self.rb1, &self.rb2], false, false),
            Self::wr_scores(&[&self.wr1, &self.wr2, &self.wr3]),
            te_score(&self.te, false),
            dst_score(&self.def, false),
            qb_score(&self.qb, false),
            flex_score(&self.flex),
            score_stacking(&[&self.wr1, &self.wr2, &self.wr3], &self.qb),
        ];
        let mut score: f32 = scores.iter().sum();

        // Filter bad lineups
        if self.qb.opp == self.def.team {
            score = 0.0;
        }
        if self.rb1.opp == self.rb2.team {
            score = 0.0;
        }
        score
    }

    pub fn historic_score(&self, week: i8, season: i16, conn: &Connection) -> f32 {
        let ids: [i16; 9] = self.get_id_array();
        let mut score: f32 = 0.0;
        for id in ids {
            score += get_past_score(week, id, season, conn);
        }
        score
    }

    pub fn get_cum_ownership(&self) -> f32 {
        self.get_ownership_arr().iter().sum()
    }

    pub fn get_id_array(&self) -> [i16; 9] {
        let flex_own: i16 = match self.flex.pos {
            Pos::Wr => {
                self.flex
                    .rec_proj
                    .as_ref()
                    .expect("Stored rec under wrong pos")
                    .id
            }
            Pos::Rb => {
                self.flex
                    .rb_proj
                    .as_ref()
                    .expect("Stored rb under wrong pos")
                    .id
            }
            _ => {
                panic!("Wrong POS in Flex..")
            }
        };
        [
            self.qb.id,
            self.rb1.id,
            self.rb2.id,
            self.wr1.id,
            self.wr2.id,
            self.wr3.id,
            self.te.id,
            flex_own,
            self.def.id,
        ]
    }

    pub fn get_ownership_arr(&self) -> [f32; 9] {
        let flex_own: f32 = match self.flex.pos {
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
        // Make sure def is returned last
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
        wrs.iter().for_each(|wr| {
            score += wr_stud_score(wr, false);
        });
        score
    }
    pub fn lineup_str(&self, conn: &Connection) -> String {
        format!(
            "\nSalary: {} Score: {} Cum Own: {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            self.salary_used,
            self.score(),
            self.get_cum_ownership(),
            format!(
                "QB: {} Team: {} Score: {} Own: {}",
                get_player_name(self.qb.id, conn),
                self.qb.team.to_str(),
                qb_score(&self.qb, false),
                self.qb.own_proj
            ),
            format!(
                "RB1: {} Team: {} Score: {} Own: {}",
                get_player_name(self.rb1.id, conn),
                self.rb1.team.to_str(),
                rb_score(&[&self.rb1], false, false),
                self.rb1.own_proj
            ),
            format!(
                "RB2: {} Team: {} Score: {} Own: {}",
                get_player_name(self.rb2.id, conn),
                self.rb2.team.to_str(),
                rb_score(&[&self.rb2], false, false),
                self.rb2.own_proj
            ),
            format!(
                "WR1: {} Team: {} Score: {} Own: {}",
                get_player_name(self.wr1.id, conn),
                self.wr1.team.to_str(),
                Self::wr_scores(&[&self.wr1]),
                self.wr1.own_proj
            ),
            format!(
                "WR2: {} Team: {} Score: {} Own: {}",
                get_player_name(self.wr2.id, conn),
                self.wr2.team.to_str(),
                Self::wr_scores(&[&self.wr2]),
                self.wr2.own_proj
            ),
            format!(
                "WR3: {} Team: {} Score: {} Own: {}",
                get_player_name(self.wr3.id, conn),
                self.wr3.team.to_str(),
                Self::wr_scores(&[&self.wr3]),
                self.wr3.own_proj
            ),
            format!(
                "DST: {} Team: {} Score: {} Own: {}",
                get_player_name(self.def.id, conn),
                self.def.team.to_str(),
                dst_score(&self.def, false),
                self.def.own_proj
            ),
            format!(
                "TE: {} Team: {} Score: {} Own: {}",
                get_player_name(self.te.id, conn),
                self.te.team.to_str(),
                te_score(&self.te, false),
                self.te.own_proj
            ),
            match self.flex.pos {
                Pos::Wr => format!(
                    "FWR: {} Team: {} Score: {} Own: {}",
                    get_player_name(self.flex.rec_proj.as_ref().expect("").id, conn),
                    self.flex.rec_proj.as_ref().expect("").team.to_str(),
                    flex_score(&self.flex),
                    self.flex.rec_proj.as_ref().expect("").own_proj
                ),
                Pos::Rb => format!(
                    "FRB: {} Team: {} Score: {} Own: {}",
                    get_player_name(self.flex.rb_proj.as_ref().expect("").id, conn),
                    self.flex.rb_proj.as_ref().expect("").team.to_str(),
                    flex_score(&self.flex),
                    self.flex.rb_proj.as_ref().expect("").own_proj
                ),
                _ => panic!("Wrong POS For Flex"),
            }
        )
    }
}

#[derive(Clone, Debug, Copy)]
pub struct LineupBuilder {
    pub qb: Option<LitePlayer>,
    pub rb1: Option<LitePlayer>,
    pub rb2: Option<LitePlayer>,
    pub wr1: Option<LitePlayer>,
    pub wr2: Option<LitePlayer>,
    pub wr3: Option<LitePlayer>,
    pub te: Option<LitePlayer>,
    pub flex: Option<LitePlayer>,
    pub def: Option<LitePlayer>,
    pub salary_used: i32,
}

impl LineupBuilder {
    pub fn score_stacking(wrs: &[&LitePlayer], qb: &LitePlayer) -> f32 {
        let mut score: f32 = 0.0;
        for wr in wrs {
            if wr.team == qb.team {
                // TODO Could add reciever target share to liteplayer
                let bonus = 0.3;
                if bonus > score {
                    score = bonus;
                }
            }
        }
        return score;
    }

    pub fn score(&self) -> f32 {
        let mut score: f32 = self.array_of_players().iter().map(|p| p.score).sum();

        // Filter bad lineups
        if self.qb.unwrap().opp == self.def.unwrap().team {
            score = 0.0;
        }
        if self.rb1.unwrap().opp == self.rb2.unwrap().team {
            score = 0.0;
        }
        score
    }

    pub fn get_ownership_arr(&self) -> [f32; 9] {
        // Make sure def is returned last
        [
            self.qb.unwrap().own_proj,
            self.rb1.unwrap().own_proj,
            self.rb2.unwrap().own_proj,
            self.wr1.unwrap().own_proj,
            self.wr2.unwrap().own_proj,
            self.wr3.unwrap().own_proj,
            self.te.unwrap().own_proj,
            self.flex.unwrap().own_proj,
            self.def.unwrap().own_proj,
        ]
    }
    // pub fn historic_score(&self, week: i8, season: i16, conn: &Connection) -> f32 {
    //     let ids: [i16; 9] = self.get_id_array();
    //     let mut score: f32 = 0.0;
    //     for id in ids {
    //         score += get_past_score(week, id, season, conn);
    //     }
    //     score
    // }

    pub fn fits_own_brackets(&self) -> bool {
        let ownerships: [f32; 9] = self.get_ownership_arr();
        for bracket in &OWN_BRACKETS {
            if !Self::fits_own_bracket(bracket, &ownerships) {
                return false;
            }
        }
        true
    }

    fn fits_own_bracket(bracket: &OwnBracket, ownerships: &[f32; 9]) -> bool {
        let mut count: i8 = 0;

        // Filter out defense
        for own in &ownerships[0..8] {
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

    pub fn array_of_players(&self) -> [LitePlayer; 9] {
        [
            self.qb.expect("Line up missing qb"),
            self.rb1.expect("Line up missing rb1"),
            self.rb2.expect("Line up missing rb2"),
            self.wr1.expect("Line up missing wr1"),
            self.wr2.expect("Line up missing wr2"),
            self.wr3.expect("Line up missing wr3"),
            self.te.expect("Line up missing te"),
            self.flex.expect("Line up missing flex"),
            self.def.expect("Line up missing def"),
        ]
    }

    pub fn get_salary_spent_score(&self) -> f32 {
        let spent: f32 = self.total_amount_spent() as f32;
        (spent - 0.0) / (SALARY_CAP as f32 - 0.0)
    }

    pub fn total_amount_spent(&self) -> i32 {
        let line_up_array: [LitePlayer; 9] = self.array_of_players();
        line_up_array.into_iter().map(|x| x.salary as i32).sum()
    }
    pub fn set_pos(mut self, lp: &LitePlayer, slot: Slot) -> LineupBuilder {
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
    pub fn build(self, week: i8, season: i16) -> Result<Lineup, Box<dyn std::error::Error>> {
        let flex: FlexProj = if self.flex.as_ref().unwrap().pos == Pos::Wr {
            FlexProj {
                pos: Pos::Wr,
                rec_proj: Some(get_rec_from_cache(self.flex.unwrap().id)),
                rb_proj: None,
            }
        } else {
            FlexProj {
                pos: Pos::Rb,
                rec_proj: None,
                rb_proj: Some(get_rb_from_cache(self.flex.unwrap().id)),
            }
        };

        let qb: QbProj = get_qb_from_cache(self.qb.unwrap().id);
        let rb1: RbProj = get_rb_from_cache(self.rb1.unwrap().id);
        let rb2: RbProj = get_rb_from_cache(self.rb2.unwrap().id);
        let wr1: RecProj = get_rec_from_cache(self.wr1.unwrap().id);
        let wr2: RecProj = get_rec_from_cache(self.wr2.unwrap().id);
        let wr3: RecProj = get_rec_from_cache(self.wr3.unwrap().id);
        let te: RecProj = get_rec_from_cache(self.te.unwrap().id);
        let flex: FlexProj = flex;
        let def: DefProj = get_def_from_cache(self.def.unwrap().id);
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
        let conn: Connection = Connection::open(DATABASE_FILE).unwrap();
        // Def vs_rb will change this every time.
        if let Some(mut rb) = query_rb_proj(129, 1, 2023, &conn) {
            // rb.vegas_total = ALL_VEGAS_TOTAL.0;
            rb.avg_att = RB_ATTS.0;
            rb.avg_rec_tgts = RB_AVG_REC_TGTS.0;
            rb.pts_plus_minus_proj = RB_CEILING.0;
            println!("High Score {}", rb_score(&[&rb], false, false));
        }
    }

    #[test]
    fn test_score_wr() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        // Def vs_rb will change this every time.
        if let Some(mut wr) = query_rec_proj(1, 1, 2023, &Pos::Wr, &conn) {
            // wr.vegas_total = ALL_VEGAS_TOTAL.0;
            wr.rec_tgt_share = WR_TGT_SHARE.0;
            wr.red_zone_op_pg = WR_RED_ZONE.0;
            wr.cieling_proj = WR_CIELING.0;
            println!("High Score {}", wr_stud_score(&wr, false));
        }
    }

    // #[test]
    // fn test_score_te() {
    //     let conn = Connection::open(DATABASE_FILE).unwrap();
    //     // Def vs_rb will change this every time.
    //     if let Some(mut te) = query_rec_proj(30, 1, 2023, &Pos::Te, &conn) {
    //         // te.vegas_total = ALL_VEGAS_TOTAL.0;
    //         te.rec_tgt_share = TE_REC_TGT.0;
    //         te.pts_sal_proj = TE_PTS_SAL.0;
    //         println!("High Score {}", te_score(&te, false));
    //     }
    // }

    #[test]
    fn test_score_dst() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        if let Some(mut dst) = query_def_proj(15, 1, 2023, &conn) {
            dst.vegas_opp_total = 14.0;
            dst.pts_plus_minus_proj = DST_PTS_PLUS_MINUS.1;
            println!(
                "High Score {} {:?}",
                dst_score(&dst, false),
                *DST_VEGAS_OPP_TOTAL
            );
        }
    }
    #[test]
    fn test_score_qb() {
        let conn = Connection::open(DATABASE_FILE).unwrap();
        if let Some(mut qb) = query_qb_proj(26, 1, 2023, &conn) {
            // qb.vegas_total = ALL_VEGAS_TOTAL.0;
            // qb.avg_rush_atts = QB_RUSH_ATT.0;
            qb.red_zone_op_pg = QB_AVG_RZ_OP.0;
            qb.cieling_proj = QB_CIELING.0;
            qb.pts_sal_proj = QB_PTS_PLUS_MINUS.0;
            println!("High Score {}", qb_score(&qb, false));
        }
    }

    #[test]
    fn test_inverse_salary() {
        let sal1 = get_normalized_score(7200.0 * -1.0, *RB_INVERSE_SAL);
        let sal2 = get_normalized_score(4500.0 * -1.0, *RB_INVERSE_SAL);
        println!("{} {}", sal1, sal2);
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
    fn test_own_arr() {
        let own = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        println!("{:?}", &own[0..8]);
    }

    // #[test]
    // fn test_ownership_bracket() {
    //     let conn = Connection::open(DATABASE_FILE).unwrap();
    //     let week: i8 = 1;
    //     let season: i16 = 2023;
    //     let lineup = Lineup {
    //         qb: query_qb_proj(115, 1, 2023, &conn).expect(""), // 0.3
    //         rb1: query_rb_proj(156, week, season, &conn).unwrap(), //
    //         rb2: query_rb_proj(43, week, season, &conn).unwrap(),
    //         wr1: query_rec_proj(228, week, season, &Pos::Wr, &conn).unwrap(),
    //         wr2: query_rec_proj(204, week, season, &Pos::Wr, &conn).unwrap(),
    //         wr3: query_rec_proj(16, week, season, &Pos::Wr, &conn).unwrap(),
    //         te: query_rec_proj(23, week, season, &Pos::Te, &conn).unwrap(),
    //         flex: FlexProj {
    //             pos: Pos::Wr,
    //             rec_proj: Some(query_rec_proj(35, week, season, &Pos::Wr, &conn).unwrap()),
    //             rb_proj: None,
    //         },
    //         def: query_def_proj(17, week, season, &conn).unwrap(),
    //         salary_used: 60000,
    //     };
    //     println!("{:?}", lineup.get_ownership_arr());
    //     println!("{}", lineup.fits_own_brackets());
    // }
}
