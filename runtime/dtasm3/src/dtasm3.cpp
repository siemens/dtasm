#include "wasm3.h"
#include "m3_env.h"
#include "wasm3_cpp.h"

#include "flatbuffers/flatbuffers.h"

#include "dtasm_generated.h"
#include "dtasm3.h"

#include <fstream>
#include <sstream>
#include <vector>
#include <exception>

#define WASM_PAGE_SIZE 65536

namespace FB = flatbuffers;
namespace DTMD = DtasmModelDescription;
namespace DTT = DtasmTypes;
namespace DTAPI = DtasmApi;


class dtasm3::Runtime::Impl {
private: 
    wasm3::runtime m_m3Runtime;
    FB::FlatBufferBuilder m_builder;
    size_t m_buffersize;
    size_t m_outputMem;
    size_t m_inputMem;

    std::unique_ptr<wasm3::function> m_allocFn;
    std::unique_ptr<wasm3::function> m_deallocFn;
    std::unique_ptr<wasm3::function> m_getModelDescriptionFn;
    std::unique_ptr<wasm3::function> m_initFn;
    std::unique_ptr<wasm3::function> m_getValuesFn;
    std::unique_ptr<wasm3::function> m_setValuesFn;
    std::unique_ptr<wasm3::function> m_doStepFn;

    std::map<int32_t, DtasmVariableType> m_varIdType;

    DtasmModelDescription m_modelDesc;

    
    DtasmModelDescription load_model_description() {
        auto return_len = m_getModelDescriptionFn->call<int32_t>(m_outputMem, m_buffersize);

        if (return_len > m_buffersize) {
            std::stringstream errMsg;
            errMsg << "Response buffer too small; need " << return_len << " bytes, have " << m_buffersize;
            throw std::runtime_error(errMsg.str().c_str());
        }

        uint32_t memSize;
        auto memPtr = m_m3Runtime.get_memory(memSize, 0);
        if (m_outputMem + m_buffersize > memSize){
            std::string errorMsg = "Response data overflowing linear memory";
            throw std::runtime_error(errorMsg.c_str());
        }

        auto verifier = FB::Verifier(memPtr + m_outputMem, return_len);
        bool ok = DTMD::VerifyModelDescriptionBuffer(verifier);
        if (ok) {
            std::cout << "Model description verifies ok" << std::endl;
        }
        else {
            std::string errorMsg = "Model description invalid";
            throw std::runtime_error(errorMsg.c_str());
        }

        auto dtMd = FB::GetRoot<DTMD::ModelDescription>(memPtr + m_outputMem);
        return mdFbToDtasm(dtMd);
    };


