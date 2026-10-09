import SwiftUI

/// The local seat's cards, left to right in the order they were given.
public struct HandLayout: Equatable {
    public var faces: [CardFace]

    public init(cards: [BoardCard]) {
        faces = cards.map { CardFace($0) }
    }
}

/// The local hand. An empty hand says so. A card in the hand can lift.
public struct HandView: View {
    public let cards: [BoardCard]
    @State private var liftedIndex: Int?

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

    /// A hand with cards can lift. An empty hand cannot.
    public var allowsDrag: Bool {
        !cards.isEmpty
    }

    /// True when this id is in the hand. The face on the payload is not checked.
    public func canLift(_ card: BoardCard) -> Bool {
        cards.contains { $0.id == card.id }
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
                        ForEach(Array(cards.enumerated()), id: \.offset) { index, card in
                            CardView(card: card)
                                .opacity(liftedIndex == index ? 0 : 1)
                                .draggable(CardDrag(cards: [card])) {
                                    CardView(card: card)
                                        .onAppear { liftedIndex = index }
                                        .onDisappear { liftedIndex = nil }
                                }
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
