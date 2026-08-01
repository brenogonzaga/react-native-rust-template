package com.myapp.rustbridge

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import kotlinx.coroutines.*

class RustBridgeModule : Module() {
  private val bridgeScope =
    CoroutineScope(Dispatchers.IO + SupervisorJob() + CoroutineName("myapp.rust-bridge"))

  override fun definition() = ModuleDefinition {
    Name("RustBridge")

    AsyncFunction("callRust") { command: String, payload: String ->
      callRustNative(command, payload)
    }.runOnQueue(bridgeScope)

    OnDestroy {
      bridgeScope.cancel()
    }
  }

  companion object {
    init {
      System.loadLibrary("rust_bridge")
    }

    @JvmStatic
    private external fun callRustNative(command: String, argsJson: String): String
  }
}
