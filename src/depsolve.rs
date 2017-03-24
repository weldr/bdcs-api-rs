use depclose::*;

use rusqlite::Connection;
use std::collections::HashMap;
use std::ops::Index;

pub fn solve_dependencies(_conn: &Connection, exprs: &mut Vec<DepExpression>) -> Result<Vec<i64>, String> {
    let mut assignments = HashMap::new();

    unit_propagation(exprs, &mut assignments);

    // FIXME:  For now, only return results if unit_propagation was able to do everything.  This
    // should handle most basic cases.  More complicated cases will require real dependency
    // solving.
    if exprs.is_empty() {
        // Take the GroupId -> bool hash map and convert it to just a list of i64.  We only care
        // about the GroupId for packages that will be installed.
        let results = assignments.into_iter().filter_map(|x| match x {
            (i, true) => Some(i),
            _ => None
        }).collect();

        Ok(results)
    } else {
        Err(String::from("Unsolved expressions"))
    }
}

fn unit_propagation(exprs: &mut Vec<DepExpression>, assignments: &mut HashMap<GroupId, bool>) -> bool {
    unit_propagation_helper(exprs, assignments, true)
}

// if assign is true, when a unit is found, process it and add it to the cache so that it will be propagated
// if assign is false, propagate units from the cache but do not add new units to the cache
// In short, assign is false when processing an OR. Given an expression like:
//
//    And(A, Not(B), Or(A, And(D, C), E), Or(D, E))
//
// the first pass might result in something like:
//
//    A=true, B=false;
//    And(Or(A, And(D, C), E), Or(D, E))
//
// Then, when processing the Or(A, And(D, C), E), the A can be removed because we know it's true,
// but we shouldn't do anything with And(D, C), since messing with D will mess with the Or(D, E)
// branch. The And(D, C) portion could be false and the parent Or() expression can still be true,
// so don't try to infer anything from those values.
fn unit_propagation_helper(exprs: &mut Vec<DepExpression>, assignments: &mut HashMap<GroupId, bool>, assign: bool) -> bool {
    let mut ever_changed = false;

    loop {
        let mut indices_to_remove = Vec::new();
        let mut indices_to_replace = Vec::new();
        let mut changed = false;

        for (i, val) in exprs.into_iter().enumerate() {
            match *val {
                DepExpression::Atom(ref id) => {
                    if !assignments.contains_key(id) {
                        if assign {
                            assignments.insert(*id, true);
                            changed = true;
                            indices_to_remove.push(i);
                        }
                    } else if assignments.get(id) == Some(&true) {
                        changed = true;
                        indices_to_remove.push(i);
                    } else {
                        panic!("conflict resolving {}", id);
                    }
                },

                DepExpression::Not(ref id) => {
                    if !assignments.contains_key(id) {
                        if assign {
                            assignments.insert(*id, false);
                            changed = true;
                            indices_to_remove.push(i);
                        }
                    } else if assignments.get(id) == Some(&false) {
                        changed = true;
                        indices_to_remove.push(i);
                    } else {
                        panic!("conflict resolving {}", id);
                    }
                    // TODO else?
                },

                DepExpression::And(ref mut and_list) => {
                    // recurse on this list of expressions
                    if unit_propagation_helper(and_list, assignments, assign) {
                        changed = true;
                    }

                    // if there's only one thing left in the list, the And is actually just that
                    // thing.
                    if and_list.len() == 1 {
                        indices_to_replace.push(i);
                        changed = true;
                    } else if and_list.is_empty() {
                        indices_to_remove.push(i);
                        changed = true;
                    }
                },

                DepExpression::Or(ref mut or_list) => {
                    // For or, check if there's only one thing first, so we don't waste time
                    // processing the child with assign=false just to (potentially) redo it with
                    // assign=true
                    if or_list.len() == 1 {
                        indices_to_replace.push(i);
                        changed = true;
                    } else if or_list.is_empty() {
                        indices_to_remove.push(i);
                        changed = true;
                    // Can't get rid of it completely, so look for known units inside the Or that can be removed
                    } else if unit_propagation_helper(or_list, assignments, false) {
                        changed = true;
                    }
                }
            }
        }

        for i in indices_to_replace {
            let expr = match *exprs.index(i) {
                DepExpression::And(ref and_list) => and_list.index(0).clone(),
                DepExpression::Or(ref or_list)   => or_list.index(0).clone(),
                _ => unreachable!()
            };
            exprs.remove(i);
            exprs.insert(i, expr);
        }

        indices_to_remove.sort_by(|a, b| b.cmp(a));
        for i in indices_to_remove {
            exprs.remove(i);
        }

        if !changed {
            break;
        } else {
            ever_changed = true;
        }
    }

    ever_changed
}
