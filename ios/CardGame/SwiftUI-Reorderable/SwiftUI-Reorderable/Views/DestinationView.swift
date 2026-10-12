/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A view that shows cards belonging to a single suit that the player completes
 from the game, with the largest value card shown on top.
*/

import SwiftUI
import SwiftData

struct DestinationView: View {
    var game: Game
    var suit: CardValue.Suit
    @Query var cards: [Card]

    init(game: Game, suit: CardValue.Suit) {
        self.game = game
        self.suit = suit
        let id = game.persistentModelID
        let group = Card.Group.destination(suit).intValue
        let index = suit.rawValue
        _cards = Query(
            filter: #Predicate<Card> {
                $0.game.persistentModelID == id && $0.group == group
                    && $0.groupIndex == index
            },
            sort: \.position)
    }

    var body: some View {
        ZStack {
            CardPlaceholderView()
            Image(systemName: suit.systemImageName)
                .resizable()
                .scaledToFit()
                .foregroundStyle(.primary.opacity(0.15))
                .padding()
            ForEach(cards) { card in
                CardFaceView(card: card.value)
                    .draggable(item: card.value)
            }
        }
        .dropDestination(for: CardValue.self) { items, session in
            guard items.count == 1 else { return }
            game.insertCard(items[0], to: suit)
        }
        .dropConfiguration { session in
            guard session.suggestedOperations.contains(.move),
                  let draggedCards = session.localSession?.draggedItemIDs(
                    for: CardValue.self),
                  draggedCards.count == 1,
                  game.validateMove(
                    value: draggedCards[0], destination: .destination(suit))
            else {
                return DropConfiguration(operation: .forbidden)
            }
            return DropConfiguration(operation: .move)
        }
    }
}
