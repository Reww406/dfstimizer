use rusqlite::Connection;

use crate::DATABASE_FILE;

pub fn init_tables() {
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
            pts_proj REAL NOT NULL,
            cieling_proj REAL NOT NULL,
            floor_proj REAL NOT NULL,
            pts_plus_minus_proj REAL NOT NULL,
            pts_sal_proj REAL NOT NULL,
            vegas_total REAL NOT NULL,
            rush_yds_share REAL NOT NULL,
            avg_atts REAL NOT NULL,
            avg_td REAL NOT NULL,
            avg_rush_yds REAL NOT NULL,
            avg_rec_tgts REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_proj REAL NOT NULL,
            rating REAL NOT NULL,
            snaps_per REAL NOT NULL,
            year_consistency REAL NOT NULL,
            vegas_team_total REAL NOT NULL,
            month_consistency REAL NOT NULL,
            day TEXT NOT NULL,
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
            pts_proj REAL NOT NULL,
            cieling_proj REAL NOT NULL,
            floor_proj REAL NOT NULL,
            pts_plus_minus_proj REAL NOT NULL,
            pts_sal_proj REAL NOT NULL,
            vegas_total REAL NOT NULL,         
            salary INTEGER NOT NULL,
            own_proj REAL NOT NULL,
            rating REAL NOT NULL,
            vegas_opp_total REAL NOT NULL,
            day TEXT NOT NULL,
            vegas_team_total REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id, season, week) on CONFLICT REPLACE
        )
    ";

    let kick_proj: &str = "
        CREATE TABLE IF NOT EXISTS kick_proj (
            id INTEGER NOT NULL,
            season INTEGER NOT NULL,
            week INTEGER NOT NULL,
            name TEXT NOT NULL,
            team TEXT NOT NULL,
            opp TEXT NOT NULL,
            pts_proj REAL NOT NULL,
            cieling_proj REAL NOT NULL,
            floor_proj REAL NOT NULL,
            pts_plus_minus_proj REAL NOT NULL,
            pts_sal_proj REAL NOT NULL,
            vegas_total REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_proj REAL NOT NULL,
            rating REAL NOT NULL,
            day TEXT NOT NULL,
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
            pts_proj REAL NOT NULL,
            cieling_proj REAL NOT NULL,
            floor_proj REAL NOT NULL,
            pts_plus_minus_proj REAL NOT NULL,
            pts_sal_proj REAL NOT NULL,
            vegas_total REAL NOT NULL,         
            avg_pass_atts REAL NOT NULL,
            avg_pass_comps REAL NOT NULL,
            avg_pass_yds REAL NOT NULL,
            avg_pass_tds REAL NOT NULL,
            avg_rush_atts REAL NOT NULL,
            avg_long_pass_yds REAL NOT NULL,
            pass_to_wr_per REAL NOT NULL,
            pass_to_te_per REAL NOT NULL,
            wind_speed REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_proj REAL NOT NULL,
            rating REAL NOT NULL,
            red_zone_op_pg REAL NOT NULL,
            vegas_team_total REAL NOT NULL,
            month_consistency REAL NOT NULL,
            yds_per_pass_att REAL NOT NULL,
            day TEXT NOT NULL,
            avg_rush_yds REAL NOT NULL,
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
            pts_proj REAL NOT NULL,
            cieling_proj REAL NOT NULL,
            floor_proj REAL NOT NULL,
            pts_plus_minus_proj REAL NOT NULL,
            pts_sal_proj REAL NOT NULL,
            vegas_total REAL NOT NULL,           
            avg_recp REAL NOT NULL,
            avg_tgts REAL NOT NULL,
            avg_td REAL NOT NULL,
            avg_rec_yds REAL NOT NULL,
            avg_rush_yds REAL NOT NULL,
            red_zone_op_pg REAL NOT NULL,
            rec_tgt_share REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_proj REAL NOT NULL,
            rating REAL NOT NULL,
            year_consistency REAL NOT NULL,
            year_upside REAL NOT NULL,
            vegas_team_total REAL NOT NULL,
            month_consistency REAL NOT NULL,
            month_upside REAL NOT NULL,
            day TEXT NOT NULL,
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
            pts_proj REAL NOT NULL,
            cieling_proj REAL NOT NULL,
            floor_proj REAL NOT NULL,
            pts_plus_minus_proj REAL NOT NULL,
            pts_sal_proj REAL NOT NULL,
            vegas_total REAL NOT NULL,           
            avg_recp REAL NOT NULL,
            avg_tgts REAL NOT NULL,
            avg_td REAL NOT NULL,
            avg_rec_yds REAL NOT NULL,
            avg_rush_yds REAL NOT NULL,
            red_zone_op_pg REAL NOT NULL,
            rec_tgt_share REAL NOT NULL,
            salary INTEGER NOT NULL,
            own_proj REAL NOT NULL,
            rating REAL NOT NULL,
            year_consistency REAL NOT NULL,
            year_upside REAL NOT NULL,
            vegas_team_total REAL NOT NULL,
            month_consistency REAL NOT NULL,
            month_upside REAL NOT NULL,
            day TEXT NOT NULL,
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
            day TEXT NOT NULL,
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

    let def_vs_qb: &str = "
        CREATE TABLE IF NOT EXISTS def_vs_qb (
            id INTEGER NOT NULL,
            team_name TEXT NOT NULL,
            pts_given_pg REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id) on CONFLICT REPLACE 
        )
    ";
    let def_vs_te: &str = "
        CREATE TABLE IF NOT EXISTS def_vs_te (
            id INTEGER NOT NULL,
            team_name TEXT NOT NULL,
            pts_given_pg REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id) on CONFLICT REPLACE 
        )
    ";
    let def_vs_wr: &str = "
        CREATE TABLE IF NOT EXISTS def_vs_wr (
            id INTEGER NOT NULL,
            team_name TEXT NOT NULL,
            pts_given_pg REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id) on CONFLICT REPLACE 
        )
    ";
    let def_vs_rb: &str = "
        CREATE TABLE IF NOT EXISTS def_vs_rb (
            id INTEGER NOT NULL,
            team_name TEXT NOT NULL,
            pts_given_pg REAL NOT NULL,
            FOREIGN key(id) REFERENCES player(id),
            UNIQUE(id) on CONFLICT REPLACE 
        )
    ";

    let max_score: &str = "
        CREATE TABLE IF NOT EXISTS max_score (
            pos TEXT NOT NULL,
            week INTEGER NOT NULL,
            season INTEGER NOT NULL,
            score REAL NOT NULL,
            UNIQUE(pos, week, season) on CONFLICT REPLACE 
        )
    ";

    let stats: &str = "
        CREATE TABLE IF NOT EXISTS fan_pts (
            id INTEGER NOT NULL,
            week INTEGER NOT NULL,
            season INTEGER NOT NULL,
            pts REAL NOT NULL,
            UNIQUE(id, week, season) on CONFLICT REPLACE,
            FOREIGN key(id) REFERENCES player(id)
        )
    ";

    let tables: [&str; 14] = [
        player, qb_proj, wr_proj, dst_proj, te_proj, rb_proj, ownership, kick_proj, def_vs_qb,
        def_vs_rb, def_vs_te, def_vs_wr, max_score, stats,
    ];
    for table in tables {
        conn.execute(table, ()).expect("Could not create table");
    }
}
