// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use dtasmtime::runtime::{Engine, Module};
use dtasmtime::model_description as MD;
use dtasmtime::types::{DtasmVarValues, LogLevel};

use anyhow::Result;
use structopt::StructOpt;

use std::{collections::HashMap, fs::File};
use std::path::PathBuf;


#[derive(Debug, StructOpt)]
#[structopt(name = "dtasmtime", about = "A wasmtime-based runtime for dtasm modules")]
struct Opt {
    #[structopt(long, default_value = "0.0")]
    tmin: f64,
    #[structopt(long, default_value = "10.0")]
    tmax: f64,
    #[structopt(long, default_value = "0.02")]
    dt: f64,
    #[structopt(long, parse(from_os_str), default_value = "")]
    csv: PathBuf,
    #[structopt(long, parse(from_os_str), default_value = "")]
    state_to: PathBuf,
    #[structopt(long, parse(from_os_str), default_value = "")]
    state_from: PathBuf,
    #[structopt(long, parse(from_os_str))]
    input: PathBuf,
    parameters: Vec<String>
}


fn main() -> Result<()> {

    let opt = Opt::from_args();

    let tmin = opt.tmin;
    let tmax = opt.tmax;
    let dt = opt.dt;

    // println!("Parameters: {:#?}", opt.parameters);

    let mut t = tmin;
    let n_steps = ((tmax-tmin)/dt).round() as i32;

    let engine = Engine::new().expect("Could not instantiate dtasm engine");
    let mut dtasm_module = Module::new(opt.input, &engine).expect("Could not instantiate dtasm module");
    let mut inst = dtasm_module.instantiate()?;

    let md = inst.get_model_description()?;
    println!("Received model description: {:#?}", md);

    let mut csv_wtr: Option<csv::Writer<File>> = None;
    let mut csv_var_names: Vec<(i32, String, MD::VariableType)> = Vec::new();

    if opt.csv.to_str() != Some("") {
        let wtr = csv::Writer::from_path(opt.csv)?;
        // write_header(&mut wtr, &md.variables, &mut csv_var_names)?;
        csv_wtr = Some(wtr);
    }
    write_header(&mut csv_wtr, &md.variables, &mut csv_var_names)?;

    let mut init_vals = extract_default_vals(&md.variables, 
        &vec![ MD::CausalityType::Local, MD::CausalityType::Input ]);

    let def_inputs = extract_default_vals(&md.variables, 
        &vec![ MD::CausalityType::Input ]);

    let cmd_vals = parse_cmd_parameters(&opt.parameters, &md.variables);
    update_var_values(&mut init_vals, &cmd_vals);

    let _init_res = inst.initialize(&init_vals, tmin, Some(tmax), None, LogLevel::Warn, true)?;

    if opt.state_from.to_str() != Some("") {
        inst.load_state(opt.state_from)?;
        t = inst.get_values(&Vec::new()).expect("Could not retrieve values after loading state").current_time;
        println!("Successfully loaded state from state.bin");
    }

    println!("Init return status: {:#?}", _init_res);

    let mut var_ids = Vec::new();
    let mut var_names: Vec<String> = Vec::new();
    let mut input_var_ids = Vec::new();
    let mut input_var_names: Vec<String> = Vec::new();

    for variable in &md.variables {
        if variable.causality == MD::CausalityType::Output || 
            variable.causality == MD::CausalityType::Local {
            var_ids.push(variable.id);
            var_names.push(variable.name.to_string());
        }
        else if variable.causality == MD::CausalityType::Input {
            input_var_ids.push(variable.id);
            input_var_names.push(variable.name.to_string());
        }
    }

    let mut get_vals = inst.get_values(&var_ids)?;
    write_record(&mut csv_wtr, &csv_var_names, &get_vals.values, get_vals.current_time)?;

    for _ in 0..n_steps {
        inst.set_values(&def_inputs)?;
        let dostep_res = inst.do_step(t,dt)?;
        get_vals = inst.get_values(&var_ids)?;
        write_record(&mut csv_wtr, &csv_var_names, &get_vals.values, get_vals.current_time)?;
        t = dostep_res.updated_time;
    }

    if opt.state_to.to_str() != Some("") {
        inst.save_state(opt.state_to)?;
        println!("Successfully wrote state to state.bin");
    }

    Ok(())
}

