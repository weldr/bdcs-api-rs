extern crate bdcs;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use bdcs::depclose::*;
use bdcs::depsolve::*;

use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::env;
use std::process::exit;
use std::rc::Rc;
use std::cell::RefCell;

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

    let depexpr = match close_dependencies(&conn, &vec!(String::from("x86_64")), &argv) {
        Err(e)    => { println!("Error: {}", e);
                       exit(1);
                     }
        Ok(expr)  => expr
    };

    // Wrap the returned depexpression in the crud it needs
    let mut exprs = vec![Rc::new(RefCell::new(depexpr))];
    let mut assignments = HashMap::new();
    unit_propagation(&mut exprs, &mut assignments);
 
    println!("====== ASSIGNMENTS ======");
    for a in assignments { println!("{:?}", a) }
    println!("====== EXPRS ======");
    for x in exprs { println!("{}", *(x.borrow())) }
}
