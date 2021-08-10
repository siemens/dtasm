// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::process::Command;
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut md_json_path = PathBuf::from(crate_dir.clone());
    md_json_path.push("src");
    md_json_path.push("modelDescription.json");

    let mut flatc_path = PathBuf::from(crate_dir.clone());
    flatc_path.push("..");
    flatc_path.push("..");
    flatc_path.push("third_party");
    flatc_path.push("flatbuffers.git");
    flatc_path.push("flatc");

    let mut fbs_path = PathBuf::from(crate_dir.clone());
    fbs_path.push("..");
    fbs_path.push("..");
    fbs_path.push("lib");
    fbs_path.push("dtasm_abi");
    fbs_path.push("schema");
    fbs_path.push("dtasm.fbs");

    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target");
    let mut out_file_tmp = out_path.clone();
    out_file_tmp.push("modelDescription.bin");
    let mut out_file = out_path.clone();
    out_file.push("modelDescription.fb");

    let mut convert = Command::new(flatc_path);
    convert.arg("-b")
        .arg("-o")
        .arg(out_path)
        .arg(fbs_path)
        .arg(md_json_path)
        .output()
        .expect("Failed to execute command");

    let mut mv = Command::new("mv");
    mv.arg(out_file_tmp)
        .arg(out_file)
        .output()
        .expect("Failed to execute command");
}