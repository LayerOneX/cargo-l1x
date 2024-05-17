use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() -> io::Result<()> {
    let out_dir =
        env::var_os("OUT_DIR").expect("OUT_DIR should be defined by cargo during compilation");
    let folder_path = Path::new("default_template");
    let zip_path = Path::new(&out_dir).join("default_template.zip");

    let file = fs::File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);

    zip.add_directory(folder_path.to_string_lossy(), FileOptions::<()>::default())?;
    zip_folder(&mut zip, folder_path)?;

    zip.finish()?;

    Ok(())
}

fn zip_folder(writer: &mut ZipWriter<fs::File>, folder_path: &Path) -> io::Result<()> {
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name() == Some("target".as_ref()) {
                continue; // ignore target directory
            }
            writer.add_directory(path.to_string_lossy(), FileOptions::<()>::default())?;
            zip_folder(writer, &path)?;
            continue;
        }

        let mut file = fs::File::open(&path)?;
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents)?;

        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        writer.start_file(path.to_string_lossy(), options)?;
        writer.write_all(&contents)?;
    }

    Ok(())
}