    DtasmModelDescription mdFbToDtasm(const DTMD::ModelDescription *fbMd) {
        DtasmModelDescription dtasmMd;

        auto exp = fbMd->experiment();
        if (exp == nullptr)
            dtasmMd.has_experiment = false;
        else {
            dtasmMd.has_experiment = true;
            dtasmMd.experiment.end_time_default = exp->endtime_default();
            dtasmMd.experiment.start_time_default = exp->starttime_default();
            dtasmMd.experiment.time_step_default = exp->timestep_default();
            dtasmMd.experiment.time_step_max = exp->timestep_max();
            dtasmMd.experiment.time_step_min = exp->timestep_min();
            dtasmMd.experiment.time_unit = exp->time_unit()->str();
        }

        auto model = fbMd->model();
        dtasmMd.model.name = model->name()->str();
        dtasmMd.model.description = model->description()->str();
        dtasmMd.model.id = model->id()->str();
        dtasmMd.model.generation_tool = model->generation_tool()->str();
        dtasmMd.model.generation_date_time = model->generation_datetime()->str();
        dtasmMd.model.name_delimiter = model->name_delimiter()->str();
        
        auto caps = model->capabilities();
        dtasmMd.model.capabilities.can_handle_variable_step_size = 
            caps->can_handle_variable_step_size();            
        dtasmMd.model.capabilities.can_interpolate_inputs = 
            caps->can_interpolate_inputs();
        dtasmMd.model.capabilities.can_reset_step = 
            caps->can_reset_step();

        auto vars = fbMd->variables();
        for (unsigned int i = 0; i < vars->size(); i++) {
            DtasmModelVariable dtasmVar; 
            auto fbVar = vars->Get(i);

            dtasmVar.id = fbVar->id();
            dtasmVar.name = fbVar->name()->str();
            dtasmVar.description = fbVar->description()->str();
            dtasmVar.unit = fbVar->unit()->str();
            dtasmVar.derivative_of_id = fbVar->derivative_of_id();

            auto fbVarType = fbVar->value_type();
            DtasmVariableValue varValue;
            dtasmVar.has_default = (fbVar->default_() != nullptr);

            switch (fbVarType) {
                case DTT::VariableType_DtasmReal:
                    dtasmVar.value_type = DtasmVariableType::DtasmReal;
                    if (dtasmVar.has_default)
                        dtasmVar.default_.real_val = fbVar->default_()->real_val();
                    break;
                case DTT::VariableType_DtasmInt:
                    dtasmVar.value_type = DtasmVariableType::DtasmInt;
                    if (dtasmVar.has_default)
                        dtasmVar.default_.int_val = fbVar->default_()->int_val();
                    break;
                case DTT::VariableType_DtasmBool:
                    dtasmVar.value_type = DtasmVariableType::DtasmBool;
                    if (dtasmVar.has_default)
                        dtasmVar.default_.bool_val = fbVar->default_()->bool_val();
                    break;
                case DTT::VariableType_DtasmString:
                    dtasmVar.value_type = DtasmVariableType::DtasmString;
                    if (dtasmVar.has_default)
                        dtasmVar.default_.string_val = fbVar->default_()->string_val()->str();
                    break;
                default:
                    std::stringstream errorMsg;
                    errorMsg << "Unknown variable type for variable " << fbVar->id() << " (" << fbVar->name()->str() << ")";
                    throw std::runtime_error(errorMsg.str().c_str());
            }

            auto fbCausType = fbVar->causality();

            switch (fbCausType) {
                case DTMD::CausalityType_parameter:
                    dtasmVar.causality = DtasmCausalityType::Parameter;
                    break;
                case DTMD::CausalityType_input:
                    dtasmVar.causality = DtasmCausalityType::Input;
                    break;
                case DTMD::CausalityType_output:
                    dtasmVar.causality = DtasmCausalityType::Output;
                    break;
                case DTMD::CausalityType_local:
                    dtasmVar.causality = DtasmCausalityType::Local;
                    break;
                default:
                    std::stringstream errorMsg;
                    errorMsg << "Unknown causality type for variable " << fbVar->id() << " (" << fbVar->name()->str() << ")";
                    throw std::runtime_error(errorMsg.str().c_str());
            }

            dtasmMd.variables.push_back(dtasmVar);
        }

        return dtasmMd;
    }


    DTT::LogLevel logLevelDtasmToFb(DtasmLogLevel logLevel) {
        switch (logLevel) {
            case DtasmLogInfo:
                return DTT::LogLevel_Info;
                break;
            case DtasmLogWarn:
                return DTT::LogLevel_Warn;
                break;
            case DtasmLogError:
                return DTT::LogLevel_Error;
                break;
            default: 
                std::stringstream errorMsg;
                errorMsg << "Unknown dtasm log level " << logLevel;
                throw std::runtime_error(errorMsg.str().c_str());
        }
    }


    DtasmStatus statusFbToDtasm(DTT::Status status) {
        switch (status) {
            case DTT::Status_OK:
                return DtasmOK;
                break;
            case DTT::Status_Discard:
                return DtasmDiscard;
                break;
            case DTT::Status_Warning:
                return DtasmWarning;
                break;
            case DTT::Status_Error:
                return DtasmError;
                break;
            default: 
                std::stringstream errorMsg;
                errorMsg << "Unknown dtasm status " << status;
                throw std::runtime_error(errorMsg.str().c_str());
        }
    }


    uint8_t* callInputOutput(wasm3::function* func, 
        uint8_t* inputBuffer, 
        int32_t inputSize) { 

        uint32_t memSize;
        auto memPtr = m_m3Runtime.get_memory(memSize, 0);
        if (m_inputMem + inputSize > memSize){
            std::string errorMsg = "Request data overflowing linear memory";
            throw std::runtime_error(errorMsg.c_str());
        }
        memcpy(memPtr + m_inputMem, inputBuffer, inputSize);

        auto resLen = func->call<int32_t>(m_inputMem, inputSize, m_outputMem, m_buffersize);
        if (resLen > m_buffersize) {
            std::stringstream errMsg;
            errMsg << "Response buffer too small; need " << resLen << " bytes, have " << m_buffersize;
            throw std::runtime_error(errMsg.str().c_str());
        }

        return memPtr + m_outputMem;
    }


