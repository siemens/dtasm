// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

#ifndef DTASM_MODULE_H
#define DTASM_MODULE_H

#include <map>
#include <stdint.h>

#include "flatbuffers/flatbuffers.h"

#include "dtasm_generated.h"
#include "dpend.h"

namespace dpend_dtasm
{
#ifdef __cplusplus
extern "C" {
#endif

uint8_t* alloc(size_t len);
void dealloc(uint8_t* p);
int init(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len);
int doStep(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len);
int getValues(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len);
int setValues(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len);
int resetStep(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len);
int terminate(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len);
int getModelDescription(uint8_t* out_p, int out_max_len);

#ifdef __cplusplus
}  /* end of extern "C" { */
#endif

enum e_dpvar {th1, th2, w1, w2, a1, a2, m1, m2, l1, l2};

typedef struct dp_state 
{
    dpend_params params;
    double t;

    std::map<int, e_dpvar> map_id_var;
    std::map<e_dpvar, int> map_var_id;
    std::map<e_dpvar, double> var_values;
    std::map<e_dpvar, double> var_defaults;
} dp_state;

void ensure_builder_init();
void ensure_md_init();

void map_scalar_vars(const DtasmModelDescription::ModelDescription &dtMd, 
    std::map<int, e_dpvar>& map_id_var, 
    std::map<e_dpvar, int>& map_var_id,
    std::map<e_dpvar, double>& var_values,
    std::map<e_dpvar, double>& var_defaults){
    auto modDescScalars = dtMd.variables();

    for (unsigned int i=0; i<modDescScalars->Length(); ++i)
    {
        auto scalarVar = modDescScalars->Get(i);
        std::string name = scalarVar->name()->str();
        int id = scalarVar->id();

        e_dpvar variable;

        if (name == "theta1")
            variable = e_dpvar::th1;
        else if (name == "theta2")
            variable = e_dpvar::th2;
        else if (name == "joint1.velocity")
            variable = e_dpvar::w1;
        else if (name == "joint2.velocity")
            variable = e_dpvar::w2;
        else if (name == "joint1.acceleration")
            variable = e_dpvar::a1;
        else if (name == "joint2.acceleration")
            variable = e_dpvar::a2;
        else if (name == "m1_Value")
            variable = e_dpvar::m1;
        else if (name == "m2_Value")
            variable = e_dpvar::m2;
        else if (name == "l1_Value")
            variable = e_dpvar::l1;
        else if (name == "l2_Value")
            variable = e_dpvar::l2;

        map_id_var.insert({id, variable});
        map_var_id.insert({variable, id});

        if (scalarVar->default_() != NULL)
        {
            double def_val = scalarVar->default_()->real_val();
            var_defaults.insert(std::make_pair(variable, def_val));
            var_values.insert(std::make_pair(variable, def_val));
        }
    }
};
}
#endif