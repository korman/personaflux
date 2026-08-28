// swift-tools-version: 5.7
import PackageDescription

let package = Package(
    name: "PersonaFluxSwift",
    platforms: [.iOS(.v13), .macOS(.v12)],
    products: [
        .library(name: "PersonaFlux", targets: ["PersonaFlux"]),
    ],
    targets: [
        .target(
            name: "CPersonaFlux",
            path: "Sources/CPersonaFlux",
            publicHeadersPath: "include"
        ),
        .target(name: "PersonaFlux", dependencies: ["CPersonaFlux"]),
        .testTarget(name: "PersonaFluxTests", dependencies: ["PersonaFlux", "CPersonaFlux"]),
    ]
)
