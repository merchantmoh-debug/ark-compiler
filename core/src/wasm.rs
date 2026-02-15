/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 *
 * 1. OPEN SOURCE: You may use this file under the terms of the GNU Affero
 * General Public License v3.0. If you link to this code, your ENTIRE
 * application must be open-sourced under AGPLv3.
 *
 * 2. COMMERCIAL: For proprietary use, you must obtain a Commercial License
 * from Sovereign Systems.
 *
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 * NO IMPLIED LICENSE to rights of Mohamad Al-Zawahreh or Sovereign Systems.
 */

use crate::compiler::Compiler;
use crate::loader::load_ark_program;
use crate::runtime::Value;
use crate::vm::VM;
use std::mem;
use std::slice;
use std::str;

// Global Persistent VM Instance for Browser/WASM Environment
static mut VM_INSTANCE: Option<VM<'static>> = None;

/// Allocates memory for a string of `size` bytes.
/// Returns a pointer to the allocated memory.
#[no_mangle]
pub extern "C" fn ark_alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

/// Deallocates memory.
#[no_mangle]
pub unsafe extern "C" fn ark_dealloc(ptr: *mut u8, size: usize) {
    let _ = Vec::from_raw_parts(ptr, 0, size);
}

/// Initializes the persistent VM with the given Ark program (JSON MAST).
/// Returns "OK" on success or error message.
#[no_mangle]
pub unsafe extern "C" fn ark_init(input_ptr: *mut u8, input_len: usize) -> *mut u8 {
    let input_slice = slice::from_raw_parts(input_ptr, input_len);
    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => return make_response("Error: Invalid UTF-8 input"),
    };

    let response = match load_ark_program(input_str) {
        Ok(mast) => {
            // Leak the MAST to ensure hash reference lives forever (static VM)
            // This is acceptable in this context as ark_init is called once.
            let leaked_mast = Box::leak(Box::new(mast));

            let compiler = Compiler::new();
            let chunk = compiler.compile(&leaked_mast.content);

            // Initialize VM with 'static lifetime (unsafe but necessary for singleton)
            // In WASM, we are single-threaded, so this is generally safe if we don't leak refs.
            // The VM takes ownership of the chunk.
            match VM::new(chunk, &leaked_mast.hash, 0) {
                Ok(mut vm) => {
                    // Run top-level code (e.g., definitions)
                    match vm.run() {
                        Ok(_) => {
                            // Store the VM instance
                            VM_INSTANCE = Some(mem::transmute(vm));
                            "OK".to_string()
                        }
                        Err(e) => format!("Runtime Error during Init: {:?}", e),
                    }
                }
                Err(e) => format!("VM Init Error: {:?}", e),
            }
        }
        Err(e) => format!("Load Error: {:?}", e),
    };

    make_response(&response)
}

/// Calls a specific function on the initialized VM.
/// Args: func_name (string), args_json (string: [arg1, arg2, ...])
/// Returns: Result as JSON string.
#[no_mangle]
pub unsafe extern "C" fn ark_call(
    name_ptr: *mut u8,
    name_len: usize,
    args_ptr: *mut u8,
    args_len: usize,
) -> *mut u8 {
    let vm = match VM_INSTANCE.as_mut() {
        Some(v) => v,
        None => return make_response("Error: VM not initialized"),
    };

    let name_slice = slice::from_raw_parts(name_ptr, name_len);
    let name_str = match str::from_utf8(name_slice) {
        Ok(s) => s,
        Err(_) => return make_response("Error: Invalid UTF-8 name"),
    };

    let args_slice = slice::from_raw_parts(args_ptr, args_len);
    let args_str = match str::from_utf8(args_slice) {
        Ok(s) => s,
        Err(_) => return make_response("Error: Invalid UTF-8 args"),
    };

    // Parse Args JSON
    let args_json: serde_json::Value = match serde_json::from_str(args_str) {
        Ok(v) => v,
        Err(e) => return make_response(&format!("Error: Invalid JSON args: {}", e)),
    };

    let args_vec = match args_json {
        serde_json::Value::Array(list) => {
            let mut vals = Vec::new();
            for item in list {
                vals.push(json_to_value(&item));
            }
            vals
        }
        _ => return make_response("Error: Args must be a JSON array"),
    };

    // Call Function
    match vm.call_public_function(name_str, args_vec) {
        Ok(val) => {
            let json_val = value_to_json(&val);
            make_response(&json_val.to_string())
        }
        Err(e) => make_response(&format!("Error: Runtime: {}", e)),
    }
}

/// Evaluates an Ark program (JSON MAST).
/// ONE-SHOT Execution (Does not persist state).
#[no_mangle]
pub unsafe extern "C" fn ark_eval(input_ptr: *mut u8, input_len: usize) -> *mut u8 {
    // 1. Reconstruct the input string
    let input_slice = slice::from_raw_parts(input_ptr, input_len);
    let input_str = match str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => return make_response("Error: Invalid UTF-8 input"),
    };

    // 2. Load and Execute
    let response = match load_ark_program(input_str) {
        Ok(mast) => {
            let compiler = Compiler::new();
            let chunk = compiler.compile(&mast.content);
            match VM::new(chunk, &mast.hash, 0) {
                Ok(mut vm) => match vm.run() {
                    Ok(val) => format!("Result: {:?}", val),
                    Err(e) => format!("Runtime Error: {:?}", e),
                },
                Err(e) => format!("VM Init Error: {:?}", e),
            }
        }
        Err(e) => format!("Load Error: {:?}", e),
    };

    // 3. Serialize Response
    make_response(&response)
}

unsafe fn make_response(s: &str) -> *mut u8 {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;

    // Layout: [len (4 bytes)] [content...]
    let mut buf = Vec::with_capacity(4 + bytes.len());

    // Write Length (Little Endian)
    buf.extend_from_slice(&len.to_le_bytes());
    // Write Content
    buf.extend_from_slice(bytes);

    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

fn json_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Unit,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                // Ark only supports Integer for now, but we can try to cast?
                Value::Integer(f as i64)
            } else {
                Value::Integer(0)
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(l) => {
            let mut list = Vec::new();
            for item in l {
                list.push(json_to_value(item));
            }
            Value::List(list)
        }
        serde_json::Value::Object(m) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in m {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Struct(map)
        }
    }
}

fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Unit => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => serde_json::Value::Number((*i).into()),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::List(l) => {
            let mut arr = Vec::new();
            for item in l {
                arr.push(value_to_json(item));
            }
            serde_json::Value::Array(arr)
        }
        Value::Struct(m) => {
            let mut map = serde_json::Map::new();
            for (k, v) in m {
                map.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        // Others map to null or string representation
        Value::LinearObject { id, .. } => serde_json::Value::String(format!("LinearObject:{}", id)),
        Value::Function(_) => serde_json::Value::String("Function".to_string()),
        Value::NativeFunction(_) => serde_json::Value::String("NativeFunction".to_string()),
        Value::Buffer(_) => serde_json::Value::String("Buffer".to_string()),
        Value::Return(val) => value_to_json(val),
    }
}
