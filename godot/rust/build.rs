use gdext_gen::args::{
    BaseDirectory,
    icons::{DefaultNodeIcon, IconsConfig, IconsDirectories},
};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    Ok(gdext_gen::generate_gdextension_file(
        BaseDirectory::ProjectFolder,
        Some({
            let mut out = PathBuf::from(std::env::var("OUT_DIR")?);
            while let Some(_) = out.parent() {
                if let Some(file) = out.file_name()
                    && file == "target"
                {
                    break;
                } else {
                    out.pop();
                }
            }
            if let (Some(file), Some(root)) = (
                out.file_name(),
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent(),
            ) && file == "target"
                && let Some(path) = pathdiff::diff_paths(&out, root)
            {
                path
            } else {
                return Err(format!(
                    "failed to resolve 'target' directory from: {}",
                    out.display()
                )
                .into());
            }
        }),
        Some("../rust.gdextension".into()),
        true,
        None,
        None,
        Some(IconsConfig::new(
            DefaultNodeIcon::Custom("icon.png".into()),
            Default::default(),
            None,
            IconsDirectories::new("".into(), "".into(), "".into(), None),
        )),
    )?)
}
