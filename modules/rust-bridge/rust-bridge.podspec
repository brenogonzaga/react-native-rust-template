Pod::Spec.new do |s|
  s.name           = 'rust-bridge'
  s.version        = '1.0.0'
  s.summary        = 'Expo module bridging React Native to Rust'
  s.author         = 'Developer'
  s.homepage       = 'https://github.com/example/rust-bridge'
  s.platforms      = { :ios => '13.4' }
  s.source         = { :path => '.' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  s.source_files = 'ios/**/*.{h,m,mm,swift}'
  s.vendored_libraries = 'ios/lib/librust_bridge.a'
end
