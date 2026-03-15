// Atlas Platform
import SwiftUI
import Sources
import Library
import Downloads
import Settings
import SourceRuntime
import SourceRegistry
import Core

let defaultRegistryURL = "https://raw.githubusercontent.com/nobottomline/atlas/main/registry"

@main
struct AtlasApp: App {

    @State private var sourceManager: SourceManager

    init() {
        let urlString = UserDefaults.standard.string(forKey: "registryURL") ?? defaultRegistryURL
        let registryURL = URL(string: urlString) ?? URL(string: defaultRegistryURL)!

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
