use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "./conf.toml";

#[derive(Deserialize, Serialize)]
pub struct Config {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn test_config_parse() {
        use std::fs;
        let content = fs::read_to_string(CONFIG_FILE).unwrap();
        let conf_value = content.parse::<toml::Value>().unwrap();
        println!("yep => {:?}", conf_value);
    }
}