fn extract_default_vals(vars: &Vec<MD::ModelVariable>, causalities: &Vec<MD::CausalityType>) -> DtasmVarValues {
    let mut default_vals = DtasmVarValues::new();

    for variable in vars {
        if !causalities.contains(&variable.causality) {
            continue;
        }

        match &variable.default {
            None => {}, 
            Some(default) => {
                match variable.value_type {
                    MD::VariableType::DtasmReal => { default_vals.real_values.insert(variable.id, default.real_val); },
                    MD::VariableType::DtasmInt => { default_vals.int_values.insert(variable.id, default.int_val); },
                    MD::VariableType::DtasmBool => { default_vals.bool_values.insert(variable.id, default.bool_val); },
                    MD::VariableType::DtasmString => { default_vals.string_values.insert(variable.id, default.string_val.clone()); },
                };
            }
        };
    }

    default_vals
}

fn parse_cmd_parameters(params: &Vec<String>, vars: &Vec<MD::ModelVariable>) -> DtasmVarValues {
    let mut kv_pairs: HashMap<String, String> = HashMap::new();
    let mut id_vals = DtasmVarValues::new();

    for kv_str in params {
        let kv = kv_str.split('=');
        let kv_vec: Vec<&str> = kv.collect();
        assert!(kv_vec.len() == 2, "Invalid parameter format: {}", kv_str);
        
        kv_pairs.insert(kv_vec[0].to_string(), kv_vec[1].to_string());
    }

    for variable in vars {
        if kv_pairs.contains_key(&variable.name) {
            let val_str = &kv_pairs[&variable.name];

            match variable.value_type {
                MD::VariableType::DtasmReal => {
                    let val: f64 = val_str.parse().expect("Could not parse cmd line argument");
                    id_vals.real_values.insert(variable.id, val);
                },
                MD::VariableType::DtasmInt => {
                    let val: i32 = val_str.parse().expect("Could not parse cmd line argument");
                    id_vals.int_values.insert(variable.id, val);
                },
                MD::VariableType::DtasmBool => {
                    let val: bool = val_str.parse().expect("Could not parse cmd line argument");
                    id_vals.bool_values.insert(variable.id, val);
                },
                MD::VariableType::DtasmString => {
                    id_vals.string_values.insert(variable.id, val_str.to_string());
                }
            }
        }
    }

    id_vals
}

fn update_var_values(values: &mut DtasmVarValues, other: &DtasmVarValues) {
    for (id, val) in &other.real_values {
        values.real_values.insert(*id, *val);
    }
    for (id, val) in &other.int_values {
        values.int_values.insert(*id, *val);
    }
    for (id, val) in &other.bool_values {
        values.bool_values.insert(*id, *val);
    }
    for (id, val) in &other.string_values {
        values.string_values.insert(*id, val.clone());
    }
}

fn write_header(csv_wtr: &mut Option<csv::Writer<File>>, 
    variables: &Vec<MD::ModelVariable>, 
    var_names: &mut Vec<(i32, String, MD::VariableType)>) -> Result<()> {

    for variable in variables {
        if variable.causality == MD::CausalityType::Output || variable.causality == MD::CausalityType::Local {
            var_names.push((variable.id, variable.name.to_string(), variable.value_type));
        }
    }

    var_names.sort_by(|a,b| a.0.cmp(&b.0));

    let mut header: Vec<String> = Vec::new();
    header.push("t".to_string());
    for (_var_id, var_name, _var_type) in var_names {
        header.push(var_name.to_string());
    }

    if let Some(ref mut wtr) = csv_wtr {
        match wtr.write_record(header) {
            Ok(()) => Ok(()), 
            Err(_e) => Err(anyhow::format_err!("Could not write to csv-file"))
        }
    }
    else
    {
        println!("{:?}", header);
        Ok(())
    }
}

fn write_record(csv_wtr: &mut Option<csv::Writer<File>>, 
    var_names: &Vec<(i32, String, MD::VariableType)>, 
    var_values: &DtasmVarValues, 
    t: f64) -> Result<()> {

    let mut line: Vec<String> = Vec::new();
    line.push(format!("{:.8}", t));

    for (var_id, _var_name, var_type) in var_names {
        match var_type {
            MD::VariableType::DtasmReal => {
                let val = var_values.real_values[var_id];
                line.push(format!("{:.8}", val));
            },
            MD::VariableType::DtasmInt => {
                let val = var_values.int_values[var_id];
                line.push(val.to_string());
            },
            MD::VariableType::DtasmBool => {
                let val = var_values.bool_values[var_id];
                line.push(val.to_string());
            },
            MD::VariableType::DtasmString => {
                let val = &var_values.string_values[var_id];
                line.push(val.to_string());
            }
        }
    };

    if let Some(ref mut wtr) = csv_wtr {
        match wtr.write_record(line) {
            Ok(()) => Ok(()), 
            Err(_e) => Err(anyhow::format_err!("Could not write to csv-file"))
        }
    }
    else
    {
        println!("{:?}", line);
        Ok(())
    }
}