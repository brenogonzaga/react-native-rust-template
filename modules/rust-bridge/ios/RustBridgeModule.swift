import ExpoModulesCore
import RustBridgeFFI

public class RustBridgeModule: Module {
  public func definition() -> ModuleDefinition {
    Name("RustBridge")
    AsyncFunction("callRust") { (envelope: String) -> String in
      guard let resultPtr = call_rust(envelope) else {
        return #"{"status":"error","kind":"internal","reason":"rust bridge returned a null pointer"}"#
      }
      defer { free_rust_string(resultPtr) }

      return String(cString: resultPtr)
    }
  }
}
