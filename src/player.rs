use serde::{Deserialize, Serialize};

// TODO add max/min variance, high sacks for def,
// TODO throws to end zone
// TODO avg attempts, rec targets need to be pulled from stats
pub struct RbProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points: String,
    pub avg_att: i16,
    pub td: i32,
    pub yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

pub struct QbProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points: f32,
    pub com: f32,
    pub int: f32,
    pub passing_yds: f32,
    pub passing_tds: f32,
    pub rushing_yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

pub struct RecProj {
    pub name: String,
    pub team: String,
    pub opp: String,
    pub points: String,
    pub avg_rec: i16,
    pub avg_tgts: i16,
    pub td: i32,
    pub yds: f32,
    pub salary: i32,
    pub own_per: f32,
}

pub struct DefProj {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerOwn {
    // We will generate our own once we load data into sqlite
    pub id: i16,
    pub name: String,
    pub team: String,
    pub pos: String,
    pub opp: String,
    pub salary: i32,
    pub own_per: f32,
}

// Id 0, player 1, team 2, opp 3, pos 4, salary 5, own 6
impl PlayerOwn {
    pub fn new(record: csv::StringRecord) -> Self {
        PlayerOwn {
            id: 1, // fetch from database
            name: record[1].to_string(),
            team: record[2].to_string(),
            opp: record[3].to_string(),
            pos: record[4].to_string(),
            salary: record[5].parse::<i32>().expect("Salary Missing"),
            own_per: record[6].parse::<f32>().expect("Owner Percentage"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::StringRecord;
    //85546-69531,Jalen Hurts,PHI,NYG,QB,9000,18.6
    #[test]
    fn test_new_from_fd() {
        let test_record: StringRecord = StringRecord::from(vec![
            "85546-69531",
            "Jalen Hurts",
            "PHI",
            "NYG",
            "QB",
            "9000",
            "18.6",
        ]);
        let player: PlayerOwn = PlayerOwn::new(test_record.clone());
        assert_eq!(player.name, test_record[1].to_string());
        assert_eq!(player.team, test_record[2].to_string());
        assert_eq!(player.opp, test_record[3].to_string());
        assert_eq!(player.pos, test_record[4].to_string());
        assert_eq!(
            player.salary,
            test_record[5].parse::<i32>().expect("Missing salary")
        );
        assert_eq!(
            player.own_per,
            test_record[6].parse::<f32>().expect("Missing Own Per")
        );
    }
}
