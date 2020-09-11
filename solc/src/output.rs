use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputError {
    pub r#type: String,
    component: String,
    pub severity: String,
    pub message: String,
    #[serde(rename = "formattedMessage")]
    formatted_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractBytecode {
    pub object: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractEvm {
    pub bytecode: ContractBytecode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contract {
    pub abi: serde_json::Value,
    pub evm: ContractEvm,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    #[serde(default = "Default::default")]
    pub errors: Vec<OutputError>,
    // "sourceFile.sol": {   "ContractName": ... }
    #[serde(default = "Default::default")]
    pub contracts: HashMap<String, HashMap<String, Contract>>,
}

impl Output {
    pub fn has_errors(&self) -> bool {
        self.errors.iter().find(|err| err.severity == "error").is_some()
    }

    pub fn error_message(&self) -> String {
        self.errors
            .iter()
            .filter(|err| err.severity == "error")
            .map(|err| err.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn format_errors(&self) {
        for err in &self.errors {
            eprintln!("{}: {}", err.severity, err.message);
            if let Some(ref fmsg) = err.formatted_message {
                eprintln!("{}", fmsg);
            }
        }
    }

    pub fn abi_for(&self, cntr_name: &str) -> Result<String, io::Error> {
        for (_fname, cntrs) in &self.contracts {
            if cntrs.contains_key(cntr_name) {
                return serde_json::to_string(&cntrs[cntr_name].abi)
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "abi not found"));
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "abi not found"))
    }

    pub fn pretty_abi_for(&self, cntr_name: &str) -> Result<String, io::Error> {
        for (_fname, cntrs) in &self.contracts {
            if cntrs.contains_key(cntr_name) {
                return serde_json::to_string_pretty(&cntrs[cntr_name].abi)
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "abi not found"));
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "abi not found"))
    }

    pub fn bytecode_for(&self, cntr_name: &str) -> Result<&str, io::Error> {
        for (_fname, cntrs) in &self.contracts {
            if cntrs.contains_key(cntr_name) {
                return Ok(&cntrs[cntr_name].evm.bytecode.object);
            }
        }
        Err(io::Error::new(io::ErrorKind::Other, "bytecode not found"))
    }
}
