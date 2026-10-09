// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "PushApp",
    platforms: [.iOS(.v16)],
    products: [
        .library(name: "PushApp", targets: ["PushApp"]),
    ],
    targets: [
        .binaryTarget(name: "PushCore", path: "PushCore.xcframework"),
        .target(
            name: "PushApp",
            dependencies: ["PushCore"],
            path: "PushApp",
            exclude: [
                "Generated/push_ffiFFI.h",
                "Generated/push_ffiFFI.modulemap",
            ]
        ),
    ]
)
