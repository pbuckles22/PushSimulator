import PushFfi

/// Seat 1's cards, drawn as backs. The digits are how many that seat holds.
/// The cards themselves stay off this value.
public struct OpponentBacks: Equatable {
    public struct Back: Equatable {
        public let index: Int
    }

    public let backs: [Back]
    public let countText: String

    /// One back per card, in order. The text is the digits of `count`.
    public static func layout(count: UInt32) -> OpponentBacks {
        OpponentBacks(
            backs: (0..<Int(count)).map { Back(index: $0) },
            countText: String(count)
        )
    }

    /// Seat 1 on this table. A fresh game has none.
    public static func shown(in game: Game) -> OpponentBacks {
        layout(count: game.tableSnapshot().opponentCount)
    }

    /// The backs the screen draws: the exhibit's other seat.
    public static func shown() -> OpponentBacks {
        shown(in: Game.exhibitSetAndRun())
    }
}
