import Foundation

/// What a card shows. Hearts and diamonds are red. A joker has no suit.
public struct CardFace: Equatable {
    public let pip: String
    public let suitSymbol: String
    public let suitName: String
    public let isRed: Bool
    public let isLocked: Bool
    public let spoken: String

    public init(_ card: BoardCard) {
        isLocked = card.lockedUntilTurn > 0
        if card.rank == .joker {
            pip = "Joker"
            suitSymbol = ""
            suitName = ""
            isRed = false
            spoken = Self.spoken(base: "Joker", isLocked: isLocked)
            return
        }

        pip = Self.pip(card.rank)
        let suit = Self.suit(card.suit)
        suitSymbol = suit.symbol
        suitName = suit.name
        isRed = suit.isRed
        let rankName = Self.rankName(card.rank)
        let base = suit.name.isEmpty ? rankName : "\(rankName) of \(suit.name)"
        spoken = Self.spoken(base: base, isLocked: isLocked)
    }

    private static func spoken(base: String, isLocked: Bool) -> String {
        isLocked ? "\(base), locked" : base
    }

    private static func pip(_ rank: BoardRank) -> String {
        switch rank {
        case .two: return "2"
        case .three: return "3"
        case .four: return "4"
        case .five: return "5"
        case .six: return "6"
        case .seven: return "7"
        case .eight: return "8"
        case .nine: return "9"
        case .ten: return "10"
        case .jack: return "J"
        case .queen: return "Q"
        case .king: return "K"
        case .ace: return "A"
        case .joker: return "Joker"
        }
    }

    private static func rankName(_ rank: BoardRank) -> String {
        switch rank {
        case .two: return "Two"
        case .three: return "Three"
        case .four: return "Four"
        case .five: return "Five"
        case .six: return "Six"
        case .seven: return "Seven"
        case .eight: return "Eight"
        case .nine: return "Nine"
        case .ten: return "Ten"
        case .jack: return "Jack"
        case .queen: return "Queen"
        case .king: return "King"
        case .ace: return "Ace"
        case .joker: return "Joker"
        }
    }

    private static func suit(_ suit: BoardSuit) -> (symbol: String, name: String, isRed: Bool) {
        switch suit {
        case .hearts: return ("♥", "Hearts", true)
        case .diamonds: return ("♦", "Diamonds", true)
        case .clubs: return ("♣", "Clubs", false)
        case .spades: return ("♠", "Spades", false)
        case .none: return ("", "", false)
        }
    }
}
