mod dispatcher;
mod error;
mod handlers;
mod safe_int;
mod state;
mod wire;

pub mod mock_export;

use dispatcher::{err_json, BridgeCommand, BridgeDispatcher};
use error::BridgeError;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;

fn native_log(msg: &str) {
    #[cfg(target_os = "android")]
    {
        extern "C" {
            fn __android_log_write(prio: i32, tag: *const c_char, text: *const c_char) -> i32;
        }
        const ANDROID_LOG_ERROR: i32 = 6;
        let tag = CString::new("RustBridge").expect("static tag has no interior null byte");
        if let Ok(text) = CString::new(msg.replace('\0', "")) {
            unsafe { __android_log_write(ANDROID_LOG_ERROR, tag.as_ptr(), text.as_ptr()) };
        }
    }
    #[cfg(not(target_os = "android"))]
    eprintln!("[RustBridge] {msg}");
}

/// Routes `tracing` output through `native_log` so app_core's structured logs
/// reach logcat/stderr instead of going nowhere.
struct NativeLogWriter;

impl std::io::Write for NativeLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        native_log(String::from_utf8_lossy(buf).trim_end());
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

static TRACING_INIT: Lazy<()> = Lazy::new(|| {
    tracing_subscriber::fmt()
        .with_writer(|| NativeLogWriter)
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .with_ansi(false)
        .init();
    native_log("RustBridge: tracing subscriber installed");
});

/// Panic hook to log panics before unwinding discards context
static PANIC_HOOK: Lazy<()> = Lazy::new(|| {
    panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        native_log(&format!(
            "PANIC at {}: {}",
            location,
            panic_detail(info.payload())
        ));
    }));
});

/// Extracts a human-readable message out of a panic payload.
fn panic_detail(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic payload".to_string())
}

