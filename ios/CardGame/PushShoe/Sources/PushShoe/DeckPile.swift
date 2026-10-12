import PushFfi
import SwiftUI

/// The digits their deck pile shows. The number is the undealt shoe.
public enum DeckPile {
    public static func text(undealt: UInt32) -> String {
        String(undealt)
    }
}

/// The undealt shoe. Story 3.2 reads this count onto their deck pile.
public enum UndealtShoe {
    public static func count() -> UInt32 {
        Game().getDeckSize()
    }
}

/// The count drawn on their deck pile.
public struct DeckPileCount: View {
    public init() {}

    /// The string `body` draws. It is the undealt shoe.
    public static func shown() -> String {
        DeckPile.text(undealt: UndealtShoe.count())
    }

    public var body: some View {
        Text(Self.shown())
            .font(.title2.bold())
            .foregroundStyle(.white)
            .shadow(color: .black.opacity(0.55), radius: 2, y: 1)
            .accessibilityIdentifier("deck-pile")
    }
}
