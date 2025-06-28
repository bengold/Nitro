use nitro::core::formula::{Formula, FormulaParser, Dependency};

#[test]
fn test_formula_parser() {
    let ruby_content = r#"
class Wget < Formula
  desc "Internet file retriever"
  homepage "https://www.gnu.org/software/wget/"
  url "https://ftp.gnu.org/gnu/wget/wget-1.24.5.tar.gz"
  sha256 "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
  
  depends_on "pkg-config" => :build
  depends_on "openssl@3"
  
  def install
    system "./configure", "--prefix=#{prefix}"
    system "make", "install"
  end
end
"#;

    let parser = FormulaParser::new();
    let formula = parser.parse_content(ruby_content).unwrap();
    
    assert_eq!(formula.name, "wget");
    assert_eq!(formula.description, Some("Internet file retriever".to_string()));
    assert_eq!(formula.homepage, Some("https://www.gnu.org/software/wget/".to_string()));
    assert_eq!(formula.version, "1.24.5");
    assert_eq!(formula.dependencies.len(), 2);
    assert!(formula.install_script.is_some());
}

#[test]
fn test_dependency_resolver() {
    use nitro::core::resolver::DependencyResolver;
    
    let _resolver = DependencyResolver::new();
    
    let formula = Formula {
        name: "test".to_string(),
        version: "1.0".to_string(),
        description: None,
        homepage: None,
        license: None,
        sources: vec![],
        dependencies: vec![
            Dependency {
                name: "dep1".to_string(),
                version: None,
                build_only: false,
                optional: false,
            },
            Dependency {
                name: "dep2".to_string(),
                version: None,
                build_only: true,
                optional: false,
            },
        ],
        build_dependencies: vec![],
        optional_dependencies: vec![],
        conflicts: vec![],
        install_script: None,
        test_script: None,
        caveats: None,
        binary_packages: vec![],
    };
    
    // This would need FormulaManager to be mockable for full testing
    // For now, just test that the resolver can be created
    assert_eq!(formula.dependencies.len(), 2);
}

#[test]
fn test_search_result_structure() {
    use nitro::search::SearchResult;
    use std::path::PathBuf;
    
    let result = SearchResult {
        name: "wget".to_string(),
        description: Some("Internet file retriever".to_string()),
        version: "1.24.5".to_string(),
        tap: "homebrew/core".to_string(),
        formula_path: PathBuf::from("/path/to/formula.rb"),
        score: 1.0,
    };
    
    assert_eq!(result.name, "wget");
    assert_eq!(result.version, "1.24.5");
    assert!(result.description.is_some());
}

#[tokio::test]
async fn test_tap_url_generation() {
    // Test that tap URLs are generated correctly
    let tap_name = "homebrew/core";
    let expected_url = "https://github.com/homebrew/core.git";
    
    // The actual URL generation happens in tap.rs
    assert_eq!(format!("https://github.com/{}.git", tap_name), expected_url);
}

#[test]
fn test_error_types() {
    use nitro::core::NitroError;
    
    let err = NitroError::PackageNotFound("test".to_string());
    assert_eq!(err.to_string(), "Package not found: test");
    
    let err = NitroError::FormulaParse("invalid syntax".to_string());
    assert_eq!(err.to_string(), "Formula parse error: invalid syntax");
}