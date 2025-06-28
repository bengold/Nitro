use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::commands::{install::InstallArgs, uninstall::UninstallArgs, list::ListArgs, update::UpdateArgs};
use crate::core::{NitroError, NitroResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub dependencies: Vec<String>,
    pub install_path: Option<PathBuf>,
    pub size_bytes: Option<u64>,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

pub struct PackageManager {
    db: sled::Db,
    formula_manager: super::formula::FormulaManager,
    installer: super::installer::Installer,
    resolver: super::resolver::DependencyResolver,
}

impl PackageManager {
    pub async fn new() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("com", "nitro", "nitro")
            .ok_or_else(|| NitroError::Other("Could not determine config directory".into()))?;
        
        let db_path = config_dir.data_dir().join("packages.db");
        std::fs::create_dir_all(db_path.parent().unwrap())?;
        
        let db = sled::Config::new()
            .path(&db_path)
            .mode(sled::Mode::HighThroughput)
            .flush_every_ms(Some(1000))
            .open()?;
        let formula_manager = super::formula::FormulaManager::new().await?;
        let installer = super::installer::Installer::new()?;
        let resolver = super::resolver::DependencyResolver::new();

        Ok(Self {
            db,
            formula_manager,
            installer,
            resolver,
        })
    }

    pub fn install<'a>(&'a self, package_name: &'a str, args: &'a InstallArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.install_impl(package_name, args).await
        })
    }

    async fn install_impl(&self, package_name: &str, args: &InstallArgs) -> Result<()> {
        // Get formula
        let formula = self.formula_manager.get_formula(package_name).await?;
        
        // Check if already installed
        if !args.force && self.is_installed(package_name)? {
            return Err(NitroError::Other(format!("{} is already installed", package_name)).into());
        }

        // Resolve dependencies
        let deps = if args.skip_deps {
            vec![]
        } else {
            self.resolver.resolve(&formula).await?
        };

        // Install dependencies first
        if !args.only_deps {
            for dep in &deps {
                if !self.is_installed(&dep.name)? {
                    self.install(&dep.name, args).await?;
                }
            }
        }

        // Install the package
        if !args.only_deps {
            self.installer.install(&formula, args.build_from_source).await?;
            self.mark_installed(&formula)?;
        }

        Ok(())
    }

    pub async fn uninstall(&self, package_name: &str, args: &UninstallArgs) -> Result<()> {
        if !self.is_installed(package_name)? {
            return Err(NitroError::PackageNotFound(package_name.to_string()).into());
        }

        let package = self.get_package(package_name)?;
        
        // Check for dependent packages
        if !args.force {
            let dependents = self.find_dependents(package_name)?;
            if !dependents.is_empty() {
                return Err(NitroError::Other(
                    format!("{} is required by: {}", package_name, dependents.join(", "))
                ).into());
            }
        }

        // Uninstall the package
        self.installer.uninstall(&package).await?;
        self.mark_uninstalled(package_name)?;

        Ok(())
    }

    pub async fn list_installed(&self, args: &ListArgs) -> Result<Vec<Package>> {
        let mut packages = Vec::new();
        
        for entry in self.db.iter() {
            let (_key, value) = entry?;
            let package: Package = serde_json::from_slice(&value)?;
            
            if package.installed {
                if let Some(prefix) = &args.prefix {
                    if !package.name.starts_with(prefix) {
                        continue;
                    }
                }
                packages.push(package);
            }
        }

        packages.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(packages)
    }

    pub async fn check_updates(&self, packages: &[String]) -> Result<Vec<(String, String, String)>> {
        let mut updates = Vec::new();
        
        let installed = if packages.is_empty() {
            self.list_installed(&ListArgs::default()).await?
        } else {
            let mut pkgs = Vec::new();
            for name in packages {
                if let Ok(pkg) = self.get_package(name) {
                    pkgs.push(pkg);
                }
            }
            pkgs
        };

        for package in installed {
            let formula = self.formula_manager.get_formula(&package.name).await?;
            if formula.version != package.version {
                updates.push((package.name, package.version, formula.version));
            }
        }

        Ok(updates)
    }

    pub async fn update_packages(&self, args: &UpdateArgs) -> Result<()> {
        let updates = self.check_updates(&args.packages).await?;
        
        for (name, _, _) in updates {
            println!("Updating {}...", name);
            self.install(&name, &InstallArgs {
                packages: vec![name.clone()],
                force: true,
                ..Default::default()
            }).await?;
        }

        Ok(())
    }

    fn is_installed(&self, package_name: &str) -> Result<bool> {
        if let Some(data) = self.db.get(package_name)? {
            let package: Package = serde_json::from_slice(&data)?;
            Ok(package.installed)
        } else {
            Ok(false)
        }
    }

    fn get_package(&self, package_name: &str) -> NitroResult<Package> {
        if let Some(data) = self.db.get(package_name)? {
            let package: Package = serde_json::from_slice(&data)?;
            Ok(package)
        } else {
            Err(NitroError::PackageNotFound(package_name.to_string()))
        }
    }

    fn mark_installed(&self, formula: &super::formula::Formula) -> Result<()> {
        let package = Package {
            name: formula.name.clone(),
            version: formula.version.clone(),
            description: formula.description.clone(),
            homepage: formula.homepage.clone(),
            installed: true,
            installed_version: Some(formula.version.clone()),
            dependencies: formula.dependencies.iter().map(|d| d.name.clone()).collect(),
            install_path: Some(self.installer.get_install_path(&formula.name)),
            size_bytes: None, // TODO: Calculate installed size
            installed_at: chrono::Utc::now(),
        };

        self.db.insert(&formula.name, serde_json::to_vec(&package)?)?;
        Ok(())
    }

    fn mark_uninstalled(&self, package_name: &str) -> Result<()> {
        self.db.remove(package_name)?;
        Ok(())
    }

    fn find_dependents(&self, package_name: &str) -> Result<Vec<String>> {
        let mut dependents = Vec::new();
        
        for entry in self.db.iter() {
            let (key, value) = entry?;
            let package: Package = serde_json::from_slice(&value)?;
            
            if package.installed && package.dependencies.contains(&package_name.to_string()) {
                dependents.push(String::from_utf8_lossy(&key).to_string());
            }
        }

        Ok(dependents)
    }
}

impl Default for ListArgs {
    fn default() -> Self {
        Self {
            versions: false,
            installed: false,
            size: false,
            prefix: None,
        }
    }
}

impl Default for InstallArgs {
    fn default() -> Self {
        Self {
            packages: vec![],
            force: false,
            build_from_source: false,
            only_deps: false,
            skip_deps: false,
            version: None,
            debug: false,
        }
    }
}

impl Drop for PackageManager {
    fn drop(&mut self) {
        // Ensure the database is properly flushed before dropping
        let _ = self.db.flush();
    }
}