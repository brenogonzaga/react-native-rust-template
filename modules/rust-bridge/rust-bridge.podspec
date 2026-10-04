require 'json'

package = JSON.parse(File.read(File.join(__dir__, 'package.json')))

Pod::Spec.new do |s|
  s.name           = 'rust-bridge'
  s.version        = package['version']
  s.summary        = package['description']
  s.license        = package['license']
  s.author         = package['author']
  s.homepage       = package['homepage']
  s.platforms      = { :ios => '16.4' }
  s.source         = { :path => '.' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  s.source_files = 'ios/**/*.swift'

  s.preserve_paths = 'ios/RustBridge.h', 'ios/module.modulemap'
  s.pod_target_xcconfig = {
    'SWIFT_INCLUDE_PATHS' => '$(PODS_TARGET_SRCROOT)/ios'
  }

  # Built by scripts/setup.js with a device slice and a simulator slice, so
  # Xcode links the right one for any destination (simulator, device, archive).
  s.vendored_frameworks = 'ios/RustBridge.xcframework'
end
