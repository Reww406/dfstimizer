use crate::models::Player;
use csv::Writer;
use rand::Rng;
use std::error::Error;
use std::fs;

// Find out why closures take &&
// kept for functional reminder
fn find_min(players: &Vec<Player>) -> i16 {
    players
        .iter()
        .filter(|x: &&Player| x.price > 0)
        .min_by_key(|x: &&Player| x.price)
        .unwrap()
        .price
}

// String is mutable and on the heap
// &str is on the stack and immutable
// TODO what does 'a do
#[derive(serde::Serialize)]
struct CsvRow<'a> {
    week: i8,
    oppt: &'a str,
    price: i32,
    points: f32,
    name: &'a str,
    team: &'a str,
    pos: &'a str,
    ownership: f32,
}

fn get_rand_ownership() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..50.0)
}

// week,Oppt,DKP,Price,FDP,Price_f,YHP,Price_y,Name,Team,Pos
pub fn generate_ownership_data() -> Result<(), Box<dyn Error>> {
    let contents: String = fs::read_to_string("dfs_data.csv").expect("Failed to read in file");
    let mut rdr: csv::Reader<&[u8]> = csv::Reader::from_reader(contents.as_bytes());
    let mut writer: Writer<fs::File> = Writer::from_path("with_ownership.scv").unwrap();

    for record in rdr.records() {
        let record: csv::StringRecord = record.unwrap();
        let csv_row: CsvRow = CsvRow {
            week: record[0].parse::<i8>().unwrap_or_default(),
            oppt: &record[1],
            price: record[5].parse::<i32>().unwrap_or_default(),
            points: record[4].parse::<f32>().unwrap_or_default(),
            name: &record[8],
            team: &record[9],
            pos: &record[10],
            ownership: get_rand_ownership(),
        };
        writer.serialize(csv_row).expect("Failed to serialize");
        writer.flush().expect("Failed to flush");
    }
    Ok(())
}
