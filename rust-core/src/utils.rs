use std::ffi::CStr;
use std::os::raw::c_char;

pub struct CommandParser;

impl CommandParser {
    pub unsafe fn parse_command(
        cmd_ptr: *const c_char,
        args_ptr: *const c_char,
    ) -> Result<String, String> {
        if cmd_ptr.is_null() {
            return Err("Command pointer is null".to_string());
        }

        let cmd_str = CStr::from_ptr(cmd_ptr)
            .to_str()
            .map_err(|_| "Command is not valid UTF-8".to_string())?;

        let args_str = if !args_ptr.is_null() {
            CStr::from_ptr(args_ptr).to_str().unwrap_or("")
        } else {
            ""
        };

        if args_str.is_empty() {
            Ok(format!(r#"{{"cmd":"{}"}}"#, cmd_str))
        } else {
            Ok(format!(r#"{{"cmd":"{}","args":{}}}"#, cmd_str, args_str))
        }
    }
}

#[allow(dead_code)]
pub struct NativeLogger;

#[allow(dead_code)]
impl NativeLogger {
    pub fn log(msg: &str) {
        println!("[RustBridge] {}", msg);
    }
}
