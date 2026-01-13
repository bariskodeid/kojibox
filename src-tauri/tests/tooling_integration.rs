use kojibox_lib::tooling;
use std::fs;

#[test]
fn test_domain_management() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    
    // Initial list should be empty
    let domains = tooling::domains_list(&root).expect("list domains");
    assert!(domains.is_empty());

    // Add a domain
    let mapping = tooling::DomainMapping {
        domain: "test.local".to_string(),
        project_id: "p1".to_string(),
        target_port: 8080,
    };
    tooling::domains_upsert(&root, mapping.clone()).expect("upsert domain");

    // Verify addition
    let domains = tooling::domains_list(&root).expect("list domains after add");
    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0].domain, "test.local");

    // Remove domain
    tooling::domains_remove(&root, "test.local").expect("remove domain");
    let domains = tooling::domains_list(&root).expect("list domains after remove");
    assert!(domains.is_empty());
}

#[test]
fn test_cert_generation() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();

    let domains = vec!["test.local".to_string(), "*.test.local".to_string()];
    let meta = tooling::certs_generate(&root, domains).expect("generate certs");

    assert_eq!(meta.name, "test-local");
    assert!(fs::metadata(&meta.path).is_ok());
    
    // Check index update
    let certs = tooling::certs_list(&root).expect("list certs");
    assert_eq!(certs.len(), 1);
    assert_eq!(certs[0].name, "test-local");
}
