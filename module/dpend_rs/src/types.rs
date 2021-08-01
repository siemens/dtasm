// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use dtasm_rs::model_description::ModelDescription;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum DpVar {TH1, TH2, W1, W2, A1, A2, M1, M2, L1, L2, TH10, TH20}

#[derive(Default, Debug)]
pub struct DpendParams
{
    pub m1: f64,
    pub m2: f64, 
    pub l1: f64,
    pub l2: f64
}

#[derive(Debug)]
pub struct DpVarMaps
{
    pub map_id_var: HashMap<i32, DpVar>
}

impl DpVarMaps {
    pub fn new() -> DpVarMaps {
        DpVarMaps {
            map_id_var: HashMap::new(), 
        }
    }
}

#[derive(Debug)]
pub struct DpState 
{
    pub params: DpendParams, 
    pub t: f64,

    pub var_maps: DpVarMaps,
    pub var_values: HashMap<DpVar, f64>,
    pub var_defaults: HashMap<DpVar, f64>
}

impl DpState {
    pub fn new() -> DpState {
        DpState {
            params: DpendParams::default(),
            t: 0.0, 

            var_maps: DpVarMaps::new(),
            var_values: HashMap::new(),
            var_defaults: HashMap::new()
        }
    }
}

pub fn create_var_maps(dt_md: &ModelDescription,
    var_maps: &mut DpVarMaps) {
    let md_vars = &dt_md.variables;

    for model_var in md_vars {
        let name = &model_var.name;
        let id = model_var.id;

        let dp_var: DpVar;

        if name == "theta1"{
            dp_var = DpVar::TH1;
        }
        else if name == "theta2"{
            dp_var = DpVar::TH2;
        }
        else if name == "theta1_0_Value"{
            dp_var = DpVar::TH10;
        }
        else if name == "theta2_0_Value"{
            dp_var = DpVar::TH20;
        }
        else if name == "joint1.velocity" {
            dp_var = DpVar::W1;
        }
        else if name == "joint2.velocity"{
            dp_var = DpVar::W2;
        }
        else if name == "joint1.acceleration"{
            dp_var = DpVar::A1;
        }
        else if name == "joint2.acceleration"{
            dp_var = DpVar::A2;
        }
        else if name == "m1_Value"{
            dp_var = DpVar::M1;
        }
        else if name == "m2_Value"{
            dp_var = DpVar::M2;
        }
        else if name == "l1_Value"{
            dp_var = DpVar::L1;
        }
        else if name == "l2_Value"{
            dp_var = DpVar::L2;
        }
        else {
            continue;
        }

        var_maps.map_id_var.insert(id, dp_var);
    }
}

pub fn create_default_vals(dt_md: &ModelDescription,
    dp_state: &mut DpState) {

    let mod_desc_vars = &dt_md.variables;
    let var_maps = &dp_state.var_maps;
    let default_vals = &mut dp_state.var_defaults;
    let init_vals = &mut dp_state.var_values;

    for scalar_var in mod_desc_vars {
        let dp_var = var_maps.map_id_var.get(&scalar_var.id)
            .expect(&format!("Unknown variable {}", scalar_var.name));

        match &scalar_var.default {
            None => {}, 
            Some(def) => {
                default_vals.insert(*dp_var, def.real_val);
                init_vals.insert(*dp_var, def.real_val);
            }
        }
    }
}