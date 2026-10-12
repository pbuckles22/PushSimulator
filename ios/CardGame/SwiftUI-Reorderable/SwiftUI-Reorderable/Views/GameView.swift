/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
The playing area layout with a single reorder container managing drag and
 drop across all piles.
*/

import PushShoe
import SwiftUI

struct GameView: View {
    var game: Game

    var body: some View {
        GeometryReader { proxy in
            let spacing: CGFloat = 10
            let cardWidth = (proxy.size.width - 6 * spacing) / 7
            VStack {
                HStack(alignment: .top, spacing: spacing) {
                    Group {
                        RemainderView(game: game)
                        JokerGap()
                        ForEach(CardValue.Suit.allCases) { suit in
                            DestinationView(game: game, suit: suit)
                        }
                    }
                    .frame(width: cardWidth)
                }
                .padding(.bottom, 20)
                LocalHandRow(cardWidth: cardWidth, spacing: spacing)
                    .padding(.bottom, 20)
                HStack(alignment: .top, spacing: spacing) {
                    ForEach(0..<7) { index in
                        PileView(game: game, index: index)
                            .frame(width: cardWidth)
                    }
                }
                .frame(maxHeight: .infinity, alignment: .top)
                .reorderContainer(for: CardValue.self, in: Card.Group.self) { difference in
                    game.moveCards(difference: difference)
                }
                .dragContainer(for: CardValue.self) { cardID in
                    game.cardStack(startingAt: cardID)
                }
                .dragPreviewsFormation(.stack)
                .dragConfiguration(DragConfiguration(allowMove: true))
                .dropDestination(for: CardValue.self) { newCards, session in
                    if let destination = session.reorderDestination(
                        for: CardValue.self, in: Card.Group.self) {
                        game.insertCards(newCards, to: destination)
                    }
                }
                .dropConfiguration { session in
                    // Calculate which pile is being dragged over.
                    let alignedX = session.location.x - 0.5 * spacing
                    let pile = Int(alignedX / (cardWidth + spacing))
                    let destination = ReorderDifference<CardValue, Card.Group>
                        .Destination(position: .end, collectionID: .pile(pile))
                    // Check if the move is allowed.
                    let allowed = session.suggestedOperations.contains(.move)
                    && game.validateMove(session: session, destination: destination)
                    let operation: DropOperation = allowed ? .move : .forbidden
                    return DropConfiguration(operation: operation, destination: destination)
                }
            }
            .dropPreviewsFormation(.stack)
        }
        .padding()
    }
}

#Preview {
    @Previewable @State var cards = [
        CardValue(rank: .ace, suit: .clubs),
        CardValue(rank: .ace, suit: .diamonds),
        CardValue(rank: .ace, suit: .hearts),
        CardValue(rank: .ace, suit: .spades)
    ]

    HStack {
        ForEach(cards) { card in
            CardFaceView(card: card)
        }
        .reorderable()
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .padding()
    .background(.green.gradient)
    .reorderContainer(for: CardValue.self) { difference in
        cards.apply(difference: difference)
    }
}

/// Seat 0's hand, left to right, under the top row. Their face view stays theirs.
/// These cards do not join the pile drag. Nothing lifts into the engine.
private struct LocalHandRow: View {
    var cardWidth: CGFloat
    var spacing: CGFloat

    private var hand: LocalHand { LocalHand.shown() }

    var body: some View {
        HStack(alignment: .top, spacing: spacing) {
            ForEach(Array(hand.slots.enumerated()), id: \.element.id) { index, slot in
                drawn(slot)
                    .frame(width: cardWidth)
                    .accessibilityElement(children: .ignore)
                    .accessibilityIdentifier("hand-face-\(index)")
                    .accessibilityLabel(slot.spoken)
                    .overlay {
                        if let mark = slot.wild {
                            WildMarkBadge(spoken: mark.spoken, identifier: "hand-wild-\(index)")
                        }
                    }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private func drawn(_ slot: LocalHand.Slot) -> some View {
        switch slot.face {
        case .natural(let natural):
            if let value = CardValue(natural: natural) {
                CardFaceView(card: value)
            }
        case .joker(let joker):
            CardFaceView(joker: joker)
        }
    }
}

/// The empty column between the turned card and the suit piles. Their spacer stays.
/// The exhibit joker draws there with their face view, and the exhibit two under it.
private struct JokerGap: View {
    var body: some View {
        VStack(spacing: 10) {
            ZStack {
                CardBackView()
                    .hidden()
                if let face = JokerFace.shown() {
                    CardFaceView(joker: face)
                        .accessibilityElement(children: .ignore)
                        .accessibilityIdentifier("joker-face")
                        .accessibilityLabel(face.spoken)
                        .overlay {
                            if let mark = WildMark.shownJoker() {
                                WildMarkBadge(spoken: mark.spoken, identifier: "joker-wild-mark")
                            }
                        }
                }
            }
            if let face = NaturalFace.shownTwo(),
               let value = CardValue(natural: face) {
                CardFaceView(card: value)
                    .accessibilityElement(children: .ignore)
                    .accessibilityIdentifier("two-face")
                    .accessibilityLabel(face.spoken)
                    .overlay {
                        if let mark = WildMark.shownTwo() {
                            WildMarkBadge(spoken: mark.spoken, identifier: "two-wild-mark")
                        }
                    }
            }
        }
    }
}

extension Array {
    mutating func apply<CollectionID: Hashable & Sendable>(
        difference: ReorderDifference<Element.ID, CollectionID>
    ) where Element: Identifiable, Element.ID: Sendable {
        // Find the source card that moved.
        guard let sourceIndex = firstIndex(
            where: { $0.id == difference.sources[0] })
        else { return }
        let movedCard = remove(at: sourceIndex)

        // Find the destination of that card.
        var destination: Int
        switch difference.destination.position {
        case let .before(value):
            guard let index = firstIndex(where: { $0.id == value })
            else { return }
            destination = index
        case .end:
            destination = endIndex
        }
        insert(movedCard, at: destination)
    }
}
