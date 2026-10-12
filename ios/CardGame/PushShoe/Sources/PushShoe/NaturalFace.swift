import PushFfi

/// One natural rank and suit, using the symbols their face view already draws.
public struct NaturalFace: Equatable {
    public let rankSymbol: String
    public let suitSymbol: String
    public let systemImageName: String
    public let isRed: Bool
    /// `CardValue.Rank` raw value, so their face view can take this card.
    public let rankRaw: Int
    /// `CardValue.Suit` raw value, so their face view can take this card.
    public let suitRaw: Int

    public var spoken: String { rankSymbol + suitSymbol }

    /// The first board card when it is a natural rank and suit. A fresh game has none.
    public static func one(in game: Game) -> NaturalFace? {
        guard let card = game.tableSnapshot().board.first?.cards.first else {
            return nil
        }
        return draw(card)
    }

    /// The one face the screen draws: the exhibit's first natural card.
    public static func shown() -> NaturalFace? {
        one(in: Game.exhibitSetAndRun())
    }

    /// Their face for a natural rank and suit. A joker or a missing suit draws nothing.
    public static func draw(_ card: CardSnapshot) -> NaturalFace? {
        guard card.rank != .joker, card.suit != .none,
              let rank = rankFace(card.rank),
              let suit = suitFace(card.suit) else {
            return nil
        }
        return NaturalFace(
            rankSymbol: rank.symbol,
            suitSymbol: suit.symbol,
            systemImageName: suit.image,
            isRed: suit.isRed,
            rankRaw: rank.raw,
            suitRaw: suit.raw
        )
    }
}

private struct RankFace {
    let symbol: String
    let raw: Int
}

private struct SuitFace {
    let symbol: String
    let image: String
    let isRed: Bool
    let raw: Int
}

private func rankFace(_ rank: CardRank) -> RankFace? {
    switch rank {
    case .two: RankFace(symbol: "2", raw: 2)
    case .three: RankFace(symbol: "3", raw: 3)
    case .four: RankFace(symbol: "4", raw: 4)
    case .five: RankFace(symbol: "5", raw: 5)
    case .six: RankFace(symbol: "6", raw: 6)
    case .seven: RankFace(symbol: "7", raw: 7)
    case .eight: RankFace(symbol: "8", raw: 8)
    case .nine: RankFace(symbol: "9", raw: 9)
    case .ten: RankFace(symbol: "10", raw: 10)
    case .jack: RankFace(symbol: "J", raw: 11)
    case .queen: RankFace(symbol: "Q", raw: 12)
    case .king: RankFace(symbol: "K", raw: 13)
    case .ace: RankFace(symbol: "A", raw: 1)
    case .joker: nil
    }
}

private func suitFace(_ suit: CardSuit) -> SuitFace? {
    switch suit {
    case .clubs:
        SuitFace(symbol: "♣️", image: "suit.club.fill", isRed: false, raw: 0)
    case .diamonds:
        SuitFace(symbol: "♦️", image: "suit.diamond.fill", isRed: true, raw: 1)
    case .hearts:
        SuitFace(symbol: "♥️", image: "suit.heart.fill", isRed: true, raw: 2)
    case .spades:
        SuitFace(symbol: "♠️", image: "suit.spade.fill", isRed: false, raw: 3)
    case .none:
        nil
    }
}
