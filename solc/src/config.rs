use std::env;
use std::path::{Path, PathBuf};

pub const APP_NAME: &'static str = "OpenTron";
#[allow(dead_code)]
pub const VENDOR_NAME: &'static str = "OpenTron Foundation";

// FIXME: should macOS use `/Users/$USER/Library/Application Support/OpenTron`?

#[cfg(target_family = "unix")]
pub fn determine_config_directory() -> PathBuf {
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        Path::new(&xdg_config_home).join(APP_NAME).to_owned()
    } else if let Ok(home) = env::var("HOME") {
        Path::new(&home).join(".config").join(APP_NAME).to_owned()
    } else {
        panic!("lack HOME environment variable")
    }
}

#[cfg(target_family = "windows")]
pub fn determine_config_directory() -> PathBuf {
    if let Ok(user_profile) = env::var("USERPROFILE") {
        Path::new(&user_profile)
            .join(r#"AppData\Roaming"#)
            .join(VENDOR_NAME)
            .join(APP_NAME)
    } else {
        Path::new(r#"C:\"#).join(APP_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = determine_config_directory();
    }
}
