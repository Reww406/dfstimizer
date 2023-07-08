use crate::models::*;

// Try doing one QB at a time? or start off with WR and do one at a time
// try to avoid cloning with references to player instead
// can try using avro or similar format to save memory
// can create vectors with init capacity aswell
pub fn build_all_possible_lineups(players: Vec<Player>) -> Vec<LineupBuilder> {
    let mut lineups: Vec<LineupBuilder> = Vec::new();
    players
        .iter()
        .filter(|player: &&Player| player.pos.to_lowercase() == "qb")
        .for_each(|qb: &Player| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            lineups.push(lineup_builder.set_qb(qb.clone()))
        });
    let wrs_lineups: Vec<LineupBuilder> = add_wrs_to_lineups(&players, lineups);
    let rbs_lineups: Vec<LineupBuilder> = add_rbs_to_lineups(&players, wrs_lineups);
    let te_linesups: Vec<LineupBuilder> = add_te_to_lineups(&players, rbs_lineups);
    // let def_lineups: Vec<LineupBuilder> = add_def_to_lineups(&players, te_linesups);
    te_linesups
}

// Needs to barrow players so it can be passed to the rest of the functions
pub fn add_wrs_to_lineups(
    players: &Vec<Player>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    // The two vectors should be dereferenced once the function ends
    let mut lineups_with_wr1: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups {
        players
            .iter()
            .filter(|player: &&Player| player.pos.to_lowercase() == "wr")
            .for_each(|wr: &Player| lineups_with_wr1.push(lineup.clone().set_wr1(wr.clone())));
    }

    let mut lineups_with_wr2: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups_with_wr1 {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "wr")
            .filter(|wr2: &&Player| wr2.name != lineup.wr1.as_ref().unwrap().name)
            .for_each(|wr2: &Player| lineups_with_wr2.push(lineup.clone().set_wr2(wr2.clone())));
    }

    let mut lineup_with_wr3: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups_with_wr2 {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "wr")
            .filter(|wr3: &&Player| wr3.name != lineup.wr1.as_ref().unwrap().name)
            .filter(|wr3: &&Player| wr3.name != lineup.wr2.as_ref().unwrap().name)
            .for_each(|wr3: &Player| lineup_with_wr3.push(lineup.clone().set_wr3(wr3.clone())));
    }
    lineup_with_wr3
}

pub fn add_rbs_to_lineups(
    players: &Vec<Player>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineups_with_rb1: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "rb")
            .for_each(|rb: &Player| lineups_with_rb1.push(lineup.clone().set_rb1(rb.clone())));
    }

    let mut lineups_with_rb2: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups_with_rb1 {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "rb")
            .filter(|p: &&Player| p.name != lineup.rb1.as_ref().unwrap().name)
            .for_each(|rb2: &Player| lineups_with_rb2.push(lineup.clone().set_rb2(rb2.clone())))
    }
    lineups_with_rb2
}

pub fn add_te_to_lineups(players: &Vec<Player>, lineups: Vec<LineupBuilder>) -> Vec<LineupBuilder> {
    let mut lineups_with_te: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "te")
            .for_each(|te: &Player| lineups_with_te.push(lineup.clone().set_te(te.clone())))
    }
    lineups_with_te
}

pub fn add_def_to_lineups(
    players: &Vec<Player>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineups_with_def: Vec<LineupBuilder> = Vec::new();
    for lineup in lineups {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "def")
            .for_each(|def: &Player| lineups_with_def.push(lineup.clone().set_def(def.clone())));
    }
    lineups_with_def
}

// Tested
pub fn calculate_linup_score(lineup: Lineup) -> f32 {
    let salary_spent_score: f32 = lineup.get_salary_spent_score();
    let point_score: f32 = lineup.get_points_score();
    let ownership_score: f32 = lineup.get_ownership_score();
    ownership_score + point_score + salary_spent_score
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function for creating line ups
    fn create_test_lineup(ownership: f32, points: f32, price: i16) -> Lineup {
        let mut players: Vec<Player> = Vec::with_capacity(9);
        for _ in 0..9 {
            players.push(Player {
                name: String::from("John bob"),
                ownership: ownership,
                points: points,
                price: price,
                pos: String::from("QB"),
            })
        }

        let lineup: Lineup = Lineup {
            def: players[0].clone(),
            qb: players[1].clone(),
            rb1: players[2].clone(),
            rb2: players[3].clone(),
            wr1: players[4].clone(),
            wr2: players[5].clone(),
            wr3: players[6].clone(),
            te: players[7].clone(),
            flex: players[8].clone(),
            total_price: 60000,
        };
        lineup
    }

    #[test]
    fn test_max_score() {
        let lineup: Lineup = create_test_lineup(MIN_AVG_OWNERSHIP, MAX_POINTS, 6666);
        assert_eq!(calculate_linup_score(lineup), 1.9999)
    }

    #[test]
    fn test_min_score() {
        let lineup: Lineup = create_test_lineup(MAX_AVG_OWNERHSIP, MAX_POINTS, 0);
        assert_eq!(calculate_linup_score(lineup), 0.0);
    }

    #[test]
    fn test_scoring() {
        let lineup: Lineup = create_test_lineup(20.0, 25.0, 4000);
        let lineup1: Lineup = create_test_lineup(19.8, 25.5, 4005);
        assert!(calculate_linup_score(lineup) < calculate_linup_score(lineup1));
    }
}
