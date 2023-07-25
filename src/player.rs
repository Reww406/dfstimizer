use serde::{Deserialize, Serialize};

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

// Rank 0, Name 1, Team 2, Position 3, Week 4, Opp 5, Opp Rank 6, Opp Pos Rank 7, Proj Points Fanduel 8,
// Points per dollar 9, Project Ownership 10, Operator (Fanduel) 11, Operator Salary 12
impl Player {
    pub fn new_from_fd(record: csv::StringRecord) -> Self {
        Player {
            name: record[1].to_string(),
            team: record[2].to_string(),
            opp: record[5].to_string(),
            // TODO how often is points _dollars blank
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
}