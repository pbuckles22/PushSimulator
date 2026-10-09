import Combine
import XCTest
@testable import PushUI

final class HandRowTests: XCTestCase {
    func testHandOnPictureKeepsTheLocalSeatAndDropsOpponents() {
        let local = [
            card(id: 21, suit: .diamonds, rank: .eight, locked: 0),
            card(id: 22, suit: .spades, rank: .king, locked: 0),
            card(id: 23, suit: .none, rank: .joker, locked: 2),
            card(id: 24, suit: .clubs, rank: .two, locked: 0),
        ]
        let opponents = [
            [card(id: 90, suit: .hearts, rank: .ace, locked: 0)],
            [card(id: 91, suit: .clubs, rank: .ace, locked: 0)],
        ]
        let shown = handOnPicture(local: local, opponents: opponents)
        XCTAssertEqual(shown.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(CardFace(shown[0]).spoken, "Eight of Diamonds")
        XCTAssertEqual(CardFace(shown[2]).spoken, "Joker, locked")
        XCTAssertTrue(shown.allSatisfy { card in
            !opponents.flatMap { $0 }.map(\.id).contains(card.id)
        })

        let sameFace = handOnPicture(
            local: [card(id: 21, suit: .hearts, rank: .ace, locked: 0)],
            opponents: [[card(id: 90, suit: .hearts, rank: .ace, locked: 0)]]
        )
        XCTAssertEqual(sameFace.map(\.id), [21])

        let sharedId = handOnPicture(
            local: [card(id: 21, suit: .diamonds, rank: .eight, locked: 0)],
            opponents: [[card(id: 21, suit: .spades, rank: .king, locked: 0)]]
        )
        XCTAssertEqual(sharedId, [card(id: 21, suit: .diamonds, rank: .eight, locked: 0)])

        XCTAssertTrue(handOnPicture(local: [], opponents: opponents).isEmpty)
        XCTAssertEqual(
            handOnPicture(local: [local[1], local[0]], opponents: []).map(\.id),
            [22, 21]
        )
    }

    func testHandRowShowsOrderLocksJokersAndLiftsACard() {
        let empty = HandView(cards: [])
        XCTAssertEqual(empty.emptyTitle, "No cards")
        XCTAssertTrue(empty.layout.faces.isEmpty)
        XCTAssertFalse(empty.allowsDrag)

        let cards = [
            card(id: 21, suit: .diamonds, rank: .eight, locked: 0),
            card(id: 22, suit: .spades, rank: .king, locked: 0),
            card(id: 23, suit: .none, rank: .joker, locked: 2),
            card(id: 24, suit: .clubs, rank: .two, locked: 0),
        ]
        let hand = HandView(cards: cards)
        XCTAssertNil(hand.emptyTitle)
        XCTAssertTrue(hand.allowsDrag)
        XCTAssertTrue(cards.allSatisfy { hand.canLift($0) })
        XCTAssertEqual(hand.layout.faces.map(\.spoken), [
            "Eight of Diamonds",
            "King of Spades",
            "Joker, locked",
            "Two of Clubs",
        ])
        XCTAssertEqual(hand.layout.faces.map(\.isRed), [true, false, false, false])
        XCTAssertEqual(CardView(card: cards[0]).face, hand.layout.faces[0])
        XCTAssertEqual(CardView(card: cards[2]).face.spoken, "Joker, locked")

        let twins = HandView(cards: [
            card(id: 7, suit: .hearts, rank: .four, locked: 0),
            card(id: 7, suit: .spades, rank: .five, locked: 0),
        ])
        XCTAssertEqual(twins.layout.faces.map(\.pip), ["4", "5"])
        XCTAssertEqual(twins.layout.faces.map(\.suitSymbol), ["♥", "♠"])
        XCTAssertTrue(twins.allowsDrag)
        XCTAssertTrue(twins.canLift(twins.cards[0]))
    }

    /// Chain: empty published shoe → board rows → local hand row.
    /// An opponent card is not on the picture. A card in the hand can lift.
    func testPublishedShoeBoardRowsThenLocalHandOmitsTheOpponentAndLifts() {
        let set = [
            card(id: 1, suit: .hearts, rank: .four, locked: 0),
            card(id: 2, suit: .spades, rank: .four, locked: 0),
            card(id: 3, suit: .clubs, rank: .four, locked: 0),
        ]
        let run = [
            card(id: 10, suit: .hearts, rank: .four, locked: 0),
            card(id: 11, suit: .hearts, rank: .five, locked: 0),
            card(id: 12, suit: .hearts, rank: .six, locked: 0),
            card(id: 13, suit: .hearts, rank: .seven, locked: 0),
        ]
        let local = [
            card(id: 21, suit: .diamonds, rank: .eight, locked: 0),
            card(id: 22, suit: .spades, rank: .king, locked: 0),
            card(id: 23, suit: .none, rank: .joker, locked: 2),
            card(id: 24, suit: .clubs, rank: .two, locked: 0),
        ]
        let opponent = card(id: 90, suit: .hearts, rank: .ace, locked: 0)
        let pictured = handOnPicture(local: local, opponents: [[opponent]])

        var loads = 0
        let table = PublishedTable {
            loads += 1
            if loads == 1 {
                return TablePicture(
                    version: "0.3.3",
                    deckSize: 108,
                    roundNumber: 1,
                    board: [],
                    hand: []
                )
            }
            return TablePicture(
                version: "0.3.3",
                deckSize: 108,
                roundNumber: 2,
                board: [set, run],
                hand: pictured
            )
        }
        let screen = GameBoardScreen(table: table)
        XCTAssertEqual(loads, 1)
        XCTAssertEqual(screen.versionLine, "0.3.3")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 1")
        XCTAssertEqual(screen.board.emptyTitle, "No melds")
        XCTAssertEqual(screen.hand.emptyTitle, "No cards")
        XCTAssertFalse(screen.hand.allowsDrag)
        XCTAssertTrue(screen.hand.cards.isEmpty)

        var signals = 0
        let sub = table.objectWillChange.sink { signals += 1 }
        table.reload()
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(loads, 2)
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.versionLine, "0.3.3")
        XCTAssertNil(screen.board.emptyTitle)
        XCTAssertNil(screen.hand.emptyTitle)
        XCTAssertTrue(screen.hand.allowsDrag)
        XCTAssertTrue(screen.hand.cards.allSatisfy { screen.hand.canLift($0) })
        XCTAssertFalse(screen.hand.canLift(opponent))
        XCTAssertEqual(screen.board.layout.rows.count, 2)
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.suitSymbol), ["♥", "♠", "♣"])
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])
        XCTAssertEqual(screen.hand.cards.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(screen.hand.layout.faces.map(\.spoken), [
            "Eight of Diamonds",
            "King of Spades",
            "Joker, locked",
            "Two of Clubs",
        ])
        XCTAssertEqual(CardView(card: local[0]).face, screen.hand.layout.faces[0])
        XCTAssertEqual(CardView(card: set[0]).face, screen.board.layout.rows[0].faces[0])
        XCTAssertFalse(screen.hand.cards.map(\.id).contains(opponent.id))
        XCTAssertFalse(screen.board.melds.flatMap { $0 }.map(\.id).contains(opponent.id))
        let handIds = Set(screen.hand.cards.map(\.id))
        let boardIds = Set(screen.board.melds.flatMap { $0 }.map(\.id))
        XCTAssertTrue(handIds.isDisjoint(with: boardIds))
        sub.cancel()
    }
}

private func card(id: UInt32, suit: BoardSuit, rank: BoardRank, locked: UInt32) -> BoardCard {
    BoardCard(id: id, suit: suit, rank: rank, lockedUntilTurn: locked)
}