    flatbuffers::Offset<DtasmTypes::VarValues> varValuesToFb(const DtasmVarValues &varValues) {
        std::vector<FB::Offset<DTT::RealVal>> reals;
        std::vector<FB::Offset<DTT::IntVal>> ints;
        std::vector<FB::Offset<DTT::BoolVal>> bools;
        std::vector<FB::Offset<DTT::StringVal>> strings;
             
        for (auto it = varValues.real_values.begin(); it != varValues.real_values.end(); it++) {
            reals.push_back(DTT::CreateRealVal(m_builder, it->first, it->second));
        }
        auto realVals = m_builder.CreateVector(reals);
            
        for (auto it = varValues.int_values.begin(); it != varValues.int_values.end(); it++) {
            ints.push_back(DTT::CreateIntVal(m_builder, it->first, it->second));
        }
        auto intVals = m_builder.CreateVector(ints);

        for (auto it = varValues.bool_values.begin(); it != varValues.bool_values.end(); it++) {
            bools.push_back(DTT::CreateBoolVal(m_builder, it->first, it->second));
        }
        auto boolVals = m_builder.CreateVector(bools);

        for (auto it = varValues.string_values.begin(); it != varValues.string_values.end(); it++) {
            strings.push_back(DTT::CreateStringValDirect(m_builder, it->first, it->second.c_str()));
        }
        auto stringVals = m_builder.CreateVector(strings);

        return DTT::CreateVarValues(m_builder, realVals, intVals, boolVals, stringVals);
    }

public: 
    Impl(Impl &&other): 
        m_m3Runtime(other.m_m3Runtime),
        m_builder(std::move(other.m_builder)),
        m_buffersize(other.m_buffersize),
        m_allocFn(std::move(other.m_allocFn)),
        m_deallocFn(std::move(other.m_deallocFn)),
        m_getModelDescriptionFn(std::move(other.m_getModelDescriptionFn)),
        m_initFn(std::move(other.m_initFn)),
        m_getValuesFn(std::move(other.m_getValuesFn)),
        m_setValuesFn(std::move(other.m_setValuesFn)),
        m_doStepFn(std::move(other.m_doStepFn)),
        m_modelDesc(other.m_modelDesc) {
                    
        m_outputMem = other.m_outputMem;
        other.m_outputMem = 0;
        m_inputMem = other.m_inputMem;
        other.m_inputMem = 0;
    }


