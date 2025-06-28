use crate::core::package::Package;
use crate::search::SearchResult;
use crate::core::tap::Tap;

pub fn show_search_results(results: &[SearchResult]) {
    println!("Found {} package(s):\n", results.len());
    
    for result in results {
        println!("🍺 {} ({})", result.name, result.version);
        if let Some(description) = &result.description {
            println!("   {}", description);
        }
        println!("   From: {}", result.tap);
        if results.len() > 1 {
            println!();
        }
    }
}

pub fn show_package_info(package: &Package) {
    println!("📦 {}", package.name);
    println!("Version: {}", package.version);
    
    if let Some(description) = &package.description {
        println!("Description: {}", description);
    }
    
    if let Some(homepage) = &package.homepage {
        println!("Homepage: {}", homepage);
    }
    
    if !package.dependencies.is_empty() {
        println!("Dependencies: {}", package.dependencies.join(", "));
    }
    
    if let Some(path) = &package.install_path {
        println!("Installed to: {}", path.display());
    }
    
    println!("Installed at: {}", package.installed_at.format("%Y-%m-%d %H:%M:%S"));
    
    if let Some(size) = package.size_bytes {
        println!("Size: {}", format_bytes(size));
    }
}

pub fn show_package_list(packages: &[Package]) {
    if packages.is_empty() {
        println!("No packages installed.");
        return;
    }
    
    println!("Installed packages ({}):\n", packages.len());
    
    for package in packages {
        println!("🍺 {} ({})", package.name, package.version);
        if let Some(description) = &package.description {
            let desc = if description.len() > 60 {
                format!("{}...", &description[..57])
            } else {
                description.clone()
            };
            println!("   {}", desc);
        }
        
        if let Some(size) = package.size_bytes {
            println!("   Size: {}", format_bytes(size));
        }
        println!();
    }
}

pub fn show_tap_list(taps: &[Tap]) {
    if taps.is_empty() {
        println!("No taps configured.");
        return;
    }
    
    println!("Configured taps ({}):\n", taps.len());
    
    for tap in taps {
        println!("🔗 {}", tap.name);
        println!("   URL: {}", tap.url);
        
        if let Some(updated) = tap.updated_at {
            println!("   Last updated: {}", updated.format("%Y-%m-%d %H:%M:%S"));
        }
        
        // Count formulae in tap
        let formula_dir = tap.path.join("Formula");
        if formula_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&formula_dir) {
                let count = entries.filter(|e| {
                    e.as_ref()
                        .map(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rb"))
                        .unwrap_or(false)
                }).count();
                
                println!("   Formulae: {}", count);
            }
        }
        println!();
    }
}

pub fn show_installation_summary(installed: &[String], failed: &[String]) {
    if !installed.is_empty() {
        println!("\n✅ Successfully installed:");
        for package in installed {
            println!("   • {}", package);
        }
    }
    
    if !failed.is_empty() {
        println!("\n❌ Failed to install:");
        for package in failed {
            println!("   • {}", package);
        }
    }
    
    println!("\nInstallation complete.");
}

pub fn show_uninstall_confirmation(packages: &[String]) -> bool {
    use std::io::{self, Write};
    
    println!("The following packages will be uninstalled:");
    for package in packages {
        println!("  • {}", package);
    }
    
    print!("\nProceed? [y/N]: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;
    
    if bytes < THRESHOLD {
        return format!("{} B", bytes);
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

pub fn show_update_summary(updated: &[String], skipped: &[String], failed: &[String]) {
    if !updated.is_empty() {
        println!("\n📦 Updated packages:");
        for package in updated {
            println!("   ✓ {}", package);
        }
    }
    
    if !skipped.is_empty() {
        println!("\n⏭️  Already up to date:");
        for package in skipped {
            println!("   • {}", package);
        }
    }
    
    if !failed.is_empty() {
        println!("\n❌ Failed to update:");
        for package in failed {
            println!("   • {}", package);
        }
    }
    
    println!("\nUpdate complete.");
}

pub fn show_formula_info(formula: &crate::core::formula::Formula, _args: &crate::cli::commands::info::InfoArgs) {
    println!("\n📦 {}", formula.name);
    println!("Version: {}", formula.version);
    
    if let Some(description) = &formula.description {
        println!("Description: {}", description);
    }
    
    if let Some(homepage) = &formula.homepage {
        println!("Homepage: {}", homepage);
    }
    
    if let Some(license) = &formula.license {
        println!("License: {}", license);
    }
    
    if !formula.dependencies.is_empty() {
        println!("\nDependencies:");
        for dep in &formula.dependencies {
            let dep_type = if dep.build_only { " (build)" } else { "" };
            println!("  • {}{}", dep.name, dep_type);
        }
    }
    
    if !formula.conflicts.is_empty() {
        println!("\nConflicts with:");
        for conflict in &formula.conflicts {
            println!("  • {}", conflict);
        }
    }
    
    if let Some(caveats) = &formula.caveats {
        println!("\n⚠️  Caveats:");
        println!("{}", caveats);
    }
}