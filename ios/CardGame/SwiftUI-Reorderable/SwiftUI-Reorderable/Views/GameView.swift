/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
The playing area layout with a single reorder container managing drag and
 drop across all piles.
*/

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
                        CardBackView()
                            .hidden()
                        ForEach(CardValue.Suit.allCases) { suit in
                            DestinationView(game: game, suit: suit)
                        }
                    }
                    .frame(width: cardWidth)
                }
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
