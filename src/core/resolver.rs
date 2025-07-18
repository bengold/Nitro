use std::collections::{HashMap, HashSet, VecDeque};

use super::formula::{Formula, FormulaManager};
use crate::core::{NitroError, NitroResult};

pub struct DependencyResolver {
    // The resolver is currently stateless. A cache could be added here later.
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn resolve(&self, formula: &Formula, formula_manager: &FormulaManager) -> NitroResult<Vec<Formula>> {
        let mut resolved = Vec::new();
        let mut seen = HashSet::new();
        let mut queue = VecDeque::new();

        // Add initial dependencies to queue
        for dep in &formula.dependencies {
            if !dep.optional {
                queue.push_back(dep.clone());
            }
        }

        // Add build dependencies if building from source
        for dep in &formula.build_dependencies {
            queue.push_back(dep.clone());
        }

        // Process queue
        while let Some(dep) = queue.pop_front() {
            if seen.contains(&dep.name) {
                continue;
            }
            seen.insert(dep.name.clone());

            // Get formula for dependency, handling special name mappings
            let dep_formula = match formula_manager.get_formula(&dep.name).await {
                Ok(f) => f,
                Err(_) => {
                    // Try common dependency name variations
                    let variations = vec![
                        dep.name.replace("@", "at"),  // openssl@3 -> opensslat3
                        dep.name.replace("-", ""),     // ca-certificates -> cacertificates
                        dep.name.replace("_", "-"),    // some_package -> some-package
                        dep.name.replace("-", "_"),    // some-package -> some_package
                    ];
                    
                    let mut found = None;
                    for variant in variations {
                        if let Ok(f) = formula_manager.get_formula(&variant).await {
                            eprintln!("Resolved dependency '{}' to '{}'", dep.name, variant);
                            found = Some(f);
                            break;
                        }
                    }
                    
                    match found {
                        Some(f) => f,
                        None => {
                            eprintln!("Warning: Could not resolve dependency '{}', skipping", dep.name);
                            continue;
                        }
                    }
                }
            };

            // Check for conflicts
            self.check_conflicts(&dep_formula, &resolved)?;

            // Add sub-dependencies to queue
            for sub_dep in &dep_formula.dependencies {
                if !sub_dep.optional && !seen.contains(&sub_dep.name) {
                    queue.push_back(sub_dep.clone());
                }
            }

            resolved.push(dep_formula);
        }

        // Sort by dependency order (topological sort)
        let sorted = self.topological_sort(resolved)?;
        
        Ok(sorted)
    }

    fn check_conflicts(&self, formula: &Formula, resolved: &[Formula]) -> NitroResult<()> {
        // Check if this formula conflicts with any already resolved
        for resolved_formula in resolved {
            if formula.conflicts.contains(&resolved_formula.name) {
                return Err(NitroError::DependencyResolution(
                    format!("{} conflicts with {}", formula.name, resolved_formula.name)
                ));
            }
            if resolved_formula.conflicts.contains(&formula.name) {
                return Err(NitroError::DependencyResolution(
                    format!("{} conflicts with {}", resolved_formula.name, formula.name)
                ));
            }
        }
        Ok(())
    }

    fn topological_sort(&self, formulae: Vec<Formula>) -> NitroResult<Vec<Formula>> {
        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut formula_map: HashMap<String, Formula> = HashMap::new();

        // Initialize graph
        for formula in &formulae {
            graph.insert(formula.name.clone(), Vec::new());
            in_degree.insert(formula.name.clone(), 0);
            formula_map.insert(formula.name.clone(), formula.clone());
        }

        // Build edges
        for formula in &formulae {
            for dep in &formula.dependencies {
                if let Some(deps) = graph.get_mut(&dep.name) {
                    deps.push(formula.name.clone());
                    *in_degree.get_mut(&formula.name).unwrap() += 1;
                }
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue = VecDeque::new();
        let mut sorted = Vec::new();

        // Find nodes with no incoming edges
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name.clone());
            }
        }

        while let Some(name) = queue.pop_front() {
            if let Some(formula) = formula_map.get(&name) {
                sorted.push(formula.clone());
            }

            if let Some(dependents) = graph.get(&name) {
                for dependent in dependents {
                    let degree = in_degree.get_mut(dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        if sorted.len() != formulae.len() {
            return Err(NitroError::DependencyResolution(
                "Circular dependency detected".into()
            ));
        }

        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        // TODO: Add tests for dependency resolution
    }
}