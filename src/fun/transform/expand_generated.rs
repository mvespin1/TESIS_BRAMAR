use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
  fun::{Book, Name, Term},
  maybe_grow,
};

/// Dereferences any non recursive generated definitions in the term.
/// Used after readback.
impl Term {
  pub fn expand_generated(&mut self, book: &Book, recursive_defs: &RecursiveDefs) {
    maybe_grow(|| {
      if let Term::Ref { nam } = &*self {
        if nam.is_generated() && !recursive_defs.contains(nam) {
          *self = book.defs.get(nam).unwrap().rule().body.clone();
        }
      }

      for child in self.children_mut() {
        child.expand_generated(book, recursive_defs);
      }
    })
  }
}

type DepGraph = HashMap<Name, HashSet<Name>>;
type Cycles = Vec<Vec<Name>>;
type RecursiveDefs = BTreeSet<Name>;

impl Book {
  pub fn recursive_defs(&self) -> RecursiveDefs {
    let mut cycle_map = BTreeSet::new();
    let deps = book_def_deps(self);
    let cycles = cycles(&deps);

    for cycle in cycles {
      for name in cycle {
        cycle_map.insert(name);
      }
    }

    cycle_map
  }
}

/// Find all cycles in the dependency graph.
fn cycles(deps: &DepGraph) -> Cycles {
  let mut cycles = vec![];
  let mut visited = HashSet::new();
  // let mut stack = vec![];
  for nam in deps.keys() {
    if !visited.contains(nam) {
      find_cycles(deps, nam, &mut visited, &mut cycles);
    }
  }
  cycles
}

fn find_cycles(deps: &DepGraph, nam: &Name, visited: &mut HashSet<Name>, cycles: &mut Cycles) {
  let mut stack = vec![(nam.clone(), vec![])];
  while let Some((current, path)) = stack.pop() {
    if visited.contains(&current) {
      // Check if the current ref is already in the stack, which indicates a cycle.
      if let Some(cycle_start) = path.iter().position(|n| n == &current) {
        // If found, add the cycle to the cycles vector.
        cycles.push(path[cycle_start..].to_vec());
      }
      continue;
    }

    // If the ref has not been visited yet, mark it as visited.
    visited.insert(current.clone());
    // Add the current ref to the stack to keep track of the path.
    let mut new_path = path.clone();
    new_path.push(current.clone());

    // Search for cycles from each dependency.
    if let Some(deps) = deps.get(&current) {
      for dep in deps {
        stack.push((dep.clone(), new_path.clone()));
      }
    }
  }
}

fn book_def_deps(book: &Book) -> DepGraph {
  book.defs.iter().map(|(nam, def)| (nam.clone(), def_deps(def))).collect()
}

fn def_deps(def: &crate::fun::Definition) -> HashSet<Name> {
  fn collect_refs(term: &Term, set: &mut HashSet<Name>) {
    if let Term::Ref { nam } = term {
      set.insert(nam.clone());
    }
    for children in term.children() {
      collect_refs(children, set);
    }
  }

  let mut set = HashSet::new();
  let term = &def.rule().body;

  collect_refs(term, &mut set);

  set
}
