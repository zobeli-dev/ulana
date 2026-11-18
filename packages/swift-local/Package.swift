// swift-tools-version: 6.0
//
// UlanaLocal: run Ulana views on the device with the Rust runtime, the same
// sandbox, limits, and checks as on a server.
//
// Build the runtime first, from the repository root:
//   scripts/build-xcframework.sh
// Then add this folder as a local package (Xcode: File > Add Package
// Dependencies > Add Local…), or depend on it by path.

import PackageDescription

let package = Package(
    name: "UlanaLocal",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "UlanaLocal", targets: ["UlanaLocal"])
    ],
    dependencies: [
        .package(name: "Ulana", path: "../..")
    ],
    targets: [
        .binaryTarget(name: "UlanaCore", path: "UlanaCore.xcframework"),
        .target(
            name: "UlanaLocal",
            dependencies: [.product(name: "Ulana", package: "Ulana"), "UlanaCore"],
            linkerSettings: [.linkedLibrary("c++")]
        ),
        .testTarget(name: "UlanaLocalTests", dependencies: ["UlanaLocal"]),
    ]
)
