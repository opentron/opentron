use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSource {
    content: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct OptimizerSetting {
    enabled: bool,
    runs: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSettings {
    optimizer: Option<OptimizerSetting>,
    // "myFile.sol": { "MyLib": "0x123123..."}
    libraries: HashMap<String, HashMap<String, String>>,
    // "*": {
    //    "*": [ "metadata"，"evm.bytecode" ]
    // },
    #[serde(rename = "outputSelection")]
    output_selection: HashMap<String, HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    // Solidity
    language: String,
    sources: HashMap<String, InputSource>,
    settings: InputSettings,
}

impl Input {
    pub fn new() -> Self {
        let required_fields: Vec<String> = vec!["abi".into(), "evm.bytecode.object".into()];
        let cntr_sel = HashMap::from_iter(vec![("*".into(), required_fields)]);

        Input {
            language: "Solidity".into(),
            settings: InputSettings {
                optimizer: None,
                libraries: HashMap::new(),
                output_selection: HashMap::from_iter(vec![("*".into(), cntr_sel)]),
            },
            sources: HashMap::new(),
        }
    }

    pub fn optimizer(mut self, runs: usize) -> Self {
        if runs == 0 {
            self.settings.optimizer = None;
        } else {
            self.settings.optimizer = Some(OptimizerSetting { enabled: true, runs });
        }
        self
    }

    pub fn source(mut self, filename: &str, content: String) -> Self {
        self.sources.insert(filename.into(), InputSource { content });
        self
    }
}