    Impl(std::shared_ptr<wasm3::module> m3_mod, wasm3::environment m3_env, 
        size_t stack_size_bytes, int32_t buffersize): 
        m_m3Runtime(m3_env.new_runtime(stack_size_bytes)),
        m_buffersize(buffersize),
        m_builder(buffersize) {

        m_m3Runtime.load(*m3_mod);
        m_allocFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("alloc"));
        m_deallocFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("dealloc"));
        m_getModelDescriptionFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("getModelDescription"));
        m_initFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("init"));
        m_getValuesFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("getValues"));
        m_setValuesFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("setValues"));
        m_doStepFn = std::make_unique<wasm3::function>(m_m3Runtime.find_function("doStep"));

        m_outputMem = m_allocFn->call<int32_t>(m_buffersize);
        m_inputMem = m_allocFn->call<int32_t>(m_buffersize);

        m_modelDesc = load_model_description();
    }


    ~Impl() {
        if (m_inputMem > 0)
            m_deallocFn->call(m_inputMem);
        if (m_outputMem > 0)
            m_deallocFn->call(m_outputMem);
    }


    DtasmModelDescription get_model_description() {
        return m_modelDesc;
    }


    DtasmStatus initialize(
        const DtasmVarValues &initial_vals,
        double tmin,
        bool tmax_set,
        double tmax,
        bool tol_set,
        double tol,
        DtasmLogLevel log_level,
        bool check) {

        auto modelId = m_builder.CreateString(m_modelDesc.model.id);

        auto varVals = varValuesToFb(initial_vals);
        auto logLevel = logLevelDtasmToFb(log_level);

        auto initRequest = DTAPI::CreateInitReq(
            m_builder,
            modelId,
            tmin,
            tmax_set,
            tmax,
            tol_set,
            tol,
            logLevel,
            check,
            varVals);

        m_builder.Finish(initRequest);

        auto reqBuffer = m_builder.GetBufferPointer();
        auto reqSize = (int32_t)m_builder.GetSize();

        uint8_t* resBuffer = callInputOutput(m_initFn.get(), reqBuffer, reqSize);

        const auto initStatus = FB::GetRoot<DTAPI::StatusRes>(resBuffer);
        auto status = initStatus->status();
        auto retStatus = statusFbToDtasm(status);

        m_builder.Reset();

        return retStatus;
    }


    DtasmStatus get_values(const std::vector<int32_t> &var_ids, 
        DtasmGetValuesResponse &getValues) {

        auto id_vec = m_builder.CreateVector(var_ids);
        auto getValuesReq = DTAPI::CreateGetValuesReq(m_builder, id_vec);
        m_builder.Finish(getValuesReq);

        auto reqBuffer = m_builder.GetBufferPointer();
        auto reqSize = (int32_t)m_builder.GetSize();

        uint8_t* resBuffer = callInputOutput(m_getValuesFn.get(), reqBuffer, reqSize);

        const auto res = FB::GetRoot<DTAPI::GetValuesRes>(resBuffer);
        auto retStatus = res->status();
        getValues.status = statusFbToDtasm(retStatus);
        getValues.current_time = res->current_time();
        auto var_values = res->values();

        if (var_values->real_vals() != nullptr) {
            for (auto it = var_values->real_vals()->begin();
                 it != var_values->real_vals()->end();
                 ++it) {

                getValues.values.real_values[it->id()] = it->val();
            }
        }

        if (var_values->int_vals() != nullptr) {
            for (auto it = var_values->int_vals()->begin(); 
                it != var_values->int_vals()->end(); 
                ++it) {

                getValues.values.int_values[it->id()] = it->val();
            }
        }

        if (var_values->bool_vals() != nullptr) {
            for (auto it = var_values->bool_vals()->begin(); 
                it != var_values->bool_vals()->end(); 
                ++it) {

                getValues.values.bool_values[it->id()] = it->val();
            }
        }

        if (var_values->string_vals() != nullptr) {
            for (auto it = var_values->string_vals()->begin(); 
                it != var_values->string_vals()->end(); 
                ++it) {

                getValues.values.string_values[it->id()] = it->val()->str();
            }
        }

        m_builder.Reset();
        return getValues.status;
    }


    DtasmStatus set_values(const DtasmVarValues &set_vals) {

        auto setValues = varValuesToFb(set_vals);
        auto setValuesReq = DTAPI::CreateSetValuesReq(m_builder, setValues);
        m_builder.Finish(setValuesReq);

        auto reqBuffer = m_builder.GetBufferPointer();
        auto reqSize = (int32_t)m_builder.GetSize();

        uint8_t* resBuffer = callInputOutput(m_setValuesFn.get(), reqBuffer, reqSize);

        const auto res = FB::GetRoot<DTAPI::StatusRes>(resBuffer);
        auto retStatus = res->status();
        auto status = statusFbToDtasm(retStatus);

        m_builder.Reset();
        return status;
    }


    DtasmDoStepResponse do_step(double t, double dt) {
        auto doStepReq = DTAPI::CreateDoStepReq(m_builder, t, dt);
        m_builder.Finish(doStepReq);

        auto reqBuffer = m_builder.GetBufferPointer();
        auto reqSize = (int32_t)m_builder.GetSize();

        uint8_t* resBuffer = callInputOutput(m_doStepFn.get(), reqBuffer, reqSize);

        const auto res = FB::GetRoot<DTAPI::DoStepRes>(resBuffer);
        auto retStatus = res->status();
        
        DtasmDoStepResponse do_step_res;
        do_step_res.status = statusFbToDtasm(retStatus);
        do_step_res.updated_time = res->updated_time();

        m_builder.Reset();
        return do_step_res;
    }


    void save_state(std::vector<uint8_t> &state_buffer) {
        
        // Need to free memory since otherwise it cannot be recovered 
        // after loading the generated state snapshot
        m_deallocFn->call(m_inputMem);
        m_deallocFn->call(m_outputMem);

        uint32_t memSize;
        auto memPtr = m_m3Runtime.get_memory(memSize, 0);
        state_buffer.resize(memSize);
        for (uint32_t i=0; i<memSize; ++i) {
            state_buffer[i] = *(memPtr++);
        }

        m_outputMem = m_allocFn->call<int32_t>(m_buffersize);
        m_inputMem = m_allocFn->call<int32_t>(m_buffersize);
    }


    void load_state(const std::vector<uint8_t> &state_buffer) {
       
        uint32_t memSize;
        auto memPtr = m_m3Runtime.get_memory(memSize, 0);

        if (memSize < state_buffer.size()) {
            if (state_buffer.size() % WASM_PAGE_SIZE != 0) {
                std::stringstream errMsg;
                errMsg << "Invalid state buffer size: " << state_buffer.size();
                throw std::runtime_error(errMsg.str().c_str());
            }

            auto n_pages = state_buffer.size() / WASM_PAGE_SIZE;
            m_m3Runtime.resize_memory(n_pages);
        }

        memPtr = m_m3Runtime.get_memory(memSize, 0);
        for (uint32_t i=0; i<memSize; ++i) {
            *(memPtr++) = state_buffer[i];
        }

        m_outputMem = m_allocFn->call<int32_t>(m_buffersize);
        m_inputMem = m_allocFn->call<int32_t>(m_buffersize);
    }
};


