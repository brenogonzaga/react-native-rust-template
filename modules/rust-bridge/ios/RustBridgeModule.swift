import ExpoModulesCore

private let bridgeQueue = DispatchQueue(
  label: "com.myapp.rust-bridge", qos: .userInitiated, attributes: .concurrent)

public class RustBridgeModule: Module {
  public func definition() -> ModuleDefinition {
    Name("RustBridge")

    AsyncFunction("callRust") { (command: String, payload: String) -> String in
      guard let resultPtr = call_rust_native(command, payload) else {
        return "{\"status\":\"error\",\"message\":\"Rust bridge returned null pointer\"}"
      }

      let resultString = String(cString: resultPtr)
      free_rust_string_native(UnsafeMutablePointer(mutating: resultPtr))

      return resultString
    }.runOnQueue(bridgeQueue)
  }
}

@_silgen_name("call_rust")
func call_rust_native(_ cmd: UnsafePointer<Int8>?, _ args: UnsafePointer<Int8>?) -> UnsafePointer<Int8>?

@_silgen_name("free_rust_string")
func free_rust_string_native(_ s: UnsafeMutablePointer<Int8>?)
