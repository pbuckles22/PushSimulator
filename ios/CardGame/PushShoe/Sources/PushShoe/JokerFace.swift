import PushFfi

/// A joker, using the same corner-and-center slots their face view already draws.
/// A suit on the card is ignored. A lock is a later story.
public struct JokerFace: Equatable {
    public let rankSymbol: String
    public let suitSymbol: String
    public let systemImageName: String
    public let isRed: Bool
    public let spoken: String

    /// The first joker on the table. A fresh game has none.
    public static func one(in game: Game) -> JokerFace? {
        let table = game.tableSnapshot()
        let cards = table.hand + table.board.flatMap(\.cards)
        guard let card = cards.first(where: { $0.rank == .joker }) else {
            return nil
        }
        return draw(card)
    }

    /// The joker face the screen draws: the exhibit hand's joker.
    public static func shown() -> JokerFace? {
        one(in: Game.exhibitSetAndRun())
    }

    /// Their face for a joker. A natural rank and suit draws nothing.
    public static func draw(_ card: CardSnapshot) -> JokerFace? {
        guard card.rank == .joker else { return nil }
        return JokerFace(
            rankSymbol: "Joker",
            suitSymbol: "",
            systemImageName: "star.fill",
            isRed: false,
            spoken: "Joker"
        )
    }
}
