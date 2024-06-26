// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use crate::messaging::{EnvManager, PythonEnvironment};
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub struct PythonEnv {
    pub executable: PathBuf,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}

impl PythonEnv {
    pub fn new(executable: PathBuf, path: Option<PathBuf>, version: Option<String>) -> Self {
        Self {
            executable,
            path,
            version,
        }
    }
}

#[derive(Debug)]
pub struct PyEnvCfg {
    pub version: String,
}

const PYVENV_CONFIG_FILE: &str = "pyvenv.cfg";

pub fn find_pyvenv_config_path(python_executable: &PathBuf) -> Option<PathBuf> {
    // Check if the pyvenv.cfg file is in the parent directory relative to the interpreter.
    // env
    // |__ pyvenv.cfg  <--- check if this file exists
    // |__ bin or Scripts
    //     |__ python  <--- interpreterPath
    let cfg = python_executable.parent()?.join(PYVENV_CONFIG_FILE);
    if fs::metadata(&cfg).is_ok() {
        return Some(cfg);
    }

    // Check if the pyvenv.cfg file is in the directory as the interpreter.
    // env
    // |__ pyvenv.cfg  <--- check if this file exists
    // |__ python  <--- interpreterPath
    let cfg = python_executable
        .parent()?
        .parent()?
        .join(PYVENV_CONFIG_FILE);
    if fs::metadata(&cfg).is_ok() {
        return Some(cfg);
    }

    None
}

pub fn find_and_parse_pyvenv_cfg(python_executable: &PathBuf) -> Option<PyEnvCfg> {
    let cfg = find_pyvenv_config_path(&PathBuf::from(python_executable))?;
    if !fs::metadata(&cfg).is_ok() {
        return None;
    }

    let contents = fs::read_to_string(&cfg).ok()?;
    let version_regex = Regex::new(r"^version\s*=\s*(\d+\.\d+\.\d+)$").unwrap();
    let version_info_regex = Regex::new(r"^version_info\s*=\s*(\d+\.\d+\.\d+.*)$").unwrap();
    for line in contents.lines() {
        if !line.contains("version") {
            continue;
        }
        if let Some(captures) = version_regex.captures(line) {
            if let Some(value) = captures.get(1) {
                return Some(PyEnvCfg {
                    version: value.as_str().to_string(),
                });
            }
        }
        if let Some(captures) = version_info_regex.captures(line) {
            if let Some(value) = captures.get(1) {
                return Some(PyEnvCfg {
                    version: value.as_str().to_string(),
                });
            }
        }
    }
    None
}

pub fn get_version(python_executable: &PathBuf) -> Option<String> {
    if let Some(parent_folder) = python_executable.parent() {
        if let Some(pyenv_cfg) = find_and_parse_pyvenv_cfg(&parent_folder.to_path_buf()) {
            return Some(pyenv_cfg.version);
        }
    }

    let output = Command::new(python_executable)
        .arg("-c")
        .arg("import sys; print(sys.version)")
        .output()
        .ok()?;
    let output = String::from_utf8(output.stdout).ok()?;
    let output = output.trim();
    let output = output.split_whitespace().next().unwrap_or(output);
    Some(output.to_string())
}

pub fn find_python_binary_path(env_path: &Path) -> Option<PathBuf> {
    let python_bin_name = if cfg!(windows) {
        "python.exe"
    } else {
        "python"
    };
    let path_1 = env_path.join("bin").join(python_bin_name);
    let path_2 = env_path.join("Scripts").join(python_bin_name);
    let path_3 = env_path.join(python_bin_name);
    let paths = vec![path_1, path_2, path_3];
    paths.into_iter().find(|path| path.exists())
}

pub fn list_python_environments(path: &PathBuf) -> Option<Vec<PythonEnv>> {
    let mut python_envs: Vec<PythonEnv> = vec![];
    for venv_dir in fs::read_dir(path).ok()? {
        if let Ok(venv_dir) = venv_dir {
            let venv_dir = venv_dir.path();
            if !venv_dir.is_dir() {
                continue;
            }
            if let Some(executable) = find_python_binary_path(&venv_dir) {
                python_envs.push(PythonEnv::new(
                    executable.clone(),
                    Some(venv_dir),
                    get_version(&executable),
                ));
            }
        }
    }

    Some(python_envs)
}

pub fn get_environment_key(env: &PythonEnvironment) -> Option<String> {
    if let Some(ref path) = env.python_executable_path {
        return Some(path.to_string_lossy().to_string());
    }
    if let Some(ref path) = env.env_path {
        return Some(path.to_string_lossy().to_string());
    }

    None
}

pub fn get_environment_manager_key(env: &EnvManager) -> String {
    return env.executable_path.to_string_lossy().to_string();
}
