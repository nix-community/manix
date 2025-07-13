use crate::{
    contains_insensitive_ascii, starts_with_insensitive_ascii, Cache, DocEntry, DocSource, Errors,
    Lowercase,
};
use colored::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptionDocumentation {
    #[serde(default)]
    description: String,

    #[serde(default, rename(serialize = "readOnly", deserialize = "readOnly"))]
    read_only: bool,

    #[serde(rename(serialize = "loc", deserialize = "loc"))]
    location: Vec<String>,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    option_type: String,
}

impl OptionDocumentation {
    pub fn name(&self) -> String {
        self.location.join(".")
    }
    pub fn pretty_printed(&self) -> String {
        format!(
            "# {}\n{}\ntype: {}\n\n",
            self.name().blue().bold(),
            self.description,
            self.option_type
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionsDatabaseType {
    NixOS,
    NixDarwin,
    HomeManager,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsDatabase {
    pub typ: OptionsDatabaseType,
    pub options: HashMap<String, OptionDocumentation>,
}

impl OptionsDatabase {
    pub fn new(typ: OptionsDatabaseType) -> Self {
        Self {
            typ,
            options: HashMap::new(),
        }
    }
}

pub fn try_from_file(path: &PathBuf) -> Result<HashMap<String, OptionDocumentation>, Errors> {
    let options: HashMap<String, OptionDocumentation> =
        serde_json::from_slice(&std::fs::read(path)?)?;
    Ok(options)
}

impl DocSource for OptionsDatabase {
    fn all_keys(&self) -> Vec<&str> {
        self.options.keys().map(|x| x.as_ref()).collect()
    }
    fn search(&self, query: &Lowercase) -> Vec<DocEntry> {
        self.options
            .iter()
            .filter(|(key, _)| starts_with_insensitive_ascii(key.as_bytes(), query))
            .map(|(_, d)| DocEntry::OptionDoc(self.typ, d.clone()))
            .collect()
    }
    fn search_liberal(&self, query: &Lowercase) -> Vec<DocEntry> {
        self.options
            .iter()
            .filter(|(key, _)| contains_insensitive_ascii(key.as_bytes(), query))
            .map(|(_, d)| DocEntry::OptionDoc(self.typ, d.clone()))
            .collect()
    }
    fn update(&mut self) -> Result<bool, Errors> {
        let opts = match self.typ {
            OptionsDatabaseType::NixOS => try_from_file(&get_nixos_json_doc_path()?)?,
            OptionsDatabaseType::NixDarwin => try_from_file(&get_nd_json_doc_path()?)?,
            OptionsDatabaseType::HomeManager => try_from_file(&get_hm_json_doc_path()?)?,
        };

        let old = std::mem::replace(&mut self.options, opts);

        Ok(old.keys().eq(self.options.keys()))
    }
}

impl Cache for OptionsDatabase {}

pub fn get_hm_json_doc_path() -> Result<PathBuf, std::io::Error> {
    let base_path_res = Command::new("nix-build")
        .env("NIXPKGS_ALLOW_UNFREE", "1")
        .env("NIXPKGS_ALLOW_BROKEN", "1")
        .env("NIXPKGS_ALLOW_INSECURE", "1")
        .arg("-E")
        .arg(include_str!("nix/hm-options.nix"))
        .output()?;

    let base_path_output = if base_path_res.status.success() {
        let path = String::from_utf8_lossy(&base_path_res.stdout);
        PathBuf::from(path.trim_end_matches('\n'))
    } else {
        // we may still get the options from profile if user set manual.json.enable
        let path = std::env::var("HOME")
            .map(|home| Path::new(&home).join(".nix-profile"))
            .expect("HOME must be set");

        if path.join("share/doc/home-manager/options.json").exists() {
            path
        } else {
            // propagate error from
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&base_path_res.stderr),
            ));
        }
    };

    Ok(base_path_output.join("share/doc/home-manager/options.json"))
}

pub fn get_nixos_json_doc_path() -> Result<PathBuf, std::io::Error> {
    let base_path_output = Command::new("nix-build")
        .env("NIXPKGS_ALLOW_UNFREE", "1")
        .env("NIXPKGS_ALLOW_BROKEN", "1")
        .env("NIXPKGS_ALLOW_INSECURE", "1")
        .arg("--no-out-link")
        .arg("-E")
        .arg(include_str!("nix/nixos-options.nix"))
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())?;

    Ok(PathBuf::from(base_path_output.trim_end_matches('\n')))
}

pub fn get_nd_json_doc_path() -> Result<PathBuf, std::io::Error> {
    let base_path_output = Command::new("nix-build")
        .env("NIXPKGS_ALLOW_UNFREE", "1")
        .env("NIXPKGS_ALLOW_BROKEN", "1")
        .env("NIXPKGS_ALLOW_INSECURE", "1")
        .env("NIXPKGS_ALLOW_UNSUPPORTED_SYSTEM", "1")
        .arg("--no-out-link")
        .arg("-E")
        .arg(include_str!("nix/darwin-options.nix"))
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())?;

    println!("{}", base_path_output.trim_end_matches('\n'));

    Ok(PathBuf::from(base_path_output.trim_end_matches('\n')))
}
