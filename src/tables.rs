use rusqlite::Connection;

use crate::DATABASE_FILE;

fn init_tables() {
    let conn: Connection = Connection::open(DATABASE_FILE).expect("Can't open DB File");
    let player: &str = "
        CREATE TABLE IF NOT EXISTS player (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            pos TEXT NOT NULL,
            UNIQUE(name, team, pos) on CONFLICT REPLACE
        )
    ";

    let rb_proj: &str = "
        CREATE TABLE IF NOT EXISTS rb_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            atts REAL NOT NULL,
            tds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            rec_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let dst_proj: &str = "
        CREATE TABLE IF NOT EXISTS dst_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let qb_proj: &str = "
        CREATE TABLE IF NOT EXISTS qb_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            atts REAL NOT NULL,
            comps REAL NOT NULL,
            ints REAL NOT NULL,
            pass_yds REAL NOT NULL,
            pass_tds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let wr_proj: &str = "
        CREATE TABLE IF NOT EXISTS wr_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            rec REAL NOT NULL,
            tgts REAL NOT NULL,
            tds REAL NOT NULL,
            rec_yds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let te_proj: &str = "
        CREATE TABLE IF NOT EXISTS te_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts REAL NOT NULL,
            rec REAL NOT NULL,
            tgts REAL NOT NULL,
            tds REAL NOT NULL,
            rec_yds REAL NOT NULL,
            rush_yds REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    // id,player,team,opponent,position,salary,ownership
    let ownership: &str = "
        CREATE TABLE IF NOT EXISTS ownership (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pos REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_per REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";
    let tables: [&str; 7] = [
        player, qb_proj, wr_proj, dst_proj, te_proj, rb_proj, ownership,
    ];
    for table in tables {
        conn.execute(table, ()).expect("Could not create table");
    }
}