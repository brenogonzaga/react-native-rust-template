mod dispatcher;
mod error;
mod handlers;
pub mod mock_export;
mod state;
mod utils;

use dispatcher::{BridgeCommand, BridgeDispatcher, BridgeResponse};
use error::BridgeError;
use once_cell::sync::Lazy;
#[allow(unused_imports)]
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;
use tokio::runtime::{Builder, Runtime};
use utils::{CommandParser, NativeLogger};

/// Panic hook to log panics before unwinding discards context
static PANIC_HOOK: Lazy<()> = Lazy::new(|| {
    panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_string());
        NativeLogger::log(&format!("PANIC at {}: {}", location, message));
    }));
});

/// Initialize Tokio multi-thread runtime
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

/// Invokes a Rust command from host C FFI environment
///
/// # Safety
/// Dereferences raw pointers passed from C environment.
#[no_mangle]
pub unsafe extern "C" fn call_rust(cmd: *const c_char, args: *const c_char) -> *const c_char {
    Lazy::force(&PANIC_HOOK);
    let result = panic::catch_unwind(|| match CommandParser::parse_command(cmd, args) {
        Ok(json_payload) => match serde_json::from_str::<BridgeCommand>(&json_payload) {
            Ok(command) => RUNTIME.block_on(BridgeDispatcher::run(command)),
            Err(err) => {
                let err_res = BridgeResponse::<()>::Error(BridgeError::InvalidArgument(format!(
                    "Failed to parse bridge command: {}",
                    err
                )));
                serde_json::to_string(&err_res).unwrap_or_else(|_| {
                    r#"{"status":"error","kind":"internal","message":"Failed to serialize error"}"#
                        .to_string()
                })
            }
        },
        Err(err_msg) => {
            let err_res = BridgeResponse::<()>::Error(BridgeError::Internal(err_msg));
            serde_json::to_string(&err_res).unwrap_or_else(|_| {
                r#"{"status":"error","kind":"internal","message":"Command parsing error"}"#
                    .to_string()
            })
        }
    });

    let final_json = result.unwrap_or_else(|_| {
        r#"{"status":"error","kind":"internal","message":"Rust Panicked"}"#.to_string()
    });

    CString::new(final_json)
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Frees C String allocated by `call_rust`
///
/// # Safety
/// Reconstructs CString from raw pointer and drops it.
#[no_mangle]
pub unsafe extern "C" fn free_rust_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

#[cfg(target_os = "android")]
fn get_jni_str(
    env: &mut jni::Env<'_>,
    jstr_val: &jni::objects::JString,
    name: &str,
) -> Result<String, jni::sys::jstring> {
    match jstr_val.try_to_string(env) {
        Ok(s) => Ok(s),
        Err(e) => {
            let err_msg = format!(
                r#"{{"status":"error","kind":"internal","message":"Failed to read {} string from JNI: {}"}}"#,
                name, e
            );
            let raw_err = env.new_string(err_msg)
                .map(|j| j.into_raw())
                .unwrap_or_else(|_| {
                    env.new_string("{\"status\":\"error\",\"kind\":\"internal\",\"message\":\"JNI error nesting\"}")
                        .map(|j| j.into_raw())
                        .expect("Static fallback string should compile")
                });
            Err(raw_err)
        }
    }
}

#[cfg(target_os = "android")]
fn to_cstring(
    env: &mut jni::Env<'_>,
    val: String,
    name: &str,
) -> Result<CString, jni::sys::jstring> {
    match CString::new(val) {
        Ok(c) => Ok(c),
        Err(_) => {
            let err_msg = format!(
                r#"{{"status":"error","kind":"internal","message":"{} contains null byte"}}"#,
                name
            );
            let raw_err = env.new_string(err_msg)
                .map(|j| j.into_raw())
                .unwrap_or_else(|_| {
                    env.new_string("{\"status\":\"error\",\"kind\":\"internal\",\"message\":\"JNI error null byte\"}")
                        .map(|j| j.into_raw())
                        .expect("Static fallback string should compile")
                });
            Err(raw_err)
        }
    }
}

/// Android JNI Entry Point
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_myapp_rustbridge_RustBridgeModule_callRustNative<'a>(
    mut unowned_env: jni::EnvUnowned<'a>,
    _class: jni::objects::JClass<'a>,
    command: jni::objects::JString<'a>,
    args_json: jni::objects::JString<'a>,
) -> jni::sys::jstring {
    unowned_env
        .with_env(|env| {
            let cmd = match get_jni_str(env, &command, "command") {
                Ok(s) => s,
                Err(raw_jstr) => return Ok::<_, jni::errors::Error>(raw_jstr),
            };

            let args = match get_jni_str(env, &args_json, "args") {
                Ok(s) => s,
                Err(raw_jstr) => return Ok::<_, jni::errors::Error>(raw_jstr),
            };

            let cmd_c = match to_cstring(env, cmd, "Command") {
                Ok(c) => c,
                Err(raw_jstr) => return Ok::<_, jni::errors::Error>(raw_jstr),
            };

            let args_c = match to_cstring(env, args, "Arguments") {
                Ok(c) => c,
                Err(raw_jstr) => return Ok::<_, jni::errors::Error>(raw_jstr),
            };

            let result_ptr = unsafe { call_rust(cmd_c.as_ptr(), args_c.as_ptr()) };
            let result_str = if result_ptr.is_null() {
                r#"{"status":"error","kind":"internal","message":"Rust returned null pointer"}"#
                    .to_string()
            } else {
                let str_val = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
                unsafe { free_rust_string(result_ptr as *mut c_char) };
                str_val
            };

            let jstr = env.new_string(result_str)?;
            Ok::<_, jni::errors::Error>(jstr.into_raw())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
