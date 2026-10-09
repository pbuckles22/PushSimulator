import Combine
import PushUI

/// Chain: undealt shoe → published picture → card face → board rows.
@main
enum BoardUICheck {
    static func main() {
        let game = Game()
        precondition(game.getDeckSize() == 108, "shoe \(game.getDeckSize())")
        precondition(game.publicVersion() == "0.3.2", "version \(game.publicVersion())")
        let opened = game.tableSnapshot()
        precondition(opened.deckSize == game.getDeckSize(), "snapshot shoe \(opened.deckSize)")
        precondition(opened.roundNumber == 1, "round \(opened.roundNumber)")
        precondition(opened.board.isEmpty, "a new game has melds")

        let table = PublishedTable { livePicture(from: game) }
        let screen = GameBoardScreen(table: table)
        precondition(screen.versionLine == "0.3.2", screen.versionLine)
        precondition(screen.deckLine == "Deck size 108", screen.deckLine)
        precondition(screen.roundLine == "Round 1", screen.roundLine)
        precondition(screen.board.emptyTitle == "No melds", "empty title")
        precondition(screen.board.layout.rows.isEmpty, "empty rows")
        precondition(table.picture.deckSize == game.getDeckSize(), "published shoe")
        precondition(game.getDeckSize() == 108, "shoe changed")

        var signals = 0
        let held = table.objectWillChange.sink { signals += 1 }
        table.reload()
        precondition(signals == 0, "unchanged table published \(signals)")
        precondition(screen.deckLine == "Deck size 108", screen.deckLine)
        held.cancel()

        let exhibit = Game.exhibitSetAndRun()
        precondition(exhibit.getDeckSize() == 108, "exhibit shoe \(exhibit.getDeckSize())")
        precondition(exhibit.publicVersion() == game.publicVersion(), "version drift")
        let shown = PublishedTable { livePicture(from: exhibit) }
        let boardScreen = GameBoardScreen(table: shown)
        precondition(boardScreen.deckLine == "Deck size 108", boardScreen.deckLine)
        precondition(boardScreen.roundLine == "Round 2", boardScreen.roundLine)
        precondition(boardScreen.versionLine == "0.3.2", boardScreen.versionLine)
        precondition(boardScreen.board.emptyTitle == nil, "exhibit looked empty")
        let rows = boardScreen.board.layout.rows
        precondition(rows.count == 2, "rows \(rows.count)")
        precondition(rows[0].faces.map(\.pip) == ["4", "4", "4"], "set pips")
        precondition(rows[0].faces.map(\.suitSymbol) == ["♥", "♠", "♣"], "set suits")
        precondition(rows[0].faces.map(\.isRed) == [true, false, false], "set color")
        precondition(rows[0].faces[0].spoken == "Four of Hearts", rows[0].faces[0].spoken)
        precondition(rows[0].faces[1].spoken == "Four of Spades", rows[0].faces[1].spoken)
        precondition(rows[0].faces[2].spoken == "Four of Clubs", rows[0].faces[2].spoken)
        precondition(rows[1].faces.map(\.pip) == ["4", "5", "6", "7"], "run pips")
        precondition(
            rows[1].faces.allSatisfy { $0.isRed && $0.suitName == "Hearts" && !$0.isLocked },
            "run faces"
        )
        precondition(rows[1].faces[3].spoken == "Seven of Hearts", rows[1].faces[3].spoken)
        let first = shown.picture.board[0][0]
        precondition(first.id == 1, "id \(first.id)")
        precondition(shown.picture.board[1][3].id == 13, "run id")
        precondition(CardView(card: first).face == rows[0].faces[0], "card view face")
        precondition(game.tableSnapshot().board.isEmpty, "exhibit wrote the live game")
        precondition(screen.roundLine == "Round 1", "live screen changed")

        let mixed = TableSnapshot(
            deckSize: 0,
            roundNumber: 6,
            board: [
                MeldSnapshot(cards: [
                    CardSnapshot(id: 10, suit: .hearts, rank: .four, lockedUntilTurn: 0),
                    CardSnapshot(id: 20, suit: .hearts, rank: .joker, lockedUntilTurn: 3),
                    CardSnapshot(id: 12, suit: .hearts, rank: .six, lockedUntilTurn: 0),
                ]),
                MeldSnapshot(cards: []),
            ]
        )
        let mixedPicture = picture(from: mixed, version: "0.3.2")
        let mixedBoard = BoardView(melds: mixedPicture.board)
        precondition(mixedPicture.deckSize == 0, "mapped shoe")
        precondition(mixedPicture.roundNumber == 6, "mapped round")
        precondition(mixedBoard.emptyTitle == nil, "empty meld hid the board")
        precondition(mixedBoard.layout.rows.count == 2, "mixed rows")
        precondition(mixedBoard.layout.rows[1].faces.isEmpty, "empty meld dropped")
        precondition(mixedBoard.layout.rows[0].faces.map(\.pip) == ["4", "Joker", "6"], "joker moved")
        precondition(mixedBoard.layout.rows[0].faces[1].spoken == "Joker, locked", "lock")
        precondition(!mixedBoard.layout.rows[0].faces[1].isRed, "joker turned red")
        precondition(CardView(card: mixedPicture.board[0][1]).face.spoken == "Joker, locked", "card")
        print("board-ui ok")
    }
}
