// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "RsIceSettingsHost",
    platforms: [.macOS(.v14)],
    targets: [
        .executableTarget(
            name: "RsIceSettingsHost",
            path: "macos",
            exclude: ["Package.swift"],
            sources: [
                "Sources",
            ],
            resources: [
                .process("views/settings.crepus"),
            ]
        ),
    ]
)
