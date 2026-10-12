/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A view that shows the remaining cards that the player can play into the
 piles.
*/

import SwiftUI
import SwiftData

struct RemainderView: View {
    @Query var cards: [Card]
    var game: Game
    @State private var hiddenCard: CardValue? = nil

    init(game: Game) {
        self.game = game
        let id = game.persistentModelID
        let group = Card.Group.remainder.intValue
        _cards = Query(
            filter: #Predicate<Card> {
                $0.game.persistentModelID == id && $0.group == group
            },
            sort: \.position)
    }

    var body: some View {
        Button {
            incrementCardIndex()
        } label: {
            ZStack {
                CardPlaceholderView()
                CardBackView()
                    .opacity(cards.isEmpty ? 0 : 1)
            }
        }
        .buttonStyle(.plain)
        .disabled(cards.isEmpty)
        ZStack {
            CardPlaceholderView()
            if let currentCard {
                CardFaceView(card: currentCard.value)
                    .draggable(containerItemID: currentCard.value)
                    .opacity(currentCard.value == hiddenCard ? 0 : 1)
            }
        }
        .dragContainer(for: CardValue.self) { cardID in
            [cardID]
        }
        .dragConfiguration(DragConfiguration(allowMove: true))
        .onDragSessionUpdated { session in
            switch session.phase {
            case .initial, .active:
                hiddenCard = session.draggedItemIDs(for: CardValue.self).first
            case let .ended(operation):
                // If the operation was successful, don't show a card until
                // the next one is ready to be shown.
                if operation == .forbidden || operation == .cancel {
                    hiddenCard = nil
                }
            default: break
            }
        }
        .onChange(of: currentCard?.value) { oldValue, newValue in
            if newValue != hiddenCard {
                hiddenCard = nil
            }
        }
    }

    /// The current card to show face up next to the deck.
    var currentCard: Card? {
        // The first position of the remainder deck should not show a card, so
        // that it's clear when you wrap around to the start. This code shifts
        // all of the indices up by one (by subtracting 1), so that the first
        // position is always that empty view.
        let index = game.remainderIndex - 1
        guard index >= 0, index < cards.count else { return nil }
        return cards[index]
    }

    func incrementCardIndex() {
        game.remainderIndex = (game.remainderIndex + 1) % (cards.count + 1)
    }
}
