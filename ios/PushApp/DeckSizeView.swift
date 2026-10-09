import SwiftUI

/// The first screen. It asks the Rust engine how big the shoe is and shows that number.
public struct DeckSizeView: View {
    private let deckSize: UInt32

    public init() {
        deckSize = Game().getDeckSize()
    }

    public var body: some View {
        Text("Deck size \(deckSize)")
    }
}
