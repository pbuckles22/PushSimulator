/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
Top-level view with a sidebar listing saved games and the active game.
*/

import SwiftUI
import SwiftData

struct ContentView: View {
    @Environment(\.modelContext) private var modelContext
    @Query(sort: \Game.number, order: .reverse) var games: [Game]
    @State private var selection: Game?
    @AppStorage("gameCount") var gameCount: Int = 0

    private var screenVersion: String {
        var url = Bundle.main.bundleURL
        if url.pathExtension == "dylib" {
            url.deleteLastPathComponent()
            url.deleteLastPathComponent()
            url.deleteLastPathComponent()
        }
        let bundle = Bundle(url: url) ?? Bundle.main
        return bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
    }

    var body: some View {
        NavigationSplitView {
            List(games, selection: $selection) { game in
                NavigationLink(value: game) {
                    GameListRow(game: game)
                }
            }
        } detail: {
            VStack(alignment: .leading, spacing: 0) {
                Text(screenVersion)
                    .accessibilityIdentifier("screen-version")
                    .accessibilityLabel(screenVersion)
                if let selection {
                    GameView(game: selection)
                } else {
                    ContentUnavailableView(
                        "No Game Selected",
                        systemImage: "suit.club.fill",
                        description: Text(
                            "Select a game from the sidebar or create a new one.")
                    )
                }
            }
        }
        .toolbar {
            Button("New", systemImage: "plus") {
                gameCount += 1
                addGame(number: gameCount)
            }
            Button("Delete", systemImage: "trash", role: .destructive) {
                deleteSelectedGame()
            }
            .disabled(selection == nil)
        }
        .navigationTitle("Solitaire")
        .containerBackground(.green.gradient, for: .window)
        .onAppear {
            if selection == nil, !games.isEmpty {
                selection = games[0]
            }
        }
    }

    func addGame(number: Int) {
        let game = Game.random(number: number)
        modelContext.insert(game)
        selection = game
    }

    func deleteSelectedGame() {
        defer { selection = nil }
        guard let selection else { return }
        modelContext.delete(selection)
    }
}

private struct GameListRow: View {
    let game: Game

    var body: some View {
        VStack(alignment: .leading) {
            Text("Game #\(game.number)")
                .font(.headline)
            Text("\(game.moveCount) moves")
                .font(.subheadline)
        }
    }
}
