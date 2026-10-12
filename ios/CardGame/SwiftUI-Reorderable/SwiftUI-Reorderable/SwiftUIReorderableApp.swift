/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
App entry point. Configures the SwiftData model container
 for game state persistence with undo support.
*/

import SwiftUI
import SwiftData

@main
struct SwiftUIReorderableApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
        .modelContainer(for: [Game.self, Card.self])
    }
}