/// Invokes a Rust command from the host C FFI environment.
///
/// `envelope` is a single JSON document (`{"cmd":...,"args":...}`) built by the
/// host with a real JSON serializer. Taking one pre-encoded string instead of
/// assembling `{"cmd":"<cmd>","args":<args>}` here is what makes the boundary
/// injection-proof: nothing is ever concatenated into JSON on this side.
///
/// The returned pointer is owned by the caller and MUST be released with
/// [`free_rust_string`]. A null return means the response could not be encoded.
///
/// # Safety
/// Dereferences a raw pointer passed from the C environment. The pointer must
/// either be null or point to a NUL-terminated string valid for this call.
#[no_mangle]
pub unsafe extern "C" fn call_rust(envelope: *const c_char) -> *mut c_char {
    Lazy::force(&PANIC_HOOK);
    Lazy::force(&TRACING_INIT);

    // Contains any panic before it can unwind across the `extern "C"` boundary,
    // which would abort the process. Requires `panic = "unwind"` (pinned in the
    // workspace release profile).
    let result = panic::catch_unwind(|| {
        if envelope.is_null() {
            return err_json(BridgeError::InvalidArgument {
                reason: "envelope pointer is null".to_string(),
            });
        }

        let json = match CStr::from_ptr(envelope).to_str() {
            Ok(json) => json,
            Err(_) => {
                return err_json(BridgeError::InvalidArgument {
                    reason: "envelope is not valid UTF-8".to_string(),
                })
            }
        };

        match serde_json::from_str::<BridgeCommand>(json) {
            Ok(command) => BridgeDispatcher::run(command),
            Err(err) => err_json(BridgeError::InvalidArgument {
                reason: format!("failed to parse bridge command: {err}"),
            }),
        }
    });

    let final_json = result.unwrap_or_else(|payload| {
        err_json(BridgeError::Internal {
            reason: format!("rust panic: {}", panic_detail(payload.as_ref())),
        })
    });

    CString::new(final_json)
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Frees a C string allocated by [`call_rust`].
///
/// # Safety
/// The pointer must be null or have come from [`call_rust`] and not been freed
/// before. Passing any other pointer is undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn free_rust_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Android JNI entry point.
///
/// Declared as an *instance* method on `RustBridgeModule` (not on its companion
/// object) so the mangled symbol is unambiguously
/// `Java_com_myapp_rustbridge_RustBridgeModule_callRustNative`.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_myapp_rustbridge_RustBridgeModule_callRustNative<'a>(
    mut unowned_env: jni::EnvUnowned<'a>,
    _this: jni::objects::JObject<'a>,
    envelope: jni::objects::JString<'a>,
) -> jni::sys::jstring {
    unowned_env
        .with_env(|env| {
            // `try_to_string` decodes JNI's modified UTF-8 (CESU-8), so
            // astral-plane characters survive the crossing intact.
            let envelope = match envelope.try_to_string(env) {
                Ok(s) => s,
                Err(e) => {
                    return Ok(jstring_or_die(
                        env,
                        &err_json(BridgeError::Internal {
                            reason: format!("failed to read envelope from JNI: {e}"),
                        }),
                    ))
                }
            };

            let envelope_c = match CString::new(envelope) {
                Ok(c) => c,
                Err(_) => {
                    return Ok(jstring_or_die(
                        env,
                        &err_json(BridgeError::InvalidArgument {
                            reason: "envelope contains an interior null byte".to_string(),
                        }),
                    ))
                }
            };

            let result_ptr = unsafe { call_rust(envelope_c.as_ptr()) };
            let result = if result_ptr.is_null() {
                err_json(BridgeError::Internal {
                    reason: "rust returned a null pointer".to_string(),
                })
            } else {
                let owned = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
                unsafe { free_rust_string(result_ptr) };
                owned
            };

            Ok::<_, jni::errors::Error>(jstring_or_die(env, &result))
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// Allocates a Java string, falling back to a static payload if even that fails.
#[cfg(target_os = "android")]
fn jstring_or_die(env: &mut jni::Env<'_>, value: &str) -> jni::sys::jstring {
    env.new_string(value)
        .map(|j| j.into_raw())
        .unwrap_or_else(|_| {
            env.new_string(
                r#"{"status":"error","kind":"internal","reason":"failed to allocate java string"}"#,
            )
            .map(|j| j.into_raw())
            .expect("static fallback string is allocatable")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(envelope: &str) -> String {
        let c = CString::new(envelope).unwrap();
        let out = unsafe { call_rust(c.as_ptr()) };
        assert!(!out.is_null());
        let response = unsafe { CStr::from_ptr(out).to_str().unwrap().to_owned() };
        unsafe { free_rust_string(out) };
        response
    }

    /// The old boundary built `{"cmd":"<cmd>","args":<args>}` with `format!`,
    /// so a command name full of quotes could forge the whole envelope.
    #[test]
    fn quotes_in_cmd_cannot_forge_args() {
        let forged = r#"user","args":{"type":"save_user","id":"1","name":"x","role":"admin"}"#;
        let envelope = serde_json::to_string(&serde_json::json!({ "cmd": forged })).unwrap();
        let response = call(&envelope);
        assert!(response.contains(r#""status":"error""#), "{response}");
        assert!(
            response.contains(r#""kind":"invalid_argument""#),
            "{response}"
        );
    }

    #[test]
    fn null_envelope_returns_structured_error() {
        let out = unsafe { call_rust(std::ptr::null()) };
        let response = unsafe { CStr::from_ptr(out).to_str().unwrap().to_owned() };
        unsafe { free_rust_string(out) };
        assert!(
            response.contains(r#""kind":"invalid_argument""#),
            "{response}"
        );
    }

    #[test]
    fn valid_envelope_round_trips() {
        assert_eq!(
            call(r#"{"cmd":"system","args":{"type":"ping"}}"#),
            r#"{"status":"success","data":"pong"}"#
        );
    }
}
