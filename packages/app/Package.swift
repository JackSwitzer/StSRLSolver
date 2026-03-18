// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "SpireMonitor",
    platforms: [.macOS(.v14)],
    targets: [
        .executableTarget(
            name: "SpireMonitor",
            path: "SpireMonitor"
        )
    ]
)
