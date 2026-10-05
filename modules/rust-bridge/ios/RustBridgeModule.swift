import ExpoModulesCore
import RustBridgeFFI

private let bridgeQueue: OperationQueue = {
  let queue = OperationQueue()
  queue.name = "rust-bridge"
  queue.qualityOfService = .userInitiated
  queue.maxConcurrentOperationCount = 6
  return queue
}()

public class RustBridgeModule: Module {
  public func definition() -> ModuleDefinition {
    Name("RustBridge")

    Constant("dataDir") {
      FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        .path(percentEncoded: false)
    }

    AsyncFunction("callRust") { (envelope: String, promise: Promise) in
      bridgeQueue.addOperation {
        guard let resultPtr = call_rust(envelope) else {
          promise.resolve(
            #"{"status":"error","kind":"internal","reason":"rust bridge returned a null pointer"}"#)
          return
        }
        defer { free_rust_string(resultPtr) }

        promise.resolve(String(cString: resultPtr))
      }
    }
  }
}
