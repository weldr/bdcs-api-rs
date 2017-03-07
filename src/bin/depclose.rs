extern crate bdcs;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use bdcs::db::*;
use bdcs::depclose::*;
use bdcs::depsolve::*;

use r2d2_sqlite::SqliteConnectionManager;
use std::env;
use std::process::exit;
use std::rc::Rc;

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
    let mut exprs = vec![Rc::new(DepCell::new(depexpr))];

    match solve_dependencies(&conn, &mut exprs) {
        Ok(ids) => { let mut results = Vec::new();
                     for id in ids {
                         match get_groups_id(&conn, &id) {
                             // Commented out for the moment - just printing group names is easier
                             // to debug.
                             // Ok(Some(grp)) => { let mut details = get_projects_details(&conn, &[grp.name.as_str()], 0, -1).unwrap();
                             //                    results.append(&mut details);
                             //                  }
                             Ok(Some(grp)) => { results.push(grp.name) }
                             _ => { }
                         }
                     }

                     for x in results {
                         println!("{}", x);
                     }
                   }
        Err(e)  => { println!("{}", e); }
    }
}
