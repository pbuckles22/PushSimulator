import SwiftUI

/// The local seat's cards, left to right in the order they were given.
public struct HandLayout: Equatable {
    public var faces: [CardFace]

    public init(cards: [BoardCard]) {
        faces = cards.map { CardFace($0) }
    }
}

/// True only for the card that is in the air. The other slots stay.
public func handSlotIsHidden(cardId: UInt32, liftedId: UInt32?) -> Bool {
    liftedId == cardId
}

/// The hand slot's opacity for one drag session. In the air it is hidden. After the
/// session it is showing again.
public struct HandSlotVisibility: Equatable {
    public var alpha: CGFloat

    public static func duringDrag(of cardId: UInt32) -> HandSlotVisibility {
        HandSlotVisibility(alpha: handSlotIsHidden(cardId: cardId, liftedId: cardId) ? 0 : 1)
    }

    public static func afterDrag(of cardId: UInt32) -> HandSlotVisibility {
        HandSlotVisibility(alpha: handSlotIsHidden(cardId: cardId, liftedId: nil) ? 0 : 1)
    }
}

/// The lines drawn on a card that is in the air. Same face as the slot.
public func handCardPreviewLines(_ card: BoardCard) -> [String] {
    let face = CardFace(card)
    var lines = [face.pip]
    if !face.suitSymbol.isEmpty {
        lines.append(face.suitSymbol)
    }
    if face.isLocked {
        lines.append("locked")
    }
    return lines
}

/// The local hand. An empty hand says so. A card in the hand can lift.
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
                        ForEach(Array(cards.enumerated()), id: \.offset) { _, card in
                            handCard(card)
                        }
                    }
                    .frame(minHeight: 44, alignment: .leading)
                }
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("hand")
            }
        }
    }

    @ViewBuilder
    private func handCard(_ card: BoardCard) -> some View {
        #if os(iOS)
        HandCardDragSource(card: card)
        #else
        CardView(card: card)
            .draggable(CardDrag(cards: [card]))
        #endif
    }
}
