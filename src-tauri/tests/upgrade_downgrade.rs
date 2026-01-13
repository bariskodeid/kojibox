use kojibox_lib::installer::apply_archive_with_rollback;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use zip::write::FileOptions;
use zip::ZipWriter;

fn write_manifest(path: &PathBuf, version: &str) {
    let content = format!(
        "{{\n  \"version\": \"{}\",\n  \"services\": [],\n  \"bundle\": {{\n    \"createdAt\": \"0\",\n    \"source\": \"test\",\n    \"signature\": \"\"\n  }}\n}}\n",
        version
    );
    fs::write(path, content).expect("write manifest");
}

fn create_archive(path: &PathBuf, version: &str, include_runtime: bool) {
    let file = fs::File::create(path).expect("create archive");
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default();
    if include_runtime {
        zip.add_directory("runtime/", options).unwrap();
        zip.add_directory("runtime/bin/", options).unwrap();
        zip.start_file("runtime/manifest.json", options).unwrap();
        let content = format!(
            "{{\n  \"version\": \"{}\",\n  \"services\": [],\n  \"bundle\": {{\n    \"createdAt\": \"0\",\n    \"source\": \"archive\",\n    \"signature\": \"\"\n  }}\n}}\n",
            version
        );
        zip.write_all(content.as_bytes()).unwrap();
    } else {
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(b"{}\n").unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn upgrade_and_rollback_validation() {
    let root = tempdir().expect("tempdir");
    let runtime_root = root.path().join("runtime");
    fs::create_dir_all(&runtime_root).expect("create runtime");
    write_manifest(&runtime_root.join("manifest.json"), "1");

    let archive_v2 = root.path().join("update-v2.zip");
    create_archive(&archive_v2, "2", true);
    apply_archive_with_rollback(&root.path().to_path_buf(), &archive_v2, "")
        .expect("apply v2");
    let updated = fs::read_to_string(runtime_root.join("manifest.json")).unwrap();
    assert!(updated.contains("\"version\": \"2\""));

    let archive_bad = root.path().join("update-bad.zip");
    create_archive(&archive_bad, "3", false);
    let result = apply_archive_with_rollback(&root.path().to_path_buf(), &archive_bad, "");
    assert!(result.is_err());
    let after = fs::read_to_string(runtime_root.join("manifest.json")).unwrap();
    assert!(after.contains("\"version\": \"2\""));
}
