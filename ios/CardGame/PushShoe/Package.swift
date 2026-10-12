// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "PushShoe",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "PushShoe", targets: ["PushShoe"]),
    ],
    targets: [
        .systemLibrary(name: "push_ffiFFI", path: "Generated"),
        .target(
            name: "PushFfi",
            dependencies: ["push_ffiFFI"],
            path: "Sources/PushFfi"
        ),
        .target(
            name: "PushShoe",
            dependencies: ["PushFfi"]
        ),
        .testTarget(
            name: "PushShoeTests",
            dependencies: ["PushShoe", "PushFfi"]
        ),
    ]
)
