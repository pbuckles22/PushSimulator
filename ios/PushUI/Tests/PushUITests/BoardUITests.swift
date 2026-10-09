import Combine
import XCTest
@testable import PushUI

final class BoardUITests: XCTestCase {
    func testEveryRankAndSuitPicksTheFace() {
        let ranks: [(BoardRank, String, String)] = [
            (.two, "2", "Two"),
            (.three, "3", "Three"),
            (.four, "4", "Four"),
            (.five, "5", "Five"),
            (.six, "6", "Six"),
            (.seven, "7", "Seven"),
            (.eight, "8", "Eight"),
            (.nine, "9", "Nine"),
            (.ten, "10", "Ten"),
            (.jack, "J", "Jack"),
            (.queen, "Q", "Queen"),
            (.king, "K", "King"),
            (.ace, "A", "Ace"),
        ]
        for (rank, pip, name) in ranks {
            let face = CardFace(card(id: 1, suit: .hearts, rank: rank, locked: 0))
            XCTAssertEqual(face.pip, pip)
            XCTAssertEqual(face.suitSymbol, "♥")
            XCTAssertEqual(face.suitName, "Hearts")
            XCTAssertTrue(face.isRed)
            XCTAssertFalse(face.isLocked)
            XCTAssertEqual(face.spoken, "\(name) of Hearts")
        }

        let suits: [(BoardSuit, String, String, Bool)] = [
            (.hearts, "♥", "Hearts", true),
            (.diamonds, "♦", "Diamonds", true),
            (.clubs, "♣", "Clubs", false),
            (.spades, "♠", "Spades", false),
        ]
        for (suit, symbol, name, isRed) in suits {
            let face = CardFace(card(id: 2, suit: suit, rank: .ace, locked: 0))
            XCTAssertEqual(face.suitSymbol, symbol)
            XCTAssertEqual(face.suitName, name)
            XCTAssertEqual(face.isRed, isRed)
            XCTAssertEqual(face.spoken, "Ace of \(name)")
        }
    }

    func testJokerIgnoresSuitAndALockedCardKeepsItsPip() {
        let joker = CardFace(card(id: 9, suit: .hearts, rank: .joker, locked: 0))
        XCTAssertEqual(joker.pip, "Joker")
        XCTAssertEqual(joker.suitSymbol, "")
        XCTAssertEqual(joker.suitName, "")
        XCTAssertFalse(joker.isRed)
        XCTAssertFalse(joker.isLocked)
        XCTAssertEqual(joker.spoken, "Joker")

        let plain = CardFace(card(id: 4, suit: .none, rank: .ace, locked: 0))
        XCTAssertEqual(plain.pip, "A")
        XCTAssertEqual(plain.suitSymbol, "")
        XCTAssertEqual(plain.suitName, "")
        XCTAssertFalse(plain.isRed)
        XCTAssertEqual(plain.spoken, "Ace")

        let locked = CardFace(card(id: 5, suit: .diamonds, rank: .queen, locked: 4))
        XCTAssertEqual(locked.pip, "Q")
        XCTAssertEqual(locked.suitSymbol, "♦")
        XCTAssertTrue(locked.isRed)
        XCTAssertTrue(locked.isLocked)
        XCTAssertEqual(locked.spoken, "Queen of Diamonds, locked")

        let lockedJoker = CardFace(card(id: 6, suit: .none, rank: .joker, locked: 1))
        XCTAssertEqual(lockedJoker.spoken, "Joker, locked")
        XCTAssertTrue(lockedJoker.isLocked)
        XCTAssertFalse(lockedJoker.isRed)
    }

    func testCardViewUsesThatFace() {
        let queen = card(id: 8, suit: .spades, rank: .queen, locked: 2)
        let view = CardView(card: queen)
        XCTAssertEqual(view.face, CardFace(queen))
        XCTAssertEqual(view.face.pip, "Q")
        XCTAssertEqual(view.face.suitSymbol, "♠")
        XCTAssertFalse(view.face.isRed)
        XCTAssertEqual(view.face.spoken, "Queen of Spades, locked")

        let other = CardView(card: card(id: 8, suit: .hearts, rank: .queen, locked: 0))
        XCTAssertNotEqual(other.face, view.face)
        XCTAssertTrue(other.face.isRed)
        XCTAssertEqual(other.face.spoken, "Queen of Hearts")
    }

