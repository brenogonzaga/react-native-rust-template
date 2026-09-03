package com.myapp.rustbridge

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import kotlinx.coroutines.CoroutineName
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel

class RustBridgeModule : Module() {
  private val bridgeScope =
    CoroutineScope(Dispatchers.IO + SupervisorJob() + CoroutineName("myapp.rust-bridge"))

  private external fun callRustNative(envelopeJson: String): String

  override fun definition() = ModuleDefinition {
    Name("RustBridge")

    AsyncFunction("callRust") { envelope: String ->
      loadError?.let {
        """{"status":"error","kind":"internal","reason":"native library unavailable: ${it.message}"}"""
      } ?: callRustNative(envelope)
    }.runOnQueue(bridgeScope)

    OnDestroy {
      bridgeScope.cancel()
    }
  }

  companion object {
    private val loadError: Throwable? =
      try {
        System.loadLibrary("rust_bridge")
        null
      } catch (t: Throwable) {
        t
      }
  }
}
