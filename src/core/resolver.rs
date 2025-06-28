use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::{NitroError, NitroResult};
use super::formula::Formula;

pub struct DependencyResolver {
    // Cache resolved dependencies to avoid re-computing
    cache: HashMap<String, Vec<Formula>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub async fn resolve(&self, formula: &Formula) -> NitroResult<Vec<Formula>> {
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

            // Get formula for dependency
            let formula_manager = super::formula::FormulaManager::new().await?;
            let dep_formula = formula_manager.get_formula(&dep.name).await?;

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