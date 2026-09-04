Pod::Spec.new do |s|
  s.name           = 'rust-bridge'
  s.version        = '1.0.0'
  s.summary        = 'Expo module bridging React Native to Rust'
  s.author         = 'Developer'
  s.homepage       = 'https://github.com/example/rust-bridge'
  s.platforms      = { :ios => '16.0' }
  s.source         = { :path => '.' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  s.source_files = 'ios/**/*.swift'

  s.preserve_paths = 'ios/RustBridge.h', 'ios/module.modulemap'
  s.pod_target_xcconfig = {
    'SWIFT_INCLUDE_PATHS' => '$(PODS_TARGET_SRCROOT)/ios'
  }

  s.vendored_libraries = 'ios/lib/librust_bridge.a'
end
