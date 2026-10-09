import PushUI

/// The picture SwiftUI publishes. The shoe, the round, the melds, and the local hand come from Rust.
func livePicture(from game: Game) -> TablePicture {
    picture(from: game.tableSnapshot(), version: game.publicVersion())
}

func picture(from snapshot: TableSnapshot, version: String) -> TablePicture {
    TablePicture(
        version: version,
        deckSize: snapshot.deckSize,
        roundNumber: snapshot.roundNumber,
        board: snapshot.board.map { meld in
            meld.cards.map(boardCard)
        },
        hand: snapshot.hand.map(boardCard)
    )
}

private func boardCard(_ card: CardSnapshot) -> BoardCard {
    BoardCard(
        id: card.id,
        suit: boardSuit(card.suit),
        rank: boardRank(card.rank),
        lockedUntilTurn: card.lockedUntilTurn
    )
}

private func boardSuit(_ suit: CardSuit) -> BoardSuit {
    switch suit {
    case .hearts: return .hearts
    case .diamonds: return .diamonds
    case .clubs: return .clubs
    case .spades: return .spades
    case .none: return .none
    }
}

private func boardRank(_ rank: CardRank) -> BoardRank {
    switch rank {
    case .two: return .two
    case .three: return .three
    case .four: return .four
    case .five: return .five
    case .six: return .six
    case .seven: return .seven
    case .eight: return .eight
    case .nine: return .nine
    case .ten: return .ten
    case .jack: return .jack
    case .queen: return .queen
    case .king: return .king
    case .ace: return .ace
    case .joker: return .joker
    }
}
