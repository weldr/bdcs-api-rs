extern crate bdcs;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use r2d2_sqlite::SqliteConnectionManager;
use bdcs::depclose::*;
use std::env;

fn print_one(p: Proposition) {
    match p {
        Proposition::Obsoletes(left, right) => println!("{} obsoletes {}", left, right),
        Proposition::Requires(nevra, thing) => println!("{} requires {}", nevra, thing),
    }
}

fn main() {
    let mut argv: Vec<String> = env::args().collect();
    if argv.len() < 3 {
        println!("depclose metadata.db RPM [RPM...]");
    }

    // Remove the program, grab the database.
    argv.remove(0);
    let db = argv.remove(0);

    let cfg = r2d2::Config::builder().build();
    let mgr = SqliteConnectionManager::new(db.as_str());
    let pool = r2d2::Pool::new(cfg, mgr).unwrap();

    let conn = pool.get().unwrap();

    let (props, provided_by_dict) = close_dependencies(&conn, argv).unwrap_or_default();

    // Split propositions into three lists for ease of comparing output with haskell version.
    let mut obs = Vec::new();
    let mut reqs = Vec::new();

    for p in props {
        match p {
            Proposition::Obsoletes(_, _) => obs.push(p),
            Proposition::Requires(_, _)  => reqs.push(p),
        }
    }

    obs.sort();
    reqs.sort();

    for i in obs { print_one(i) }
    for i in reqs { print_one(i) }
    for (p, what_provides) in provided_by_dict {
        println!("{} is provided by {:?}", p, what_provides);
    }
}
