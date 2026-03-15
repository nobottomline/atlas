// Atlas Platform
import SwiftUI
import Core

/// Horizontal paging reader for manga chapters.
public struct ReaderView: View {

    let pages: [Page]
    let chapterTitle: String

    @State private var currentPage = 0
    @State private var showOverlay = true
    @Environment(\.dismiss) private var dismiss

    public init(pages: [Page], chapterTitle: String) {
        self.pages = pages
        self.chapterTitle = chapterTitle
    }

    public var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            TabView(selection: $currentPage) {
                ForEach(Array(pages.enumerated()), id: \.offset) { index, page in
                    pageView(page)
                        .tag(index)
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .never))
            .onTapGesture {
                withAnimation(.easeInOut(duration: 0.2)) {
                    showOverlay.toggle()
                }
            }

            if showOverlay {
                overlay
            }
        }
        .navigationBarHidden(true)
        .statusBarHidden(!showOverlay)
        .ignoresSafeArea()
    }

    @ViewBuilder
    private func pageView(_ page: Page) -> some View {
        switch page.data {
        case .url(let urlString):
            AsyncImage(url: URL(string: urlString)) { phase in
                switch phase {
                case .success(let image):
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                case .failure:
                    VStack {
                        Image(systemName: "exclamationmark.triangle")
                            .font(.largeTitle)
                        Text("Failed to load page")
                            .font(.caption)
                    }
                    .foregroundStyle(.secondary)
                default:
                    ProgressView()
                        .tint(.white)
                }
            }

        case .base64(let data):
            if let imageData = Data(base64Encoded: data),
               let uiImage = UIImage(data: imageData) {
                Image(uiImage: uiImage)
                    .resizable()
                    .aspectRatio(contentMode: .fit)
            } else {
                Text("Invalid image data")
                    .foregroundStyle(.secondary)
            }

        case .text(let text):
            ScrollView {
                Text(text)
                    .foregroundStyle(.white)
                    .padding()
            }
        }
    }

    private var overlay: some View {
        VStack {
            // Top bar
            HStack {
                Button { dismiss() } label: {
                    Image(systemName: "xmark")
                        .font(.title3)
                        .foregroundStyle(.white)
                        .padding(8)
                        .background(.ultraThinMaterial, in: Circle())
                }

                Spacer()

                Text(chapterTitle)
                    .font(.subheadline)
                    .foregroundStyle(.white)
                    .lineLimit(1)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(.ultraThinMaterial, in: Capsule())

                Spacer()

                Text("\(currentPage + 1)/\(pages.count)")
                    .font(.subheadline.monospacedDigit())
                    .foregroundStyle(.white)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(.ultraThinMaterial, in: Capsule())
            }
            .padding()

            Spacer()

            // Page slider
            if pages.count > 1 {
                Slider(
                    value: Binding(
                        get: { Double(currentPage) },
                        set: { currentPage = Int($0) }
                    ),
                    in: 0...Double(max(pages.count - 1, 1)),
                    step: 1
                )
                .padding(.horizontal, 40)
                .padding(.bottom, 40)
                .tint(.white)
            }
        }
        .transition(.opacity)
    }
}
