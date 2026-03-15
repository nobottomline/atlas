import ProjectDescription

// MARK: - Shared settings

let baseSettings = SettingsDictionary()
    .swiftVersion("6.0")
    .automaticCodeSigning(devTeam: "569SDR24JU")

// MARK: - Module factory

/// Creates a framework target + optional test target for a named module.
func module(
    name: String,
    dependencies: [TargetDependency] = [],
    resources: ResourceFileElements? = nil,
    hasTests: Bool = true
) -> [Target] {
    var targets: [Target] = [
        .target(
            name: name,
            destinations: [.iPhone, .iPad],
            product: .framework,
            bundleId: "com.atlas.\(name.lowercased())",
            deploymentTargets: .iOS("17.0"),
            sources: ["Modules/\(name)/Sources/**"],
            resources: resources,
            dependencies: dependencies,
            settings: .settings(base: baseSettings)
        )
    ]
    if hasTests {
        targets.append(
            .target(
                name: "\(name)Tests",
                destinations: [.iPhone, .iPad],
                product: .unitTests,
                bundleId: "com.atlas.\(name.lowercased())tests",
                deploymentTargets: .iOS("17.0"),
                sources: ["Modules/\(name)/Tests/**"],
                dependencies: [.target(name: name)],
                settings: .settings(base: baseSettings)
            )
        )
    }
    return targets
}

// MARK: - App target

let appTarget = Target.target(
    name: "App",
    destinations: [.iPhone, .iPad],
    product: .app,
    bundleId: "com.atlas.app",
    deploymentTargets: .iOS("17.0"),
    infoPlist: .extendingDefault(with: [
        "UILaunchScreen": ["UIColorName": ""],
    ]),
    sources: ["Modules/App/Sources/**"],
    resources: ["Modules/App/Resources/**"],
    dependencies: [
        .target(name: "Sources"),
        .target(name: "Library"),
        .target(name: "Reader"),
        .target(name: "Downloads"),
        .target(name: "Settings"),
        .target(name: "SourceRuntime"),
        .target(name: "SourceRegistry"),
        .target(name: "Core"),
    ],
    settings: .settings(base: baseSettings)
)

// MARK: - Infrastructure modules

let sourceRuntimeTargets = module(
    name: "SourceRuntime",
    dependencies: [
        .target(name: "Core"),
        .external(name: "WasmKit"),
        .external(name: "MessagePack"),
    ]
)

let sourceRegistryTargets = module(
    name: "SourceRegistry",
    dependencies: [
        .target(name: "Core"),
    ]
)

let coreTargets         = module(name: "Core",         dependencies: [])
let networkingTargets   = module(name: "Networking",   dependencies: [.target(name: "Core")])
let persistenceTargets  = module(name: "Persistence",  dependencies: [.target(name: "Core")])
let diagnosticsTargets  = module(name: "Diagnostics",  dependencies: [.target(name: "Core")])

// MARK: - Feature modules

let sourcesTargets = module(
    name: "Sources",
    dependencies: [
        .target(name: "SourceRuntime"),
        .target(name: "SourceRegistry"),
        .target(name: "Core"),
        .target(name: "Reader"),
    ]
)

let libraryTargets = module(
    name: "Library",
    dependencies: [
        .target(name: "SourceRuntime"),
        .target(name: "Core"),
    ]
)

let readerTargets = module(
    name: "Reader",
    dependencies: [
        .target(name: "Core"),
        .target(name: "SourceRuntime"),
    ]
)

let downloadsTargets = module(
    name: "Downloads",
    dependencies: [
        .target(name: "Core"),
    ]
)

let settingsTargets = module(
    name: "Settings",
    dependencies: [.target(name: "Core")]
)

// MARK: - All targets

var allTargets: [Target] = [appTarget]
allTargets += sourcesTargets
allTargets += libraryTargets
allTargets += readerTargets
allTargets += downloadsTargets
allTargets += settingsTargets
allTargets += sourceRuntimeTargets
allTargets += sourceRegistryTargets
allTargets += coreTargets
allTargets += networkingTargets
allTargets += persistenceTargets
allTargets += diagnosticsTargets

// MARK: - Project

let project = Project(
    name: "Atlas",
    organizationName: "Atlas Platform",
    options: .options(
        automaticSchemesOptions: .enabled(
            targetSchemesGrouping: .byNameSuffix(
                build: [],
                test: ["Tests"],
                run: []
            )
        )
    ),
    targets: allTargets,
    fileHeaderTemplate: .string("// Atlas Platform")
)
