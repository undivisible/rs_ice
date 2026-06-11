// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "RsIceSettingsHost",
    platforms: [.macOS(.v14)],
    dependencies: [
        .package(name: "Aurorality", path: "../aurorality"),
    ],
    targets: [
        .executableTarget(
            name: "RsIceSettingsHost",
            dependencies: [
                .product(name: "Aurorality", package: "Aurorality"),
            ],
            path: "macos",
            exclude: ["Package.swift"],
            sources: [
                "Sources",
            ],
            resources: [
                .process("views/settings.crepus"),
            ],
            linkerSettings: [
                .unsafeFlags([
                    "-L", "target/debug", "-laurorality_core",
                    "-framework", "JavaScriptCore",
                ]),
            ]
        ),
    ]
)
