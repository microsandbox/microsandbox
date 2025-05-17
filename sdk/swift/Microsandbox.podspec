Pod::Spec.new do |s|
  s.name             = 'Microsandbox'
  s.version          = '0.1.0'
  s.summary          = 'Swift SDK for Microsandbox'
  s.description      = <<-DESC
                       Microsandbox Swift SDK provides a simple interface to interact with Microsandbox services.
                       DESC
  s.homepage         = 'https://github.com/yourusername/microsandbox'
  s.license          = { :type => 'MIT', :file => 'LICENSE' }
  s.author           = { 'Your Name' => 'your.email@example.com' }
  s.source           = { :git => 'https://github.com/yourusername/microsandbox.git', :tag => s.version.to_s }

  s.ios.deployment_target = '13.0'
  s.osx.deployment_target = '10.15'
  s.swift_version = '5.0'

  s.source_files = 'sdk/swift/Sources/**/*'

  # Dependencies
  # Add any dependencies your SDK has here
  # s.dependency 'SomeDependency', '~> 1.0'
end