dtasm3::Runtime::Runtime(Impl &impl): 
    _rt(std::make_unique<dtasm3::Runtime::Impl>(std::move(impl)))
{}

// dtasm3::Runtime::Runtime(const Runtime &rt): 
//     _rt(std::make_unique<dtasm3::Runtime::Impl>(std::move(*rt._rt)))
// {}

dtasm3::Runtime::~Runtime() = default;

dtasm3::DtasmModelDescription dtasm3::Runtime::get_model_description() {
    return _rt->get_model_description();
}

dtasm3::DtasmStatus dtasm3::Runtime::initialize(
            const dtasm3::DtasmVarValues &initial_vals,
            double tmin,
            bool tmax_set,
            double tmax,
            bool tol_set,
            double tol,
            dtasm3::DtasmLogLevel log_level,
            bool check) {

    return _rt->initialize(
        initial_vals,
        tmin,
        tmax_set,
        tmax,
        tol_set,
        tol,
        log_level,
        check);
}

dtasm3::DtasmStatus dtasm3::Runtime::get_values(const std::vector<int32_t> &var_ids, 
    dtasm3::DtasmGetValuesResponse &res) {
    return _rt->get_values(var_ids, res);
}

dtasm3::DtasmStatus dtasm3::Runtime::set_values(const DtasmVarValues &set_vals) {
    return _rt->set_values(set_vals);
}

dtasm3::DtasmDoStepResponse dtasm3::Runtime::do_step(double t, double dt) {
    return _rt->do_step(t, dt);
}

void dtasm3::Runtime::save_state(std::vector<uint8_t> &state_buffer) {
    return _rt->save_state(state_buffer);
}

void dtasm3::Runtime::load_state(const std::vector<uint8_t> &state_buffer) {
    return _rt->load_state(state_buffer);
}


class dtasm3::Module::Impl {
private: 
    std::shared_ptr<wasm3::module> m3_module;

public: 
    Impl(std::shared_ptr<wasm3::module> mod) : m3_module(mod){}
    std::shared_ptr<wasm3::module> get_m3_module() {
       return m3_module;
    }
};

dtasm3::Module::Module(Impl &impl): 
    _mod{std::make_unique<dtasm3::Module::Impl>(impl)}
{}

dtasm3::Module::Module(const Module &mod):
    _mod{std::make_unique<dtasm3::Module::Impl>(*mod._mod)}
{}

dtasm3::Module::~Module() = default;



class dtasm3::Environment::Impl {
private:
    size_t stack_size;
    wasm3::environment m3_env;

public:
    Impl(size_t stack_size_bytes) {
        stack_size = stack_size_bytes;
    }

    Module load_module(const uint8_t* data, const size_t len) {
        auto m3_module = std::make_shared<wasm3::module>(m3_env.parse_module(data, len));
        auto mod_impl = dtasm3::Module::Impl(m3_module);
        return dtasm3::Module(mod_impl);
    }

    Module load_module(std::shared_ptr<std::vector<uint8_t>> data) {
        auto m3_module = std::make_shared<wasm3::module>(m3_env.parse_module(data));
        auto mod_impl = dtasm3::Module::Impl(m3_module);
        return dtasm3::Module(mod_impl);
    }

    Runtime create_runtime(Module &module, int buffersize = 8192) {
        auto rt_impl = dtasm3::Runtime::Impl(
            module._mod->get_m3_module(), 
            m3_env, 
            stack_size, 
            buffersize);
        return dtasm3::Runtime(rt_impl);
    }
};

dtasm3::Environment::Environment(size_t stack_size_bytes = 64 * 1024): 
    _env{std::make_unique<dtasm3::Environment::Impl>(stack_size_bytes)} 
{}

dtasm3::Module dtasm3::Environment::load_module(const uint8_t* data, const size_t len){
    return _env->load_module(data, len);
}

dtasm3::Module dtasm3::Environment::load_module(std::shared_ptr<std::vector<uint8_t>> data){
    return _env->load_module(data);
}

dtasm3::Runtime dtasm3::Environment::create_runtime(Module &module, int buffersize){
    return _env->create_runtime(module, buffersize);
}

dtasm3::Environment::~Environment() = default;
