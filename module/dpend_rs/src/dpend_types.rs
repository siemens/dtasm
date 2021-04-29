// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

use dtasm_abi::dtasm_generated::dtasm_model_description as DTMD;

use std::collections::HashMap;


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
    pub map_id_var: HashMap<i32, DpVar>,
    pub map_var_id: HashMap<DpVar, i32>
}

impl DpVarMaps {
    pub fn new() -> DpVarMaps {
        DpVarMaps {
            map_id_var: HashMap::new(), 
            map_var_id: HashMap::new()
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

pub fn create_var_maps(dt_md: &DTMD::ModelDescription,
    var_maps: &mut DpVarMaps) {

    let mod_desc_scalars = dt_md.variables();

    for i in 0..mod_desc_scalars.len() {
        let scalar_var = mod_desc_scalars.get(i);
        let name = scalar_var.name();
        let id = scalar_var.id();

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

        let id2 = id.clone();
        var_maps.map_id_var.insert(id, dp_var);
        var_maps.map_var_id.insert(dp_var, id2);
    }
}