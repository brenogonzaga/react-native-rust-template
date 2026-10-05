mod dispatcher;
mod error;
mod handlers;
mod safe_int;
mod state;
mod wire;

use dispatcher::{err_json, BridgeCommand, BridgeDispatcher};
use error::BridgeError;
use std::backtrace::Backtrace;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<&tracing::Level> for LogLevel {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::ERROR => Self::Error,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::INFO => Self::Info,
            _ => Self::Debug,
        }
    }
}

fn native_log(level: LogLevel, msg: &str) {
    // One entry per line: logcat and iOS unified logging truncate long
    // entries, which would cut a panic backtrace short.
    for line in msg.lines().filter(|l| !l.is_empty()) {
        platform_log(level, line);
    }
}

#[cfg(target_os = "android")]
fn platform_log(level: LogLevel, line: &str) {
    extern "C" {
        fn __android_log_write(prio: i32, tag: *const c_char, text: *const c_char) -> i32;
    }
    // android/log.h priorities, so `adb logcat RustBridge:W *:S` filters.
    let prio = match level {
        LogLevel::Debug => 3,
        LogLevel::Info => 4,
        LogLevel::Warn => 5,
        LogLevel::Error => 6,
    };
    let tag = CString::new("RustBridge").expect("static tag has no interior null byte");
    if let Ok(text) = CString::new(line.replace('\0', "")) {
        unsafe { __android_log_write(prio, tag.as_ptr(), text.as_ptr()) };
    }
}

/// iOS routes syslog into unified logging, so Rust logs reach `log stream` and
/// Console.app, not only the Xcode console (stderr is dropped when the app
/// isn't launched from Xcode, e.g. by `npm run ios`).
#[cfg(target_os = "ios")]
fn platform_log(level: LogLevel, line: &str) {
    extern "C" {
        fn syslog(priority: i32, format: *const c_char, ...);
    }
    // sys/syslog.h priorities. iOS keeps info and debug apart but files
    // everything above as "Default", so warnings and errors look alike there.
    let prio = match level {
        LogLevel::Debug => 7,
        LogLevel::Info => 6,
        LogLevel::Warn => 4,
        LogLevel::Error => 3,
    };
    if let Ok(text) = CString::new(format!("[RustBridge] {}", line.replace('\0', ""))) {
        unsafe { syslog(prio, c"%s".as_ptr(), text.as_ptr()) };
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn platform_log(_level: LogLevel, line: &str) {
    eprintln!("[RustBridge] {line}");
}

/// Routes `tracing` output through `native_log` so app_core's structured logs
/// reach logcat/stderr instead of going nowhere — at the event's own level.
struct NativeLog;

struct NativeLogWriter(LogLevel);

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for NativeLog {
    type Writer = NativeLogWriter;

    fn make_writer(&'a self) -> NativeLogWriter {
        NativeLogWriter(LogLevel::Info)
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> NativeLogWriter {
        NativeLogWriter(meta.level().into())
    }
}

impl std::io::Write for NativeLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        native_log(self.0, String::from_utf8_lossy(buf).trim_end());
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn install_tracing() {
    let installed = tracing_subscriber::fmt()
        .with_writer(NativeLog)
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .without_time()
        .try_init();
    match installed {
        Ok(()) => native_log(LogLevel::Info, "RustBridge: tracing subscriber installed"),
        Err(err) => native_log(
            LogLevel::Error,
            &format!("tracing subscriber not installed: {err}"),
        ),
    }
}

static TRACING_INIT: LazyLock<()> = LazyLock::new(install_tracing);

/// Logs panics to logcat/stderr before unwinding discards the context, then
/// hands over to the hook installed before it (Rust's default, or another
/// library's) instead of silently replacing it.
fn install_panic_hook() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        native_log(LogLevel::Error, &panic_report(info));
        previous(info);
    }));
}

