// swift-tools-version: 6.0
//
// Ulana for Swift: draw Ulana views with SwiftUI and follow views that run
// on a server.
//
//   .package(url: "https://github.com/zobeli-dev/ulana", branch: "main")
//
// Running views on the device is the separate `UlanaLocal` package in
// packages/swift-local, because it needs the Rust runtime built first.

import PackageDescription

let package = Package(
    name: "Ulana",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "Ulana", targets: ["Ulana"])
    ],
    targets: [
        .target(name: "Ulana", path: "packages/swift/Sources/Ulana"),
        .testTarget(name: "UlanaTests", dependencies: ["Ulana"], path: "packages/swift/Tests/UlanaTests"),
    ]
)
