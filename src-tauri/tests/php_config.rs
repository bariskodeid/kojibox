use kojibox_lib::config::php::PhpConfigManager;
use std::fs;

#[test]
fn test_php_extension_toggle() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();
    
    // Setup initial config
    let config_dir = root.join("runtime/config/php");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::write(
        config_dir.join("extensions.ini"),
        "extension=curl\n;extension=gd\n"
    ).expect("write initial config");

    let manager = PhpConfigManager::new(root);

    // List initial state
    let list = manager.list_extensions().expect("list extensions");
    let curl = list.iter().find(|(name, _)| name == "curl").expect("find curl");
    assert!(curl.1, "curl should be enabled");
    let gd = list.iter().find(|(name, _)| name == "gd").expect("find gd");
    assert!(!gd.1, "gd should be disabled");

    // Toggle gd ON
    manager.toggle_extension("gd".to_string(), true).expect("enable gd");
    
    // Verify persistence
    let list_new = manager.list_extensions().expect("list extensions again");
    let gd_new = list_new.iter().find(|(name, _)| name == "gd").expect("find gd again");
    assert!(gd_new.1, "gd should now be enabled");

    // Verify file content
    let content = fs::read_to_string(config_dir.join("extensions.ini")).expect("read config");
    assert!(content.contains("extension=gd"));
    assert!(!content.contains(";extension=gd"));
}
