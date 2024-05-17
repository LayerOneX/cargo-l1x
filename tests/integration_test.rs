use cargo_l1x::build::build;
use cargo_l1x::create::create;
use std::os::unix::prelude::MetadataExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const TEST_DIR_NAME: &str = "test";
static FOLDER_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TestFolder {
    path: PathBuf,
}

impl TestFolder {
    fn new() -> Self {
        let folder_id = FOLDER_COUNTER.fetch_add(1, Ordering::SeqCst);
        let folder_name = format!("{}_{}", TEST_DIR_NAME, folder_id);
        Self {
            path: PathBuf::from(folder_name),
        }
    }

    fn name(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    fn get_path(&self, suffix: &str) -> PathBuf {
        self.path.join(suffix)
    }

    fn exists(&self, suffix: &str) -> bool {
        self.get_path(suffix).exists()
    }
}

impl Drop for TestFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.path.as_path());
    }
}

#[test]
fn test_create_and_build() {
    let folder = TestFolder::new();
    create(folder.name(), "local_default".to_string()).unwrap();
    assert!(folder.exists("Cargo.toml"));
    let target_dir = folder.get_path("target");
    let args = vec![
        "--manifest-path".to_string(),
        folder.get_path("Cargo.toml").to_str().unwrap().to_string(),
    ];

    build(args, target_dir.clone()).unwrap();

    let wasm_file_path = folder.get_path("target/wasm32-unknown-unknown/release/l1x_contract.wasm");
    assert!(wasm_file_path.exists());
    assert!(folder.exists("target/l1x/release/l1x_contract.ll"));
    let versioned_ll_path = folder.get_path("target/l1x/release/l1x_contract.versioned.ll");
    assert!(versioned_ll_path.exists());
    let o_file_path = folder.get_path("target/l1x/release/l1x_contract.o");
    assert!(o_file_path.exists());

    let wasm_size = std::fs::metadata(wasm_file_path).unwrap().size();
    let o_size = std::fs::metadata(&o_file_path).unwrap().size();
    assert!(wasm_size < o_size);

    let output = std::process::Command::new("readelf")
        .arg("-s")
        .arg(&o_file_path)
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    println!("{}", output);
    assert!(!output.contains("memcpy"));

    let output = std::process::Command::new("readelf")
        .arg("-S")
        .arg(versioned_ll_path)
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(!output.contains("_memory"));
    assert!(!output.contains("_init_memory"));
    assert!(!output.contains("_version"));
}

#[test]
fn test_create_and_build_no_strip() {
    let folder = TestFolder::new();
    create(folder.name(), "local_default".to_string()).unwrap();
    assert!(folder.exists("Cargo.toml"));
    let target_dir = folder.get_path("target");
    let args = vec![
        "--manifest-path".to_string(),
        folder.get_path("Cargo.toml").to_str().unwrap().to_string(),
        "--no-strip".to_string(),
    ];

    build(args, target_dir.clone()).unwrap();

    let wasm_file_path = folder.get_path("target/wasm32-unknown-unknown/release/l1x_contract.wasm");
    assert!(wasm_file_path.exists());
    assert!(folder.exists("target/l1x/release/l1x_contract.ll"));
    let versioned_ll_path = folder.get_path("target/l1x/release/l1x_contract.versioned.ll");
    assert!(versioned_ll_path.exists());
    let o_file_path = folder.get_path("target/l1x/release/l1x_contract.o");
    assert!(o_file_path.exists());

    let output = std::process::Command::new("readelf")
        .arg("-s")
        .arg(&o_file_path)
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("memcpy"));

    let output = std::process::Command::new("readelf")
        .arg("-S")
        .arg(versioned_ll_path)
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    println!("{}", output);
    assert!(!output.contains("_memory"));
    assert!(!output.contains("_init_memory"));
    assert!(!output.contains("_version"));
}
