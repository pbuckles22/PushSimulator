import Foundation

/// Suit the card face draws. `none` has no pip suit.
public enum BoardSuit: Equatable, Codable {
    case hearts
    case diamonds
    case clubs
    case spades
    case none
}

/// Rank the card face draws.
public enum BoardRank: Equatable, Codable {
    case two
    case three
    case four
    case five
    case six
    case seven
    case eight
    case nine
    case ten
    case jack
    case queen
    case king
    case ace
    case joker
}

/// One card the board lays out. `id` keeps two copies of the same face apart.
public struct BoardCard: Equatable, Identifiable, Codable {
    public let id: UInt32
    public let suit: BoardSuit
    public let rank: BoardRank
    /// A value above 0 draws the lock mark. The view does not compare it to a turn.
    public let lockedUntilTurn: UInt32

    public init(id: UInt32, suit: BoardSuit, rank: BoardRank, lockedUntilTurn: UInt32) {
        self.id = id
        self.suit = suit
        self.rank = rank
        self.lockedUntilTurn = lockedUntilTurn
    }
}

/// The table the published object holds. The board is melds in seat order.
/// `hand` is the local seat. Opponent hands stay off this picture.
public struct TablePicture: Equatable {
    public var version: String
    public var deckSize: UInt32
    public var roundNumber: UInt32
    public var board: [[BoardCard]]
    public var hand: [BoardCard]

    public init(
        version: String,
        deckSize: UInt32,
        roundNumber: UInt32,
        board: [[BoardCard]],
        hand: [BoardCard] = []
    ) {
        self.version = version
        self.deckSize = deckSize
        self.roundNumber = roundNumber
        self.board = board
        self.hand = hand
    }
}

/// The picture shows the local seat. Opponent hands are not copied.
public func handOnPicture(local: [BoardCard], opponents: [[BoardCard]]) -> [BoardCard] {
    _ = opponents
    return local
}
