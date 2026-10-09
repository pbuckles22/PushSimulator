import Combine
import PushUI

/// Chain: undealt shoe → published picture → card face → board rows.
@main
enum BoardUICheck {
    static func main() {
        let game = Game()
        precondition(game.getDeckSize() == 108, "shoe \(game.getDeckSize())")
        precondition(game.publicVersion() == "0.3.5", "version \(game.publicVersion())")
        let opened = game.tableSnapshot()
        precondition(opened.deckSize == game.getDeckSize(), "snapshot shoe \(opened.deckSize)")
        precondition(opened.roundNumber == 1, "round \(opened.roundNumber)")
        precondition(opened.board.isEmpty, "a new game has melds")

        let table = PublishedTable { livePicture(from: game) }
        let screen = GameBoardScreen(table: table)
        precondition(screen.versionLine == "0.3.5", screen.versionLine)
        precondition(screen.deckLine == "Deck size 108", screen.deckLine)
        precondition(screen.roundLine == "Round 1", screen.roundLine)
        precondition(screen.board.emptyTitle == "No melds", "empty title")
        precondition(screen.board.layout.rows.isEmpty, "empty rows")
        precondition(screen.hand.emptyTitle == "No cards", "empty hand")
        precondition(screen.hand.allowsDrag == false, "empty hand drags")
        precondition(screen.hand.cards.isEmpty, "empty hand has cards")
        precondition(table.picture.hand.isEmpty, "published hand")
        precondition(table.picture.deckSize == game.getDeckSize(), "published shoe")
        precondition(game.getDeckSize() == 108, "shoe changed")

        var signals = 0
        let unchanged = table.objectWillChange.sink { signals += 1 }
        table.reload()
        precondition(signals == 0, "unchanged table published \(signals)")
        precondition(screen.deckLine == "Deck size 108", screen.deckLine)
        unchanged.cancel()

        let exhibit = Game.exhibitSetAndRun()
        precondition(exhibit.getDeckSize() == 108, "exhibit shoe \(exhibit.getDeckSize())")
        precondition(exhibit.publicVersion() == game.publicVersion(), "version drift")
        let shown = PublishedTable { livePicture(from: exhibit) }
        let boardScreen = GameBoardScreen(table: shown)
        precondition(boardScreen.deckLine == "Deck size 108", boardScreen.deckLine)
        precondition(boardScreen.roundLine == "Round 2", boardScreen.roundLine)
        precondition(boardScreen.versionLine == "0.3.5", boardScreen.versionLine)
        precondition(boardScreen.board.emptyTitle == nil, "exhibit looked empty")
        precondition(boardScreen.hand.emptyTitle == nil, "exhibit hand looked empty")
        precondition(boardScreen.hand.allowsDrag == true, "exhibit hand does not lift")
        precondition(boardScreen.dropTargets == [.newMeld, .meld(0), .meld(1)], "exhibit zones")
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
        let hand = boardScreen.hand
        precondition(hand.cards.map(\.id) == [21, 22, 23, 24], "hand ids")
        precondition(hand.layout.faces.map(\.spoken) == [
            "Eight of Diamonds",
            "King of Spades",
            "Joker, locked",
            "Two of Clubs",
        ], "hand faces")
        precondition(hand.layout.faces.map(\.isRed) == [true, false, false, false], "hand color")
        precondition(CardView(card: hand.cards[0]).face == hand.layout.faces[0], "hand card view")
        let handIds = Set(shown.picture.hand.map(\.id))
        let boardIds = Set(shown.picture.board.flatMap { $0 }.map(\.id))
        precondition(handIds.isDisjoint(with: boardIds), "hand card is on the board")
        precondition(!handIds.contains(90), "opponent is on the hand")
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
            ],
            hand: [
                CardSnapshot(id: 21, suit: .diamonds, rank: .eight, lockedUntilTurn: 0),
                CardSnapshot(id: 90, suit: .hearts, rank: .ace, lockedUntilTurn: 0),
            ]
        )
        let mixedPicture = picture(from: mixed, version: "0.3.3")
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
        precondition(mixedPicture.hand.map(\.id) == [21, 90], "mapped hand")
        precondition(CardFace(mixedPicture.hand[0]).spoken == "Eight of Diamonds", "mapped eight")
        let pictured = handOnPicture(local: [mixedPicture.hand[0]], opponents: [[mixedPicture.hand[1]]])
        precondition(pictured.map(\.id) == [21], "opponent stayed on the picture")
        precondition(CardFace(pictured[0]).spoken == "Eight of Diamonds", "local face")

        // Chain: exhibit shoe → published rows → drag → PlayMeld or HitMeld
        // → GameError or refusal snaps the hand back. The live game stays the exhibit.
        let eightHearts = BoardCard(id: 40, suit: .hearts, rank: .eight, lockedUntilTurn: 0)
        let eightDiamonds = BoardCard(id: 41, suit: .diamonds, rank: .eight, lockedUntilTurn: 0)
        let king = BoardCard(id: 50, suit: .spades, rank: .king, lockedUntilTurn: 0)
        let lockedJoker = BoardCard(id: 60, suit: .none, rank: .joker, lockedUntilTurn: 2)
        let held = HeldCards(
            hand: [eightHearts, king, eightDiamonds, lockedJoker],
            board: shown.picture.board
        )
        precondition(
            shown.picture.board.flatMap { $0 }.map(\.id).contains(50) == false,
            "hand card is on the published board"
        )
        var dragSignals = 0
        let dragWatch = shown.objectWillChange.sink { dragSignals += 1 }
        var plays = 0
        let snapped = settleDrop(
            table: held,
            drag: CardDrag(cards: [
                BoardCard(id: 41, suit: .spades, rank: .king, lockedUntilTurn: 0),
                eightHearts,
            ]),
            onto: .newMeld
        ) { intent in
            plays += 1
            guard case .playMeld(let groups) = intent else {
                preconditionFailure("play \(intent)")
            }
            precondition(groups.count == 1, "groups \(groups.count)")
            precondition(groups[0].map(\.id) == [41, 40], "play ids")
            precondition(groups[0][0].suit == .diamonds, "forged suit replaced the hand")
            precondition(groups[0][0].rank == .eight, "forged rank replaced the hand")
            return .gameError
        }
        shown.reload()
        precondition(plays == 1, "plays \(plays)")
        precondition(snapped == held, "game error moved the hand")
        precondition(dragSignals == 0, "snap published \(dragSignals)")
        precondition(boardScreen.deckLine == "Deck size 108", boardScreen.deckLine)
        precondition(boardScreen.roundLine == "Round 2", boardScreen.roundLine)
        precondition(boardScreen.board.layout.rows[0].faces.map(\.pip) == ["4", "4", "4"], "set moved")
        precondition(boardScreen.board.layout.rows[1].faces.map(\.pip) == ["4", "5", "6", "7"], "run moved")
        precondition(CardFace(snapped.hand[3]).spoken == "Joker, locked", "lock dropped")
        precondition(exhibit.getDeckSize() == 108, "shoe \(exhibit.getDeckSize())")
        precondition(exhibit.tableSnapshot().board.count == 2, "settle wrote the exhibit")
        precondition(game.tableSnapshot().board.isEmpty, "settle wrote the live game")

        let refused = settleDrop(
            table: held,
            drag: CardDrag(cards: [king]),
            onto: .meld(1)
        ) { intent in
            guard case .hitMeld(let hits) = intent else {
                preconditionFailure("hit \(intent)")
            }
            precondition(hits.count == 1, "hits \(hits.count)")
            precondition(hits[0].meldIndex == 1, "index \(hits[0].meldIndex)")
            precondition(hits[0].cards == [king], "hit cards")
            return .refused
        }
        precondition(refused == held, "refusal moved the king")

        let empty = settleDrop(table: held, drag: CardDrag(cards: []), onto: .newMeld) { _ in
            preconditionFailure("empty drag asked the engine")
        }
        precondition(empty == held, "empty drag changed the hand")

        let outsider = BoardCard(id: 99, suit: .clubs, rank: .ace, lockedUntilTurn: 0)
        let missed = settleDrop(table: held, drag: CardDrag(cards: [outsider]), onto: .meld(0)) { _ in
            preconditionFailure("a card outside the hand asked the engine")
        }
        precondition(missed == held, "outsider changed the hand")

        let repeated = settleDrop(
            table: held,
            drag: CardDrag(cards: [
                king,
                eightHearts,
                BoardCard(id: 50, suit: .hearts, rank: .ace, lockedUntilTurn: 0),
            ]),
            onto: .newMeld
        ) { _ in
            preconditionFailure("a repeated id asked the engine")
        }
        precondition(repeated == held, "repeat changed the hand")

        let playRefusal = settleDrop(
            table: held,
            drag: CardDrag(cards: [eightDiamonds, eightHearts]),
            onto: .newMeld
        ) { intent in
            guard case .playMeld(let groups) = intent else {
                preconditionFailure("refusal play \(intent)")
            }
            precondition(groups == [[eightDiamonds, eightHearts]], "refusal group")
            return .refused
        }
        precondition(playRefusal == held, "play refusal moved the eights")
        precondition(dragSignals == 0, "refusal published \(dragSignals)")

        let engine = HeldCards(
            hand: [eightHearts, eightDiamonds, lockedJoker],
            board: held.board + [[king]]
        )
        let accepted = settleDrop(
            table: held,
            drag: CardDrag(cards: [king]),
            onto: .newMeld
        ) { intent in
            guard case .playMeld(let groups) = intent else {
                preconditionFailure("accept \(intent)")
            }
            precondition(groups == [[king]], "accepted group")
            return .accepted(engine)
        }
        precondition(accepted == engine, "accept rewrote the engine table")
        precondition(exhibit.tableSnapshot().board.count == 2, "accept wrote the exhibit")
        precondition(CardView(card: accepted.board[2][0]).face.spoken == "King of Spades", "king face")
        precondition(boardScreen.board.layout.rows[0].faces.map(\.pip) == ["4", "4", "4"], "accept published")
        precondition(boardScreen.board.layout.rows[1].faces.map(\.pip) == ["4", "5", "6", "7"], "accept moved the run")
        precondition(boardScreen.deckLine == "Deck size 108", boardScreen.deckLine)
        precondition(boardScreen.versionLine == "0.3.5", boardScreen.versionLine)
        precondition(boardScreen.hand.cards.map(\.id) == [21, 22, 23, 24], "accept rewrote the hand")
        precondition(boardScreen.hand.allowsDrag == true, "exhibit hand does not lift")
        let publishedIds = shown.picture.board.flatMap { $0 }.map(\.id) + shown.picture.hand.map(\.id)
        for id in [UInt32(40), 41, 50, 60] {
            precondition(!publishedIds.contains(id), "drag \(id) is on the picture")
        }
        precondition(dragSignals == 0, "accept published \(dragSignals)")

        let gone = settleDrop(table: accepted, drag: CardDrag(cards: [king]), onto: .newMeld) { _ in
            preconditionFailure("a card that left the hand asked the engine")
        }
        precondition(gone == accepted, "left card changed the accept")

        let twoHit = settleDrop(
            table: accepted,
            drag: CardDrag(cards: [
                BoardCard(id: 41, suit: .spades, rank: .king, lockedUntilTurn: 9),
                eightHearts,
            ]),
            onto: .meld(1)
        ) { intent in
            guard case .hitMeld(let hits) = intent else {
                preconditionFailure("two hit \(intent)")
            }
            precondition(hits.count == 1, "hits \(hits.count)")
            precondition(hits[0].meldIndex == 1, "index \(hits[0].meldIndex)")
            precondition(hits[0].cards.map(\.id) == [41, 40], "hit order")
            precondition(hits[0].cards[0].suit == .diamonds, "forged suit replaced the hand")
            precondition(hits[0].cards[0].rank == .eight, "forged rank replaced the hand")
            return .refused
        }
        precondition(twoHit == accepted, "two-card hit moved the accept")
        precondition(boardScreen.roundLine == "Round 2", boardScreen.roundLine)

        let after = settleDrop(
            table: accepted,
            drag: CardDrag(cards: [lockedJoker]),
            onto: .meld(0)
        ) { _ in
            .gameError
        }
        precondition(after == accepted, "later error undid the accept")
        precondition(dragSignals == 0, "later error published \(dragSignals)")
        precondition(game.getDeckSize() == 108, "live shoe")
        dragWatch.cancel()

        let kingCard = boardScreen.hand.cards[1]
        let played = boardScreen.drop(CardDrag(cards: [kingCard]), onto: .newMeld) { intent in
            standInAnswer(picture: shown.picture, intent: intent)
        }
        precondition(played, "king drop did not update the picture")
        precondition(boardScreen.hand.cards.map(\.id) == [21, 23, 24], "king stayed in the hand")
        precondition(boardScreen.board.layout.rows.count == 3, "king did not open a row")
        precondition(
            boardScreen.board.layout.rows[2].faces.map(\.spoken) == ["King of Spades"],
            "king face"
        )
        precondition(boardScreen.versionLine == "0.3.5", boardScreen.versionLine)
        precondition(boardScreen.deckLine == "Deck size 108", boardScreen.deckLine)
        precondition(boardScreen.roundLine == "Round 2", boardScreen.roundLine)
        precondition(exhibit.getDeckSize() == 108, "drop wrote the shoe")

        let jokerCard = boardScreen.hand.cards[1]
        let refusedJoker = boardScreen.drop(CardDrag(cards: [jokerCard]), onto: .meld(0)) { intent in
            standInAnswer(picture: shown.picture, intent: intent)
        }
        precondition(!refusedJoker, "locked joker left the hand")
        precondition(boardScreen.hand.cards.map(\.id) == [21, 23, 24], "joker drop changed the hand")
        precondition(CardFace(boardScreen.hand.cards[1]).spoken == "Joker, locked", "lock")
        precondition(boardScreen.board.layout.rows[0].faces.map(\.pip) == ["4", "4", "4"], "joker hit the set")

        shown.reload()
        precondition(boardScreen.hand.cards.map(\.id) == [21, 22, 23, 24], "reload kept the drop")
        precondition(boardScreen.board.layout.rows.count == 2, "reload kept the king row")
        precondition(boardScreen.hand.allowsDrag == true, "restored hand does not lift")
        print("board-ui ok")
    }
}
