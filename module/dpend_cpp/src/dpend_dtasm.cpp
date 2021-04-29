// Copyright 2021 Siemens AG
// SPDX-License-Identifier: MIT

#include "flatbuffers/flatbuffers.h"

#include "dtasm_generated.h"

#include "modelDescription.h"

#include "dpend_dtasm.h"
#include "model_info.h"
#include "dpend.h"

#define WASM_EXPORT __attribute__((used)) __attribute__((visibility ("default")))

namespace DTAPI = DtasmApi;
namespace DTT = DtasmTypes;
namespace DTMD = DtasmModelDescription;
namespace FB = flatbuffers;
namespace SM = simModule;


namespace dpend_dtasm 
{
std::shared_ptr<const DTMD::ModelDescription> dtMd(nullptr);
static dp_state state = {};
std::shared_ptr<flatbuffers::FlatBufferBuilder> builder(nullptr);


WASM_EXPORT
uint8_t* alloc(size_t len){
    return (uint8_t*)malloc(len);
}

WASM_EXPORT
void dealloc(uint8_t* p){
    free(p);
}

WASM_EXPORT
int getModelDescription(uint8_t* out_p, int out_max_len)
{
    ensure_md_init();
    if ((int)modelDescription_fb_len <= out_max_len)
    {
        std::memcpy(out_p, modelDescription_fb, modelDescription_fb_len);
    }
    
    return modelDescription_fb_len;
}


WASM_EXPORT
int init(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len)
{
    auto initReq = FB::GetRoot<DTAPI::InitReq>(in_p);
    state.t = initReq->starttime();

    ensure_md_init();

    // read variables, default and initial values from model description
    map_scalar_vars(*dtMd, 
        state.map_id_var, 
        state.map_var_id, 
        state.var_values, 
        state.var_defaults);

    // get inital values from request and overwrite those from model description    
    auto initValsReal = initReq->init_values()->real_vals();
    for (unsigned int i=0; i<initValsReal->Length(); ++i)
    {
        int id = initValsReal->Get(i)->id();
        double val = initValsReal->Get(i)->val();

        e_dpvar variable = state.map_id_var[id];
        state.var_values[variable] = val;
    }

    if (state.var_values.count(e_dpvar::m1))
        state.params.m1 = state.var_values[e_dpvar::m1];

    if (state.var_values.count(e_dpvar::m2))
        state.params.m2 = state.var_values[e_dpvar::m2];

    if (state.var_values.count(e_dpvar::l1))
        state.params.l1 = state.var_values[e_dpvar::l1];

    if (state.var_values.count(e_dpvar::l2))
        state.params.l2 = state.var_values[e_dpvar::l2];

    // Todo: Convert initVals to something that can be easily used to 
    // set initial values during initialization stage

    /* Careful: usage of C++ streams increases wasm size by ~250kB */
    //std::cout << "Module id: " << id << std::endl;
    /* Better: */
    //printf("Module id: %s\n", id.c_str());

    // Build response
    // FB::FlatBufferBuilder builder;
    ensure_builder_init();

    auto res = DTAPI::CreateStatusRes(*builder, true ? DTT::Status_OK : DTT::Status_Error);
    // fprintf(stderr, "parse request...\n");
    builder->Finish(res);

    auto ptr = builder->GetBufferPointer();
    auto size = (int)builder->GetSize();

    //printf("Maxlen: %d, len: %d \n", out_max_len, size);

    if (size <= out_max_len)
    {
        std::memcpy(out_p, ptr, size);
    }
    builder->Reset();

    return size;
}


WASM_EXPORT
int setValues(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len)
{
    auto setReq = FB::GetRoot<DTAPI::SetValuesReq>(in_p);
    ensure_md_init();

    auto setValsReal = setReq->values()->real_vals();
    for (unsigned int i=0; i<setValsReal->Length(); ++i)
    {
        int id = setValsReal->Get(i)->id();
        double val = setValsReal->Get(i)->val();

        e_dpvar variable = state.map_id_var[id];
        state.var_values[variable] = val;
    }

    ensure_builder_init();

    auto res = DTAPI::CreateStatusRes(*builder, true ? DTT::Status_OK : DTT::Status_Error);
    builder->Finish(res);

    auto ptr = builder->GetBufferPointer();
    auto size = (int)builder->GetSize();

    if (size <= out_max_len)
    {
        std::memcpy(out_p, ptr, size);
    }
    builder->Reset();

    return size;
}


WASM_EXPORT
int getValues(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len)
{
    ensure_md_init();

    bool getOk = true;

    auto getValReq = FB::GetRoot<DTAPI::GetValuesReq>(in_p);
    auto fbIds = getValReq->ids();

    std::map<int, double> req_vals;
    
    for (unsigned int i=0; i<fbIds->Length(); ++i)
    {
        int id = fbIds->Get(i);

        if (!state.map_id_var.count(id))
        {
            getOk = false;
            continue;      
        }
        auto variable = state.map_id_var[id];
        double val = state.var_values[variable];
        req_vals.insert(std::make_pair(id, val));
    }

    // Build response
    //FB::FlatBufferBuilder builder(1024);
    FB::Offset<DTT::VarValues> varVals;
    // FB::FlatBufferBuilder builder;
    ensure_builder_init();

    if (getOk)
    {
        FB::Offset<FB::Vector<FB::Offset<DTT::RealVal>>> realVals;
        std::vector<FB::Offset<DTT::RealVal>> reals;
        std::map<int, double>::iterator it = req_vals.begin();
        while (it != req_vals.end())
        {
            auto realVal = DTT::CreateRealVal(*builder, it->first, it->second);
            reals.push_back(realVal);
            it++;
        }
        realVals = builder->CreateVector(reals);

        varVals = DTT::CreateVarValues(*builder, realVals);
    }

    auto res = DTAPI::CreateGetValuesRes(*builder, getOk ? DTT::Status_OK : DTT::Status_Error, 
        state.t, getOk ? varVals : 0);
    
    builder->Finish(res);

    auto ptr = builder->GetBufferPointer();
    auto size = (int)builder->GetSize();

    if (size <= out_max_len)
    {
        std::memcpy(out_p, ptr, size);
    }
    builder->Reset();

    return size;
}


WASM_EXPORT
int doStep(uint8_t* in_p, int in_len, uint8_t* out_p, int out_max_len)
{
    auto doStepReq = FB::GetRoot<DTAPI::DoStepReq>(in_p);

    auto currentTime = doStepReq->current_time();
    auto step = doStepReq->timestep();

    if (dtMd == nullptr)
    {
        printf("Dtasm module not initialized");
        exit(1);
    }

    // if (state.t - (float)currentTime > 1.0e-5)
    // {
    //     // printf("Supplied timestep does not match internal state");
    //     exit(1);
    // }

    dpend_state st;
    st.t = state.t;
    st.th1 = state.var_values[e_dpvar::th1];
    st.th2 = state.var_values[e_dpvar::th2];
    st.w1 = state.var_values[e_dpvar::w1];
    st.w2 = state.var_values[e_dpvar::w2];

    dpend_input in;
    in.a1 = state.var_values[e_dpvar::a1];
    in.a2 = state.var_values[e_dpvar::a2];
    in.dt = step;

    dp_step(&state.params, &st, &in);

    state.t = st.t;
    state.var_values[e_dpvar::th1] = st.th1;
    state.var_values[e_dpvar::th2] = st.th2;
    state.var_values[e_dpvar::w1] = st.w1;
    state.var_values[e_dpvar::w2] = st.w2;

    // FB::FlatBufferBuilder builder;
    ensure_builder_init();
    auto res = DTAPI::CreateDoStepRes(*builder, DTT::Status::Status_OK, st.t);
    builder->Finish(res);

    auto ptr = builder->GetBufferPointer();
    auto size = (int)builder->GetSize();

    //printf("Maxlen: %d, len: %d \n", out_max_len, size);

    if (size <= out_max_len)
    {
        std::memcpy(out_p, ptr, size);
    }
    builder->Reset();

    return size;
}


void ensure_builder_init(){
    if (builder == nullptr)
        builder = std::make_shared<flatbuffers::FlatBufferBuilder> ();
}

void ensure_md_init(){
    if (dtMd == nullptr)
        dtMd = std::shared_ptr<const DTMD::ModelDescription>(
            FB::GetRoot<DTMD::ModelDescription>(modelDescription_fb));
}

}