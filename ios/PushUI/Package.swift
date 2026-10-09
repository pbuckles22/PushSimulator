// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "PushUI",
    platforms: [.iOS(.v16), .macOS(.v13)],
    products: [
        .library(name: "PushUI", targets: ["PushUI"]),
    ],
    targets: [
        .target(name: "PushUI"),
        .testTarget(name: "PushUITests", dependencies: ["PushUI"]),
    ]
)
