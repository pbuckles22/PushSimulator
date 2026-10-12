/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A model class that captures the state of a single game.
*/

import SwiftUI
import SwiftData

@Model
final class Game {
    @Relationship(deleteRule: .cascade, inverse: \Card.game)
    var cards: [Card]
    var remainderIndex: Int = 0

    var creationDate: Date
    @Attribute(.unique) var number: Int
    var moveCount: Int = 0

    init(number: Int, cards: [Card], creationDate: Date = .now) {
        self.number = number
        self.cards = cards
        self.creationDate = creationDate
    }
}

// MARK: Initialization

extension Game {
    /// Creates a new game, shuffling the cards using the system random number
    /// generator.
    static func random(number: Int) -> Game {
        var generator = SystemRandomNumberGenerator()
        return random(number: number, using: &generator)
    }

    /// Creates a new game, randomized using the provided generator.
    static func random(
        number: Int,
        using generator: inout some RandomNumberGenerator
    ) -> Game {
        let game = Game(number: number, cards: [])
        var cards = CardValue.deck.shuffled(using: &generator)

        // Fill the piles with 1-7 cards each based on the index.
        for column in 0..<7 {
            for index in 0...column {
                game.cards.append(Card(
                    value: cards.removeLast(), isFaceDown: index < column,
                    game: game, group: .pile(column), position: index))
            }
        }

        // Move the remaining cards into the remainder deck.
        for (offset, card) in cards.enumerated() {
            game.cards.append(Card(
                value: card, isFaceDown: false, game: game, group: .remainder,
                position: offset))
        }

        return game
    }
}

// MARK: Mutation

extension Game {
    /// Moves cards from anywhere in the game to the end of the specified group.
    func appendCards(_ cards: [Card], to group: Card.Group) {
        let startOffset = appendPosition(for: group)
        var affectedGroups = Set<Card.Group>()
        for (offset, card) in cards.enumerated() {
            affectedGroups.insert(card.groupID)
            card.groupID = group
            card.position = startOffset + offset
        }
        reindexCards(groups: affectedGroups)
        moveCount += 1
    }

    /// Returns the topmost card for the provided group.
    private func lastCard(for group: Card.Group) -> Card? {
        var last: Card? = nil
        for card in cards {
            if card.groupID == group, card.position > (last?.position ?? -1) {
                last = card
            }
        }
        return last
    }

    /// Returns the position that is one past the end for the provided group.
    private func appendPosition(for group: Card.Group) -> Int {
        return (lastCard(for: group)?.position ?? -1) + 1
    }

    /// Corrects any position inconsistencies in the cards after updating the
    /// provided groups.
    private func reindexCards(groups: Set<Card.Group>) {
        for group in groups {
            let groupCards = cards
                .filter { $0.groupID == group }
                .sorted { $0.position < $1.position }

            for (index, card) in groupCards.enumerated() {
                card.position = index
            }

            // Flip any last cards in a pile, if they're face down.
            if case .pile = group, let last = groupCards.last, last.isFaceDown {
                last.isFaceDown = false
            }
        }
    }
}

// MARK: View hooks

extension Game {
    func moveCards(difference: ReorderDifference<CardValue, Card.Group>) {
        let cards = difference.sources.compactMap { value in
            cards.first { $0.value == value }
        }
        appendCards(cards, to: difference.destination.collectionID)
    }

    func cardStack(startingAt value: CardValue) -> [CardValue] {
        guard let card = cards.first(where: { $0.value == value }) else {
            return []
        }
        return cards
            .filter {
                $0.groupID == card.groupID && $0.position >= card.position
            }
            .sorted { $0.position < $1.position }
            .map { $0.value }
    }

    func insertCards(
        _ values: [CardValue],
        to destination: ReorderDifference<CardValue, Card.Group>.Destination
    ) {
        let cards = values.compactMap { value in
            cards.first { $0.value == value }
        }
        appendCards(cards, to: destination.collectionID)
    }

    func insertCard(
        _ value: CardValue, to suit: CardValue.Suit
    ) {
        let cards = cards.first { $0.value == value }.map { [$0] } ?? []
        appendCards(cards, to: .destination(suit))
    }
}

extension Game {
    func validateMove(value: CardValue, destination: Card.Group) -> Bool {
        switch destination {
        case .remainder: return false
        case .pile:
            // Ensure that the value is one lower and the opposite suit.
            if let last = lastCard(for: destination) {
                return value.suit.playingColor != last.value.suit.playingColor
                    && value.rank.rawValue + 1 == last.value.rank.rawValue
            }
            // Any card can go on an empty pile.
            return true
        case let .destination(suit):
            guard value.suit == suit else { return false }
            // Only allow the next greater card of the same suit.
            if let last = lastCard(for: destination) {
                return value.rank.rawValue == last.value.rank.rawValue + 1
            }
            // If it's empty, only accept the ace.
            return value.rank == .ace
        }
    }

    func validateMove(
        session: DropSession,
        destination: ReorderDifference<CardValue, Card.Group>.Destination
    ) -> Bool {
        if let draggedItemIDs = session.localSession?.draggedItemIDs(
            for: CardValue.self),
           let first = draggedItemIDs.first,
           validateMove(value: first, destination: destination.collectionID) {
            return true
        }
        return false
    }
}
