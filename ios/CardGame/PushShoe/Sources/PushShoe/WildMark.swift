import PushFfi

/// The word on a two or a joker. The face underneath stays theirs. A lock is a later story.
public struct WildMark: Equatable {
    public let spoken: String

    /// A two or a joker. Any other rank draws nothing.
    public static func draw(_ card: CardSnapshot) -> WildMark? {
        switch card.rank {
        case .two, .joker:
            return WildMark(spoken: "wild")
        default:
            return nil
        }
    }

    /// The exhibit hand's two. A fresh game has none.
    public static func shownTwo() -> WildMark? {
        shownTwo(in: Game.exhibitSetAndRun())
    }

    public static func shownTwo(in game: Game) -> WildMark? {
        mark(in: game, rank: .two)
    }

    /// The exhibit hand's joker. A fresh game has none.
    public static func shownJoker() -> WildMark? {
        shownJoker(in: Game.exhibitSetAndRun())
    }

    public static func shownJoker(in game: Game) -> WildMark? {
        mark(in: game, rank: .joker)
    }

    private static func mark(in game: Game, rank: CardRank) -> WildMark? {
        let table = game.tableSnapshot()
        let cards = table.hand + table.board.flatMap(\.cards)
        guard let card = cards.first(where: { $0.rank == rank }) else {
            return nil
        }
        return draw(card)
    }
}
