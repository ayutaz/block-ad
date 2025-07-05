// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "AdBlock",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "AdBlockCore",
            targets: ["AdBlockCore", "AdBlockCoreFFI"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "AdBlockCore",
            dependencies: ["AdBlockCoreFFI"],
            path: "AdBlock"
        ),
        .binaryTarget(
            name: "AdBlockCoreFFI",
            path: "AdBlockCoreFFI.xcframework"
        ),
        .testTarget(
            name: "AdBlockTests",
            dependencies: ["AdBlockCore"],
            path: "AdBlockTests"
        ),
    ]
)