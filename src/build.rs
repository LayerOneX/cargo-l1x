use super::which::which;
use anyhow::anyhow;
use l1x_wasm_llvmir::translate_module_to_file_by_path;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;

use thiserror::Error;

const OBJECT_FILE_VERSION: i64 = 1;
const EXPECTED_RUNTIME_VERSION: i64 = 3;
const EBPF_STACK_FRAME_SIZE: u32 = 8192;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Invalid target directory. Should not happen")]
    TargetDirError,
    #[error("Failed to execute cargo: {0}")]
    CargoBuildError(std::io::Error),
    #[error("Failed to build wasm")]
    WasmBuildError,
    #[error("Could not build ll file: {0}")]
    LlBuildError(anyhow::Error),
    #[error("filesystem error")]
    IoError(anyhow::Error, std::io::Error),
    #[error("Failed to run llc command. Please ensure that your version of llc is > 17, or you have llc-17, 18 or 19 installed")]
    LlcRunError(anyhow::Error),
    #[error("Failed to build object file")]
    ObjectBuildError,
    #[error(
        "Failed to run llvm strip on object file. Please ensure that you have llvm-strip installed"
    )]
    LlvmStripRunError(anyhow::Error),
    #[error("Failed to strip object file")]
    LlvmStripError,
}

pub fn build(mut args: Vec<String>, target_dir: PathBuf) -> Result<(), BuildError> {
    let mut command = process::Command::new("cargo");

    let mut no_strip = false;
    if args.contains(&"--no-strip".to_string()) {
        args = args.into_iter().filter(|x| x != "--no-strip").collect();
        no_strip = true;
    } else {
        command.env("RUSTFLAGS", "-C link-arg=-s");
    }

    command
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .args(&args);

    if !args.contains(&"--release".to_string()) {
        // avoid double --release
        command.arg("--release");
    }

    let mut output = command
        .spawn()
        .map_err(|e| BuildError::CargoBuildError(e))?;

    let status = output.wait().map_err(|e| BuildError::CargoBuildError(e))?;

    if !status.success() {
        println!("Failed to build wasm");
        return Err(BuildError::WasmBuildError);
    }

    let bin_dir = target_dir.join("l1x/release");

    fs::create_dir_all(bin_dir.clone())
        .map_err(|e| BuildError::IoError(anyhow!("Could not create target directory"), e))?;

    let output = command
        .arg("--message-format")
        .arg("json")
        .output()
        .map_err(|e| BuildError::CargoBuildError(e))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.split("\n").collect();

    for line in lines {
        if let Ok(cargo_metadata::Message::CompilerArtifact(artifact)) =
            serde_json::from_str::<cargo_metadata::Message>(line)
        {
            let wasm_file_path = artifact.filenames[0].clone();
            if wasm_file_path.extension() == Some("wasm") {
                let ll_file_path = wasm_file_path.with_extension("ll");
                let ll_file_path = bin_dir.join(
                    &ll_file_path
                        .file_name()
                        .expect("Generated .ll file should have a file name"),
                );
                translate_module_to_file_by_path(
                    &wasm_file_path.clone().into(),
                    &ll_file_path.clone().into(),
                )
                .map_err(|e| BuildError::LlBuildError(e))?;

                build_ebpf(&ll_file_path, no_strip)?;

                let object_file_path = wasm_file_path.with_extension("o");
                println!(
                    "✅ Contract object file '{:?}' has been built",
                    object_file_path
                        .file_name()
                        .expect("Generated .o file should have a file name")
                );
            }
        }
    }

    Ok(())
}

pub fn build_ebpf<P: AsRef<Path> + Clone>(path: P, no_strip: bool) -> Result<(), BuildError> {
    let source_file = path.clone();
    let versioned_file = path.as_ref().with_extension("versioned.ll");
    let target_file = path.as_ref().with_extension("o");

    // Copy the source file to the versioned file
    std::fs::copy(source_file, &versioned_file)
        .map_err(|e| BuildError::IoError(anyhow!("Failed to copy source file"), e))?;

    // Add the version information to the versioned file
    add_version_info(&versioned_file)?;

    // Fix the versioned file for mac os compatibility
    fix_version_file(&versioned_file)?;

    // Compile the versioned file to the target file
    compile_to_object(&versioned_file, &target_file)?;

    if !no_strip {
        // Strip the target file
        strip_object_file(&target_file)?;
    }

    Ok(())
}

fn add_version_info<P: AsRef<Path>>(versioned_file: P) -> Result<(), BuildError> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(versioned_file)
        .map_err(|e| BuildError::IoError(anyhow!("Failed to open versioned file"), e))?;

    writeln!(
        file,
        "@_OBJECT_VERSION = global i64 {}, section \"_version\", align 1",
        OBJECT_FILE_VERSION
    )
    .map_err(|e| BuildError::IoError(anyhow!("Failed to write version info"), e))?;
    writeln!(
        file,
        "@_EXPECTED_RUNTIME_VERSION = global i64 {}, section \"_version\", align 1",
        EXPECTED_RUNTIME_VERSION
    )
    .map_err(|e| BuildError::IoError(anyhow!("Failed to write version info"), e))?;
    Ok(())
}

