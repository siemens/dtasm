#pragma once

#include "wasm3.h"
#include "wasm3_cpp.h"


namespace wasm3 {
    class runtime_mem : public wasm3::runtime {
    public: 
        IM3Runtime get_m3runtime() {
            return m_runtime.get();
        };

        runtime_mem(const std::shared_ptr<M3Environment> &env, size_t stack_size_bytes)
            : wasm3::runtime(env, stack_size_bytes) {}
    };

    class environment_mem : public wasm3::environment {
    public: 
        runtime_mem new_runtime_mem(size_t stack_size_bytes) {
            return runtime_mem(m_env, stack_size_bytes);
        };
    };
}