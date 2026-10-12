import PushFfi

/// The discard top, drawn with the face view that rank already uses.
/// The last card is the top. Cards under it stay off the picture.
public enum DiscardFace: Equatable {
    case natural(NaturalFace)
    case joker(JokerFace)

    public var spoken: String {
        switch self {
        case .natural(let face): face.spoken
        case .joker(let face): face.spoken
        }
    }

    public var systemImageName: String {
        switch self {
        case .natural(let face): face.systemImageName
        case .joker(let face): face.systemImageName
        }
    }

    public var isRed: Bool {
        switch self {
        case .natural(let face): face.isRed
        case .joker(let face): face.isRed
        }
    }

    public var rankSymbol: String {
        switch self {
        case .natural(let face): face.rankSymbol
        case .joker(let face): face.rankSymbol
        }
    }

    public var suitSymbol: String {
        switch self {
        case .natural(let face): face.suitSymbol
        case .joker(let face): face.suitSymbol
        }
    }

    /// Their face for this card. A missing suit that is not a joker draws nothing.
    public static func draw(_ card: CardSnapshot) -> DiscardFace? {
        if let joker = JokerFace.draw(card) {
            return .joker(joker)
        }
        if let natural = NaturalFace.draw(card) {
            return .natural(natural)
        }
        return nil
    }

    /// The last card. An empty pile, or a top that has no face, draws nothing.
    public static func drawTop(_ cards: [CardSnapshot]) -> DiscardFace? {
        guard let top = cards.last else { return nil }
        return draw(top)
    }

    /// The discard top on this table. A fresh game has none.
    public static func shown(in game: Game) -> DiscardFace? {
        guard let card = game.tableSnapshot().discardTop else { return nil }
        return draw(card)
    }

    /// The discard top the screen draws: the exhibit queen of diamonds.
    public static func shown() -> DiscardFace? {
        shown(in: Game.exhibitSetAndRun())
    }

    /// A wild top still uses the existing mark. The queen on the exhibit has none.
    public static func shownMark(in game: Game) -> WildMark? {
        guard let card = game.tableSnapshot().discardTop else { return nil }
        return WildMark.draw(card)
    }

    public static func shownMark() -> WildMark? {
        shownMark(in: Game.exhibitSetAndRun())
    }
}
