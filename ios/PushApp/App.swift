import PushUI
import SwiftUI

@main
struct PushApp: App {
    @StateObject private var table = PublishedTable {
        livePicture(from: Game.exhibitSetAndRun())
    }

    var body: some Scene {
        WindowGroup {
            GameBoardScreen(table: table)
        }
    }
}
