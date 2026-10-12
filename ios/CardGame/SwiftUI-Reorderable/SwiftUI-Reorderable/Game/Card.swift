/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A model class that expresses the state of a single playing card in a game,
 including its value, whether it is face up, and its position within the
 game.
*/

import Foundation
import CoreTransferable
import UniformTypeIdentifiers
import SwiftData

@Model
final class Card {
    private(set) var rank: Int
    private(set) var suit: Int
    var isFaceDown: Bool

    /// The game instance that this card belongs to.
    var game: Game
    /// The group (pile, remainder, destination) that this card belongs to.
    private(set) var group: Int
    /// The sub-group index this card belongs to (pile 0, pile 1, etc.).
    private(set) var groupIndex: Int
    /// The position within the subgroup.
    var position: Int

    #Unique<Card>([\.game, \.group, \.groupIndex, \.position])

    init(
        value: CardValue, isFaceDown: Bool, game: Game, group: Group,
        position: Int
    ) {
        rank = value.rank.rawValue
        suit = value.suit.rawValue
        self.isFaceDown = isFaceDown
        self.game = game
        self.group = group.intValue
        groupIndex = group.groupIndex
        self.position = position
    }
}

extension Card {
    /// An identifier type that represents which group the card belongs to in a
    /// game.
    nonisolated enum Group: Hashable, Sendable {
        case pile(Int)
        case remainder
        case destination(CardValue.Suit)

        var intValue: Int {
            switch self {
            case .pile: 0
            case .remainder: 1
            case .destination: 2
            }
        }

        var groupIndex: Int {
            switch self {
            case let .pile(index): index
            case .remainder: 0
            case let .destination(suit): suit.rawValue
            }
        }

        init(value: Int, groupIndex: Int) {
            self = switch value {
            case 0: .pile(groupIndex)
            case 2: .destination(CardValue.Suit(rawValue: groupIndex) ?? .clubs)
            default: .remainder
            }
        }
    }
}

extension Card {
    /// A computed property that provides safe reads and writes to the stored
    /// values for rank and suit.
    var value: CardValue {
        get {
            CardValue(
                rank: .init(rawValue: rank)!, suit: .init(rawValue: suit)!)
        }
        set {
            rank = newValue.rank.rawValue
            suit = newValue.suit.rawValue
        }
    }

    /// A computed property that provides safe reads and writes to the stored
    /// values for group and groupIndex.
    var groupID: Group {
        get { Group(value: group, groupIndex: groupIndex) }
        set {
            group = newValue.intValue
            groupIndex = newValue.groupIndex
        }
    }
}
