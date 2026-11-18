// swift-tools-version: 6.0
//
// Ulana for Swift.
//
// - `Ulana`: the frame types, the SwiftUI renderer (`UlanaView`), and
//   `ServerSession`, which draws views that run on a server (`ulana serve`).
//   Pure Swift.
// - `UlanaLocal`: runs views on the device with the Rust runtime. It needs
//   UlanaCore.xcframework, built by `scripts/build-xcframework.sh`; until then
//   the product is left out and `Ulana` still builds.

import Foundation
import PackageDescription

let hasCore = FileManager.default.fileExists(
    atPath: Context.packageDirectory + "/UlanaCore.xcframework/Info.plist")

var products: [Product] = [
    .library(name: "Ulana", targets: ["Ulana"])
]
var targets: [Target] = [
    .target(name: "Ulana"),
    .testTarget(name: "UlanaTests", dependencies: ["Ulana"]),
]

if hasCore {
    products.append(.library(name: "UlanaLocal", targets: ["UlanaLocal"]))
    targets += [
        .binaryTarget(name: "UlanaCore", path: "UlanaCore.xcframework"),
        .target(
            name: "UlanaLocal",
            dependencies: ["Ulana", "UlanaCore"],
            linkerSettings: [.linkedLibrary("c++")]
        ),
        .testTarget(name: "UlanaLocalTests", dependencies: ["UlanaLocal"]),
    ]
}

let package = Package(
    name: "Ulana",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: products,
    targets: targets
)
