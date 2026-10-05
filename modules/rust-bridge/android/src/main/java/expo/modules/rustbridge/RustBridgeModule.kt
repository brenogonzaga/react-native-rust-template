package expo.modules.rustbridge

import expo.modules.kotlin.exception.Exceptions
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import kotlinx.coroutines.CoroutineName
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel

private const val BRIDGE_PARALLELISM = 6

class RustBridgeModule : Module() {
  private val bridgeScope =
    CoroutineScope(
      Dispatchers.IO.limitedParallelism(BRIDGE_PARALLELISM, "rust-bridge") +
        SupervisorJob() +
        CoroutineName("rust-bridge")
    )

  private external fun callRustNative(envelopeJson: String): String

  override fun definition() = ModuleDefinition {
    Name("RustBridge")

    Constant("dataDir") {
      (appContext.reactContext ?: throw Exceptions.ReactContextLost()).filesDir.absolutePath
    }

    AsyncFunction("callRust") { envelope: String ->
      loadError?.let { throw IllegalStateException("native library unavailable: ${it.message}", it) }
      callRustNative(envelope)
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