    func testEmptyBoardAndASetBesideARun() {
        let empty = BoardView(melds: [])
        XCTAssertEqual(empty.emptyTitle, "No melds")
        XCTAssertTrue(empty.layout.rows.isEmpty)

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
        let board = BoardView(melds: [set, run])
        XCTAssertNil(board.emptyTitle)
        XCTAssertEqual(board.layout.rows.count, 2)
        XCTAssertEqual(board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])
        XCTAssertEqual(board.layout.rows[0].faces.map(\.isRed), [true, false, false])
        XCTAssertEqual(board.layout.rows[0].faces.map(\.suitSymbol), ["♥", "♠", "♣"])
        XCTAssertEqual(board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])
        XCTAssertTrue(board.layout.rows[1].faces.allSatisfy { $0.isRed && $0.suitName == "Hearts" })
        XCTAssertEqual(CardView(card: set[0]).face, board.layout.rows[0].faces[0])
    }

    func testBoardKeepsOrderLocksJokersEmptyRowsAndDuplicateIds() {
        let reversed = BoardView(melds: [[
            card(id: 1, suit: .hearts, rank: .seven, locked: 0),
            card(id: 2, suit: .hearts, rank: .six, locked: 0),
            card(id: 3, suit: .hearts, rank: .five, locked: 0),
            card(id: 4, suit: .hearts, rank: .four, locked: 0),
        ]])
        XCTAssertEqual(reversed.layout.rows[0].faces.map(\.pip), ["7", "6", "5", "4"])

        let joker = card(id: 20, suit: .hearts, rank: .joker, locked: 3)
        let mixed = BoardView(melds: [[
            card(id: 10, suit: .hearts, rank: .four, locked: 0),
            joker,
            card(id: 12, suit: .hearts, rank: .six, locked: 0),
        ]])
        XCTAssertEqual(mixed.layout.rows[0].faces.map(\.pip), ["4", "Joker", "6"])
        XCTAssertEqual(mixed.layout.rows[0].faces.map(\.isRed), [true, false, true])
        XCTAssertEqual(mixed.layout.rows[0].faces[1].spoken, "Joker, locked")
        XCTAssertEqual(CardView(card: joker).face, mixed.layout.rows[0].faces[1])

        let long = BoardView(melds: [[
            card(id: 1, suit: .clubs, rank: .three, locked: 0),
            card(id: 2, suit: .clubs, rank: .four, locked: 0),
            card(id: 3, suit: .clubs, rank: .five, locked: 0),
            card(id: 4, suit: .clubs, rank: .six, locked: 0),
            card(id: 5, suit: .clubs, rank: .seven, locked: 0),
            card(id: 6, suit: .clubs, rank: .eight, locked: 0),
            card(id: 7, suit: .clubs, rank: .nine, locked: 0),
            card(id: 8, suit: .clubs, rank: .ten, locked: 0),
        ]])
        XCTAssertEqual(long.layout.rows.count, 1)
        XCTAssertEqual(long.layout.rows[0].faces.map(\.pip), ["3", "4", "5", "6", "7", "8", "9", "10"])
        XCTAssertTrue(long.layout.rows[0].faces.allSatisfy { !$0.isRed && $0.suitName == "Clubs" })

        let one = BoardView(melds: [[card(id: 1, suit: .diamonds, rank: .ace, locked: 0)]])
        XCTAssertNil(one.emptyTitle)
        XCTAssertEqual(one.layout.rows[0].faces.map(\.spoken), ["Ace of Diamonds"])

        let gap = BoardView(melds: [
            [card(id: 1, suit: .spades, rank: .king, locked: 0)],
            [],
        ])
        XCTAssertNil(gap.emptyTitle)
        XCTAssertEqual(gap.layout.rows.count, 2)
        XCTAssertTrue(gap.layout.rows[1].faces.isEmpty)

        let twins = BoardView(melds: [[
            card(id: 7, suit: .hearts, rank: .four, locked: 0),
            card(id: 7, suit: .spades, rank: .five, locked: 0),
        ]])
        XCTAssertEqual(twins.layout.rows[0].faces.map(\.pip), ["4", "5"])
        XCTAssertEqual(twins.layout.rows[0].faces.map(\.suitSymbol), ["♥", "♠"])
    }

    func testPublishedTableStartsFromTheLoadAndPublishesAChange() {
        var loads = 0
        let table = PublishedTable {
            loads += 1
            if loads == 1 {
                return TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 1, board: [])
            }
            return TablePicture(version: "0.3.2", deckSize: 0, roundNumber: 2, board: [[
                card(id: 1, suit: .hearts, rank: .four, locked: 0),
            ]])
        }
        XCTAssertEqual(loads, 1)
        XCTAssertEqual(table.deckSizeText, "Deck size 108")
        XCTAssertEqual(table.picture.roundNumber, 1)
        XCTAssertTrue(table.picture.board.isEmpty)

        var signals = 0
        let same = table.objectWillChange.sink { signals += 1 }
        table.reload()
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(loads, 2)
        XCTAssertEqual(table.deckSizeText, "Deck size 0")
        XCTAssertEqual(table.picture.board.count, 1)
        same.cancel()

        let again = PublishedTable {
            TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 1, board: [])
        }
        var quiet = 0
        let held = again.objectWillChange.sink { quiet += 1 }
        again.reload()
        XCTAssertEqual(quiet, 0)
        XCTAssertEqual(again.picture.deckSize, 108)
        XCTAssertEqual(again.deckSizeText, "Deck size 108")
        held.cancel()
        XCTAssertEqual(table.picture.deckSize, 0)
    }

    func testPublishedTableCardViewAndBoardViewLayTheSetThenTheRun() {
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
        var loads = 0
        let table = PublishedTable {
            loads += 1
            if loads == 1 {
                return TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 1, board: [])
            }
            return TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 2, board: [set, run])
        }
        let screen = GameBoardScreen(table: table)
        XCTAssertEqual(screen.versionLine, "0.3.2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 1")
        XCTAssertEqual(screen.board.emptyTitle, "No melds")
        XCTAssertTrue(screen.board.layout.rows.isEmpty)

        var signals = 0
        let sub = table.objectWillChange.sink { signals += 1 }
        table.reload()
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(screen.versionLine, "0.3.2")
        XCTAssertNil(screen.board.emptyTitle)
        XCTAssertEqual(screen.board.layout.rows.count, 2)
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])
        XCTAssertEqual(
            CardView(card: set[0]).face.spoken,
            screen.board.layout.rows[0].faces[0].spoken
        )
        XCTAssertEqual(screen.board.layout.rows[0].faces[0].spoken, "Four of Hearts")
        sub.cancel()

        let other = PublishedTable {
            TablePicture(version: "9.9.9", deckSize: 1, roundNumber: 6, board: [])
        }
        XCTAssertEqual(GameBoardScreen(table: other).versionLine, "9.9.9")
        XCTAssertEqual(GameBoardScreen(table: other).deckLine, "Deck size 1")
        XCTAssertEqual(GameBoardScreen(table: other).roundLine, "Round 6")
        XCTAssertEqual(screen.versionLine, "0.3.2")
    }

    func testDeckSizeTextUsesThePublishedNumber() {
        let table = PublishedTable {
            TablePicture(version: "0.3.2", deckSize: UInt32.max, roundNumber: 1, board: [])
        }
        XCTAssertEqual(table.deckSizeText, "Deck size \(UInt32.max)")
    }
}

private func card(id: UInt32, suit: BoardSuit, rank: BoardRank, locked: UInt32) -> BoardCard {
    BoardCard(id: id, suit: suit, rank: rank, lockedUntilTurn: locked)
}
