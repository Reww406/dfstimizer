use crate::models::*;

pub fn build_all_possible_lineups(players: Vec<Player>) -> Vec<LineupBuilder> {
    let mut lineups: Vec<LineupBuilder> = Vec::new();
    players
        .iter()
        .filter(|player| player.pos.to_lowercase() == "qb")
        .for_each(|qb| {
            let lineup_builder: LineupBuilder = LineupBuilder::new();
            lineups.push(lineup_builder.set_qb(qb.clone()))
        });
    add_wr_to_qb_lineups(players, lineups)
}

// Home maybe use .as_ref for creating line ups then we can clone on build..?
// Could just fitler on salary later throw away everything that is over after building all line ups
pub fn add_wr_to_qb_lineups(
    players: Vec<Player>,
    lineups: Vec<LineupBuilder>,
) -> Vec<LineupBuilder> {
    let mut lineup_with_wr1: Vec<LineupBuilder> = Vec::new();
    // First Iteration
    for lineup in lineups {
        players
            .iter()
            .filter(|player| player.pos.to_lowercase() == "wr")
            .for_each(|wr| {
                if (lineup.total_price + wr.price as i32) < SALARY_CAP {
                    lineup_with_wr1.push(
                        LineupBuilder::new()
                            // Duplicating variables
                            .set_qb(lineup.qb.clone().unwrap())
                            .set_wr1(wr.clone()),
                    );
                }
            });
    }
    let mut lineup_with_wr2: Vec<LineupBuilder> = Vec::new();
    for lineup in lineup_with_wr1 {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "wr")
            .filter(|wr2: &&Player| wr2.name != lineup.wr1.as_ref().unwrap().name)
            .filter(|wr2: &&Player| (wr2.price as i32 + lineup.total_price) < SALARY_CAP)
            .for_each(|wr2| {
                lineup_with_wr2.push(
                    LineupBuilder::new()
                        .set_qb(lineup.qb.clone().unwrap())
                        .set_wr1(lineup.wr1.clone().unwrap())
                        .set_wr2(wr2.clone()),
                )
            });
    }
    let mut lineup_with_wr3: Vec<LineupBuilder> = Vec::new();
    for lineup in lineup_with_wr2 {
        players
            .iter()
            .filter(|p: &&Player| p.pos.to_lowercase() == "wr")
            .filter(|wr3: &&Player| {
                wr3.name != lineup.wr2.as_ref().unwrap().name
                    && wr3.name != lineup.wr1.as_ref().unwrap().name
            })
            .filter(|wr3: &&Player| (wr3.price as i32 + lineup.total_price) < SALARY_CAP)
            .for_each(|wr3| {
                lineup_with_wr3.push(
                    LineupBuilder::new()
                        .set_qb(lineup.qb.clone().unwrap())
                        .set_wr1(lineup.wr1.clone().unwrap())
                        .set_wr2(lineup.wr2.clone().unwrap())
                        .set_wr3(wr3.clone()),
                )
            });
    }
    lineup_with_wr3
}

// Tested
pub fn calculate_linup_score(lineup: Lineup) -> f32 {
    let salary_spent_score = lineup.get_salary_spent_score();
    let point_score = lineup.get_points_score();
    let ownership_score = lineup.get_ownership_score();
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
        let lineup = create_test_lineup(MIN_AVG_OWNERSHIP, MAX_POINTS, 6666);
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
