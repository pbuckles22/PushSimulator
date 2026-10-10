import Combine
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

/// The exhibit the screen shows and the game a drop mutates. They are one object,
/// so a later app init cannot point the drop at a different table.
final class LiveExhibit: ObservableObject {
    let game: Game
    let table: PublishedTable

    init(game: Game) {
        self.game = game
        table = PublishedTable { livePicture(from: game) }
    }

    func answer(_ intent: MoveIntent) -> DropAnswer {
        engineAnswer(game: game, intent: intent)
    }
}

/// Asks Rust to apply one `PlayMeld` or one `HitMeld`.
/// An accept is the table Rust returned. A game error or a refusal is not.
func engineAnswer(game: Game, intent: MoveIntent) -> DropAnswer {
    let verdict: DropVerdict
    switch intent {
    case .playMeld(let groups):
        verdict = game.applyPlayMeld(groups: groups.map { group in
            group.map(cardSnapshot)
        })
    case .hitMeld(let hits):
        verdict = game.applyHitMeld(hits: hits.map { hit in
            HitRequest(
                meldIndex: UInt32(hit.meldIndex),
                cards: hit.cards.map(cardSnapshot)
            )
        })
    }
    switch verdict {
    case .accepted:
        let picture = livePicture(from: game)
        return .accepted(HeldCards(hand: picture.hand, board: picture.board))
    case .refused:
        return .refused
    case .gameError:
        return .gameError
    }
}

private func cardSnapshot(_ card: BoardCard) -> CardSnapshot {
    CardSnapshot(
        id: card.id,
        suit: cardSuit(card.suit),
        rank: cardRank(card.rank),
        lockedUntilTurn: card.lockedUntilTurn
    )
}

private func cardSuit(_ suit: BoardSuit) -> CardSuit {
    switch suit {
    case .hearts: return .hearts
    case .diamonds: return .diamonds
    case .clubs: return .clubs
    case .spades: return .spades
    case .none: return .none
    }
}

private func cardRank(_ rank: BoardRank) -> CardRank {
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