pub fn fix_version_file<P: AsRef<Path>>(versioned_file: P) -> Result<(), BuildError> {
    let mut content = fs::read_to_string(versioned_file.as_ref())
        .map_err(|e| BuildError::IoError(anyhow::anyhow!("Failed to read version file"), e))?;

    // MAC OS: wasm-llvmir tool adds a comma before a section name for unknown reason.
    // This is a workaround until it's fixed in wasm-llvmir
    content = content.replace("section \",_memory\"", "section \"_memory\"");
    content = content.replace("section \",_init_memory\"", "section \"_init_memory\"");

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(versioned_file.as_ref())
        .map_err(|e| BuildError::IoError(anyhow::anyhow!("Failed to open version file"), e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| BuildError::IoError(anyhow::anyhow!("Failed to write to version file"), e))?;

    Ok(())
}

///  $ llc-17 -march=bpf -mattr=help
///  Available CPUs for this target:
///
///    generic - Select the generic processor.
///    probe   - Select the probe processor.
///    v1      - Select the v1 processor.
///    v2      - Select the v2 processor.
///    v3      - Select the v3 processor.
///
///  Available features for this target:
///
///    alu32    - Enable ALU32 instructions.
///    dummy    - unused feature.
///    dwarfris - Disable MCAsmInfo DwarfUsesRelocationsAcrossSections.
///
/// Use +feature to enable a feature, or -feature to disable it.
/// For example, llc -mcpu=mycpu -mattr=+feature1,-feature2
///
/// https://chromium.googlesource.com/external/github.com/llvm/llvm-project/+/refs/heads/upstream/release/17.x/llvm/lib/Target/BPF/BPFSubtarget.cpp
///   if (CPU == "v3") {
///    HasJmpExt = true;
///    HasJmp32 = true;
///    HasAlu32 = true;
///    return;
///  }
fn compile_to_object<P: AsRef<Path>>(input_file: P, output_file: P) -> Result<(), BuildError> {
    let command = get_llc_command()?.to_string();

    let output = Command::new(command)
        .args(&[
            "-march=bpf",
            "-mcpu=v3",
            "-filetype=obj",
            "--nozero-initialized-in-bss",
            "--bpf-stack-size",
            EBPF_STACK_FRAME_SIZE.to_string().as_str(),
            input_file
                .as_ref()
                .to_str()
                .expect("Path should be valid unicode"),
            "-o",
            output_file
                .as_ref()
                .to_str()
                .expect("Path should be valid unicode"),
        ])
        .output()
        .map_err(|e| BuildError::LlcRunError(e.into()))?;

    if !output.status.success() {
        eprintln!(
            "Error compiling to object file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(BuildError::ObjectBuildError);
    }
    Ok(())
}

fn strip_object_file<P: AsRef<Path>>(target_file: P) -> Result<(), BuildError> {
    let command = get_llvm_command()?.to_string();

    let output = Command::new(command)
        .arg("-x")
        .arg(
            target_file
                .as_ref()
                .to_str()
                .expect("Path should be valid unicode"),
        )
        .output()
        .map_err(|e| BuildError::LlvmStripRunError(e.into()))?;

    if !output.status.success() {
        eprintln!(
            "Error stripping object file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(BuildError::LlvmStripError);
    }
    Ok(())
}

fn get_llc_command() -> Result<String, BuildError> {
    if let Ok(path_str) = std::env::var("LLVM_BIN_PATH") {
        let path = format!("{}/llc", path_str);
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }
    if which("llc-17".to_string()).is_some() {
        return Ok("llc-17".into());
    } else if which("llc-18".to_string()).is_some() {
        return Ok("llc-18".into());
    } else if which("llc-19".to_string()).is_some() {
        return Ok("llc-19".into());
    } else if which("llc".to_string()).is_some() {
        let output = Command::new("llc").arg("--version").output();

        if let Ok(output) = output {
            let version_str = String::from_utf8_lossy(&output.stdout);
            if version_str.contains("version 17.")
                || version_str.contains("version 18.")
                || version_str.contains("version 19.")
            {
                return Ok("llc".into());
            } else {
                return Err(BuildError::LlcRunError(anyhow!("")));
            }
        } else {
            return Err(BuildError::LlcRunError(anyhow!("")));
        }
    } else {
        return Err(BuildError::LlcRunError(anyhow!("")));
    }
}

fn get_llvm_command() -> Result<String, BuildError> {
    if std::env::var("LLVM_BIN_PATH").is_ok() {
        let path = format!(
            "{}/llvm-strip",
            std::env::var("LLVM_BIN_PATH").expect("checked")
        );
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }
    if which("llvm-strip-17".to_string()).is_some() {
        return Ok("llvm-strip-17".into());
    } else if which("llvm-strip-18".to_string()).is_some() {
        return Ok("llvm-strip-18".into());
    } else if which("llvm-strip-19".to_string()).is_some() {
        return Ok("llvm-strip-19".into());
    } else if which("llvm-strip".to_string()).is_some() {
        let output = Command::new("llvm-strip").arg("--version").output();

        if let Ok(output) = output {
            let version_str = String::from_utf8_lossy(&output.stdout);
            if version_str.contains("version 17.")
                || version_str.contains("version 18.")
                || version_str.contains("version 19.")
            {
                return Ok("llvm-strip".into());
            } else {
                return Err(BuildError::LlvmStripRunError(anyhow!("")));
            }
        } else {
            return Err(BuildError::LlvmStripRunError(anyhow!("")));
        }
    } else {
        return Err(BuildError::LlvmStripRunError(anyhow!("")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_version_file() {
        let versioned_file = "tests/fixtures/macos.versioned.ll";
        let content = fs::read_to_string(versioned_file).unwrap();

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        fix_version_file(temp_file.path()).unwrap();

        let content = fs::read_to_string(temp_file).unwrap();
        assert!(
            content.contains(" section \"_init_memory\""),
            "Can't find 'section \"_init_memory\"' in .versioned.ll"
        );
        assert!(
            content.contains(" section \"_memory\""),
            "Can't find 'section \"_memory\"' in .versioned.ll"
        );
    }
}
