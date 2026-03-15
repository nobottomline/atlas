// Atlas Platform
import SwiftUI
import Sources
import Library
import Downloads
import Settings
import SourceRuntime
import SourceRegistry
import Core

@main
struct AtlasApp: App {

    @State private var sourceManager: SourceManager

    init() {
        let registryURL = URL(string:
            UserDefaults.standard.string(forKey: "registryURL")
            ?? "https://raw.githubusercontent.com/atlas-platform/atlas-registry/main"
        )!

        let manager = SourceManager(
            runtime: AtlasRuntime(),
            registryClient: RegistryClient(),
            storage: SourceStorageAdapter(),
            registryBaseURL: registryURL
        )

        self._sourceManager = State(initialValue: manager)
    }

    var body: some Scene {
        WindowGroup {
            ContentView(sourceManager: sourceManager)
                .task {
                    await sourceManager.loadInstalledSources()
                }
        }
    }
}

struct ContentView: View {

    let sourceManager: SourceManager

    var body: some View {
        TabView {
            LibraryView()
                .tabItem { Label("Library", systemImage: "books.vertical") }

            SourcesView(sourceManager: sourceManager)
                .tabItem { Label("Sources", systemImage: "puzzlepiece.extension") }

            DownloadsView()
                .tabItem { Label("Downloads", systemImage: "arrow.down.circle") }

            SettingsView()
                .tabItem { Label("Settings", systemImage: "gear") }
        }
    }
}
