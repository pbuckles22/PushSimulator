import SwiftUI

/// The local seat's cards, left to right in the order they were given.
public struct HandLayout: Equatable {
    public var faces: [CardFace]

    public init(cards: [BoardCard]) {
        faces = cards.map { CardFace($0) }
    }
}

/// The local hand. An empty hand says so. The cards are not draggable.
public struct HandView: View {
    public let cards: [BoardCard]

    public init(cards: [BoardCard]) {
        self.cards = cards
    }

    public var layout: HandLayout {
        HandLayout(cards: cards)
    }

    /// Set when the local seat has no cards.
    public var emptyTitle: String? {
        cards.isEmpty ? "No cards" : nil
    }

    /// Drag starts in a later story. This row only shows the cards.
    public var allowsDrag: Bool {
        false
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Hand")
                .accessibilityIdentifier("hand-label")
            if let emptyTitle {
                Text(emptyTitle)
                    .accessibilityIdentifier("empty-hand")
            } else {
                ScrollView(.horizontal, showsIndicators: true) {
                    HStack(spacing: 8) {
                        ForEach(Array(cards.enumerated()), id: \.offset) { _, card in
                            CardView(card: card)
                        }
                    }
                    .frame(minHeight: 44, alignment: .leading)
                }
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("hand")
            }
        }
    }
}