/// Location, message and the backtrace of a panic: the unwind that
/// `catch_unwind` stops would otherwise discard where it happened.
fn panic_report(info: &panic::PanicHookInfo) -> String {
    let location = info
        .location()
        .map(|l| format!("{}:{}", l.file(), l.line()))
        .unwrap_or_else(|| "unknown location".to_string());
    format!(
        "PANIC at {location}: {}\n{}",
        panic_detail(info.payload()),
        Backtrace::force_capture()
    )
}

static PANIC_HOOK: LazyLock<()> = LazyLock::new(install_panic_hook);

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
    LazyLock::force(&PANIC_HOOK);
    LazyLock::force(&TRACING_INIT);

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
            Err(err) => err_json(BridgeError::InvalidCommand {
                reason: err.to_string(),
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
/// The symbol is fixed by the Kotlin side: `Java_<package>_<Class>_<method>`
/// for `expo.modules.rustbridge.RustBridgeModule.callRustNative`. That package
/// belongs to the local module, not to the app, so changing the app's
/// `android.package` never requires touching this. If you do move the Kotlin
/// class, `tests/jni_symbol.rs` fails until this name matches again.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_expo_modules_rustbridge_RustBridgeModule_callRustNative<'a>(
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
            response.contains(r#""kind":"invalid_command""#),
            "{response}"
        );
    }

    #[test]
    fn unknown_command_reports_invalid_command() {
        let response = call(r#"{"cmd":"system","args":{"type":"brand_new"}}"#);
        assert!(
            response.contains(r#""kind":"invalid_command""#),
            "{response}"
        );
        assert!(response.contains("unknown variant"), "{response}");
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

    /// The panic hook is process-global: tests that replace it take turns.
    static HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn panic_report_has_location_message_and_backtrace() {
        use std::sync::Mutex;
        static REPORT: Mutex<String> = Mutex::new(String::new());
        let _turn = HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        panic::set_hook(Box::new(|info| {
            if panic_detail(info.payload()) == "report-marker" {
                *REPORT.lock().unwrap() = panic_report(info);
            }
        }));
        let _ = panic::catch_unwind(|| panic!("report-marker"));
        let report = REPORT.lock().unwrap().clone();
        assert!(
            report.starts_with("PANIC at rust-core/src/lib.rs:"),
            "{report}"
        );
        assert!(report.contains("report-marker"), "{report}");
        assert!(
            report.contains("panic_report_has_location_message_and_backtrace"),
            "no backtrace frames:\n{report}"
        );
    }

    #[test]
    fn panic_hook_chains_to_the_previous_hook() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static REACHED: AtomicBool = AtomicBool::new(false);
        let _turn = HOOK_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        panic::set_hook(Box::new(|info| {
            if panic_detail(info.payload()) == "chain-marker" {
                REACHED.store(true, Ordering::SeqCst);
            }
        }));
        install_panic_hook();
        let _ = panic::catch_unwind(|| panic!("chain-marker"));
        assert!(REACHED.load(Ordering::SeqCst));
    }

    /// iOS runs `callRust` on a concurrent queue, Android on Dispatchers.IO.
    #[test]
    fn concurrent_calls_are_safe() {
        let workers: Vec<_> = (0..8)
            .map(|t| {
                std::thread::spawn(move || {
                    for i in 0..50 {
                        let save = serde_json::json!({
                            "cmd": "user",
                            "args": { "type": "save_user", "id": format!("c{t}-{i}"), "name": "n", "role": "r" }
                        });
                        let response = call(&save.to_string());
                        assert!(response.contains(r#""status":"success""#), "{response}");
                        assert_eq!(
                            call(r#"{"cmd":"system","args":{"type":"ping"}}"#),
                            r#"{"status":"success","data":"pong"}"#
                        );
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    }

    #[test]
    fn installing_tracing_twice_does_not_panic() {
        install_tracing();
        install_tracing();
    }

    #[test]
    fn valid_envelope_round_trips() {
        assert_eq!(
            call(r#"{"cmd":"system","args":{"type":"ping"}}"#),
            r#"{"status":"success","data":"pong"}"#
        );
    }
}
