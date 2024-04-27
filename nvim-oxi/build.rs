use std::{
    env,
    fs::{create_dir, File},
    path::Path,
};

#[cfg(unix)]
mod dynamic {
    #[allow(unused_imports)]
    pub use std::os::unix::fs::symlink;

    pub const PREFIX: &str = "lib";

    #[cfg(target_os = "linux")]
    pub const SOURCE_EXTENSION: &str = "so";
    #[cfg(target_os = "macos")]
    pub const SOURCE_EXTENSION: &str = "dylib";

    pub const TARGET_EXTENSION: &str = "so";
}
#[cfg(target_os = "windows")]
mod dynamic {
    #[allow(unused_imports)]
    pub use std::os::windows::fs::symlink_file as symlink;

    pub const PREFIX: &str = "";
    pub const SOURCE_EXTENSION: &str = "dll";

    pub const TARGET_EXTENSION: &str = "dll";
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_dir = env::var("CARGO_MANIFEST_DIR")?;
    if let Err(e) = create_dir(format!("{}/lua", root_dir)) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create 'lua' directory: {}", e);
        }
    }

    let target = env::var("CARGO_PKG_NAME")?;
    let profile = env::var("PROFILE")?;

    let source = format!(
        "{}/target/{}/{}{}.{}",
        root_dir,
        profile,
        dynamic::PREFIX,
        target,
        dynamic::SOURCE_EXTENSION,
    );
    let target = format!("{}/lua/{}.{}", root_dir, target, dynamic::TARGET_EXTENSION);

    if !Path::new(&source).exists() {
        let _ = File::create_new(&source)?;
    }

    if let Err(e) = dynamic::symlink(source, target) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create symlink: {}", e);
        }
    }

    Ok(())
}
