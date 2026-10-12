import PushFfi

/// Seat 0's hand, in the order Rust holds those cards.
/// A card with no face is left out. This layout does not lift into the engine.
public struct LocalHand: Equatable {
    public struct Slot: Equatable {
        public let id: UInt32
        public let spoken: String
        public let wild: WildMark?
        public let face: Face
    }

    public enum Face: Equatable {
        case natural(NaturalFace)
        case joker(JokerFace)
    }

    public let slots: [Slot]
    /// A lift is a later story. This hand does not call the engine.
    public let allowsLift: Bool

    /// Their faces, in the order the cards were given.
    public static func layout(_ cards: [CardSnapshot]) -> LocalHand {
        LocalHand(slots: cards.compactMap(slot), allowsLift: false)
    }

    /// Seat 0's hand on this table. A fresh game has none.
    public static func shown(in game: Game) -> LocalHand {
        layout(game.tableSnapshot().hand)
    }

    /// The hand the screen draws: the exhibit hand.
    public static func shown() -> LocalHand {
        shown(in: Game.exhibitSetAndRun())
    }

    /// Leaves the table as it was. The card id is not sent to Rust.
    public static func lift(cardId _: UInt32, in game: Game) -> LocalHand {
        shown(in: game)
    }

    private static func slot(_ card: CardSnapshot) -> Slot? {
        if let joker = JokerFace.draw(card) {
            return Slot(
                id: card.id,
                spoken: joker.spoken,
                wild: WildMark.draw(card),
                face: .joker(joker)
            )
        }
        if let natural = NaturalFace.draw(card) {
            return Slot(
                id: card.id,
                spoken: natural.spoken,
                wild: WildMark.draw(card),
                face: .natural(natural)
            )
        }
        return nil
    }
}
