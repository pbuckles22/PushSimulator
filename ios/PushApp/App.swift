import PushUI
import SwiftUI

@main
struct PushApp: App {
    @StateObject private var exhibit: LiveExhibit

    init() {
        _exhibit = StateObject(wrappedValue: LiveExhibit(game: Game.exhibitSetAndRun()))
    }

    var body: some Scene {
        WindowGroup {
            GameBoardScreen(table: exhibit.table, engine: exhibit.answer)
        }
    }
}
