import Combine
import CoreTransferable
import XCTest
@testable import PushUI

/// Drag payload, drop target, and the table the gesture settles.
/// A new-meld drop is one `PlayMeld` group. A meld drop is one `HitMeld`.
/// `GameError` and a rules refusal both return the hand and the board that entered.
final class DragDropTests: XCTestCase {
    func testNewMeldDropIsOnePlayMeldInDragOrder() {
        let hearts = card(id: 40, suit: .hearts, rank: .four, locked: 0)
        let spades = card(id: 42, suit: .spades, rank: .four, locked: 0)
        let diamonds = card(id: 41, suit: .diamonds, rank: .four, locked: 0)
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let hand = [hearts, king, diamonds, spades]
        let intent = moveIntent(
            drag: CardDrag(cards: [spades, hearts, diamonds]),
            onto: .newMeld,
            hand: hand
        )
        XCTAssertEqual(intent, .playMeld([[spades, hearts, diamonds]]))
    }

    func testMeldDropIsOneHitOfThatIndex() {
        let eight = card(id: 8, suit: .hearts, rank: .eight, locked: 0)
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let intent = moveIntent(
            drag: CardDrag(cards: [eight]),
            onto: .meld(0),
            hand: [king, eight]
        )
        XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])]))
    }

    func testHitIndexZeroAndIndexOneStayApart() {
        let three = card(id: 3, suit: .clubs, rank: .three, locked: 0)
        let four = card(id: 4, suit: .spades, rank: .four, locked: 0)
        let hand = [three, four]
        let onTheSet = moveIntent(drag: CardDrag(cards: [three]), onto: .meld(0), hand: hand)
        let onTheRun = moveIntent(drag: CardDrag(cards: [four]), onto: .meld(1), hand: hand)
        XCTAssertEqual(onTheSet, .hitMeld([MeldDrop(meldIndex: 0, cards: [three])]))
        XCTAssertEqual(onTheRun, .hitMeld([MeldDrop(meldIndex: 1, cards: [four])]))
        XCTAssertNotEqual(onTheSet, onTheRun)
    }

    func testAHitOfAMissingRowStillNamesThatIndex() {
        let eight = card(id: 8, suit: .diamonds, rank: .eight, locked: 0)
        let intent = moveIntent(
            drag: CardDrag(cards: [eight]),
            onto: .meld(3),
            hand: [eight]
        )
        XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 3, cards: [eight])]))
    }

    func testIntentUsesTheHandCardWhenTheDragFaceDiffers() {
        let real = card(id: 1, suit: .hearts, rank: .four, locked: 4)
        let forged = card(id: 1, suit: .spades, rank: .king, locked: 0)
        let played = moveIntent(
            drag: CardDrag(cards: [forged]),
            onto: .newMeld,
            hand: [real]
        )
        XCTAssertEqual(played, .playMeld([[real]]))

        let hit = moveIntent(
            drag: CardDrag(cards: [forged]),
            onto: .meld(1),
            hand: [real]
        )
        XCTAssertEqual(hit, .hitMeld([MeldDrop(meldIndex: 1, cards: [real])]))
        XCTAssertEqual(CardFace(real).spoken, "Four of Hearts, locked")
    }

    func testASingleJokerDropIsOnePlayMeld() {
        let joker = card(id: 9, suit: .hearts, rank: .joker, locked: 0)
        let intent = moveIntent(
            drag: CardDrag(cards: [joker]),
            onto: .newMeld,
            hand: [joker]
        )
        XCTAssertEqual(intent, .playMeld([[joker]]))
        XCTAssertEqual(CardFace(joker).spoken, "Joker")
    }

    func testEmptyDropDoesNotAskTheEngine() {
        let four = card(id: 1, suit: .hearts, rank: .four, locked: 0)
        let table = HeldCards(hand: [four], board: [[four]])
        let next = settleDrop(table: table, drag: CardDrag(cards: []), onto: .newMeld) { _ in
            XCTFail("empty drag asked the engine")
            return .gameError
        }
        XCTAssertNil(moveIntent(drag: CardDrag(cards: []), onto: .newMeld, hand: [four]))
        XCTAssertNil(moveIntent(drag: CardDrag(cards: []), onto: .meld(0), hand: [four]))
        XCTAssertEqual(next, table)
    }

    func testACardOutsideTheHandDoesNotAskTheEngine() {
        let held = card(id: 1, suit: .hearts, rank: .four, locked: 0)
        let other = card(id: 2, suit: .spades, rank: .four, locked: 0)
        let table = HeldCards(hand: [held], board: [])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [other]),
            onto: .newMeld
        ) { _ in
            XCTFail("a card outside the hand asked the engine")
            return .accepted(HeldCards(hand: [], board: [[other]]))
        }
        XCTAssertNil(moveIntent(drag: CardDrag(cards: [other]), onto: .meld(0), hand: [held]))
        XCTAssertEqual(next, table)
    }

    func testARepeatedIdDoesNotAskTheEngine() {
        let four = card(id: 7, suit: .hearts, rank: .four, locked: 0)
        let again = card(id: 7, suit: .spades, rank: .five, locked: 0)
        let table = HeldCards(hand: [four], board: [])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [four, again]),
            onto: .newMeld
        ) { _ in
            XCTFail("a repeated id asked the engine")
            return .gameError
        }
        XCTAssertNil(
            moveIntent(drag: CardDrag(cards: [four, again]), onto: .newMeld, hand: [four])
        )
        XCTAssertEqual(next, table)
        XCTAssertEqual(next.hand, [four])
    }

    func testOneMissingCardRefusesTheCardsThatAreInTheHand() {
        let hearts = card(id: 40, suit: .hearts, rank: .four, locked: 0)
        let spades = card(id: 42, suit: .spades, rank: .four, locked: 0)
        let missing = card(id: 99, suit: .clubs, rank: .four, locked: 0)
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let table = HeldCards(hand: [hearts, king, spades], board: [])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [spades, missing, hearts]),
            onto: .newMeld
        ) { _ in
            XCTFail("a partial hand miss asked the engine")
            return .accepted(HeldCards(hand: [king], board: [[spades, hearts]]))
        }
        XCTAssertNil(
            moveIntent(
                drag: CardDrag(cards: [spades, missing, hearts]),
                onto: .meld(0),
                hand: table.hand
            )
        )
        XCTAssertEqual(next.hand.map(\.id), [40, 50, 42])
        XCTAssertTrue(next.board.isEmpty)
    }

    func testGameErrorReturnsTheHandTheLockAndTheBoard() {
        let set = exhibitSet()
        let run = exhibitRun()
        let joker = card(id: 60, suit: .none, rank: .joker, locked: 2)
        let eight = card(id: 80, suit: .hearts, rank: .eight, locked: 0)
        let table = HeldCards(hand: [eight, joker], board: [set, run])
        var asked = 0
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [joker, eight]),
            onto: .newMeld
        ) { intent in
            asked += 1
            XCTAssertEqual(intent, .playMeld([[joker, eight]]))
            return .gameError
        }
        XCTAssertEqual(asked, 1)
        XCTAssertEqual(next, table)
        XCTAssertEqual(CardFace(next.hand[1]).spoken, "Joker, locked")
        XCTAssertEqual(next.board[0].map(\.id), [1, 2, 3])
        XCTAssertEqual(next.board[1].map(\.id), [10, 11, 12, 13])
    }

    func testARefusalReturnsTheHandAndTheBoard() {
        let set = exhibitSet()
        let run = exhibitRun()
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let table = HeldCards(hand: [king], board: [set, run])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [king]),
            onto: .meld(1)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [king])]))
            return .refused
        }
        XCTAssertEqual(next, table)
        XCTAssertEqual(next.hand, [king])
        XCTAssertEqual(next.board[1].map(\.rank), [.four, .five, .six, .seven])
    }

    func testAnAcceptedDropReturnsTheEngineTableVerbatim() {
        let set = exhibitSet()
        let run = exhibitRun()
        let hearts = card(id: 40, suit: .hearts, rank: .four, locked: 0)
        let diamonds = card(id: 41, suit: .diamonds, rank: .four, locked: 0)
        let spades = card(id: 42, suit: .spades, rank: .four, locked: 0)
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let table = HeldCards(hand: [hearts, king, diamonds, spades], board: [set, run])
        let engine = HeldCards(
            hand: [king],
            board: [set, run, [spades, hearts, diamonds]]
        )
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [spades, hearts, diamonds]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[spades, hearts, diamonds]]))
            return .accepted(engine)
        }
        XCTAssertEqual(next, engine)
        XCTAssertEqual(next.hand.map(\.id), [50])
        XCTAssertEqual(next.board[2].map(\.suit), [.spades, .hearts, .diamonds])
    }

    func testAGameErrorAfterAnAcceptLeavesThatAccept() {
        let set = exhibitSet()
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let joker = card(id: 60, suit: .none, rank: .joker, locked: 2)
        let accepted = HeldCards(hand: [joker], board: [set + [king]])
        let next = settleDrop(
            table: accepted,
            drag: CardDrag(cards: [joker]),
            onto: .meld(0)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [joker])]))
            return .gameError
        }
        XCTAssertEqual(next, accepted)
        XCTAssertEqual(next.board, [set + [king]])
        XCTAssertEqual(CardFace(next.hand[0]).spoken, "Joker, locked")
    }

    func testACardOnTheBoardAndInTheHandMapsAndARefusalRestoresIt() {
        let four = card(id: 1, suit: .hearts, rank: .four, locked: 0)
        let table = HeldCards(hand: [four], board: [[four]])
        var asked = false
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [four]),
            onto: .meld(0)
        ) { intent in
            asked = true
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [four])]))
            return .refused
        }
        XCTAssertTrue(asked)
        XCTAssertEqual(next, table)
    }

    func testCardDragEncodesTheJokerTheLockAndBothFours() throws {
        let joker = card(id: 9, suit: .hearts, rank: .joker, locked: 0)
        let locked = card(id: 5, suit: .diamonds, rank: .queen, locked: 4)
        let hearts = card(id: 1, suit: .hearts, rank: .four, locked: 0)
        let heartsAgain = card(id: 14, suit: .hearts, rank: .four, locked: 0)
        let drag = CardDrag(cards: [joker, locked, hearts, heartsAgain])
        let data = try JSONEncoder().encode(drag)
        let back = try JSONDecoder().decode(CardDrag.self, from: data)
        XCTAssertEqual(back, drag)
        XCTAssertEqual(back.cards.map(\.id), [9, 5, 1, 14])
        XCTAssertEqual(CardFace(back.cards[0]).spoken, "Joker")
        XCTAssertFalse(CardFace(back.cards[0]).isRed)
        XCTAssertEqual(CardFace(back.cards[1]).spoken, "Queen of Diamonds, locked")
        XCTAssertTrue(CardFace(back.cards[1]).isLocked)
        XCTAssertEqual(CardFace(back.cards[2]).spoken, "Four of Hearts")
        XCTAssertEqual(CardFace(back.cards[3]).spoken, "Four of Hearts")
        XCTAssertNotEqual(back.cards[2].id, back.cards[3].id)
    }

    func testAHitOfSeveralCardsKeepsDragOrderAndTheHandFace() {
        let joker = card(id: 60, suit: .hearts, rank: .joker, locked: 2)
        let two = card(id: 70, suit: .clubs, rank: .two, locked: 0)
        let eight = card(id: 80, suit: .diamonds, rank: .eight, locked: 0)
        let forgedJoker = card(id: 60, suit: .spades, rank: .king, locked: 0)
        let forgedTwo = card(id: 70, suit: .hearts, rank: .ace, locked: 4)
        let hand = [eight, joker, two]
        let intent = moveIntent(
            drag: CardDrag(cards: [forgedTwo, eight, forgedJoker]),
            onto: .meld(0),
            hand: hand
        )
        XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [two, eight, joker])]))
        guard case .hitMeld(let hits) = intent else {
            return XCTFail("hit \(String(describing: intent))")
        }
        XCTAssertEqual(hits.count, 1)
        XCTAssertEqual(hits[0].cards.map(\.id), [70, 80, 60])
        XCTAssertEqual(CardFace(hits[0].cards[0]).spoken, "Two of Clubs")
        XCTAssertFalse(CardFace(hits[0].cards[0]).isRed)
        XCTAssertEqual(CardFace(hits[0].cards[1]).spoken, "Eight of Diamonds")
        XCTAssertTrue(CardFace(hits[0].cards[1]).isRed)
        XCTAssertEqual(CardFace(hits[0].cards[2]).spoken, "Joker, locked")
        XCTAssertFalse(CardFace(hits[0].cards[2]).isRed)
        XCTAssertFalse(hits[0].cards.map(\.id).contains(1))
    }

    func testSixCardsStayOnePlayMeldGroup() {
        let cards = (0..<6).map { offset in
            card(id: UInt32(40 + offset), suit: .hearts, rank: .four, locked: 0)
        }
        let intent = moveIntent(
            drag: CardDrag(cards: cards.reversed()),
            onto: .newMeld,
            hand: cards
        )
        XCTAssertEqual(intent, .playMeld([cards.reversed()]))
        guard case .playMeld(let groups) = intent else {
            return XCTFail("play \(String(describing: intent))")
        }
        XCTAssertEqual(groups.count, 1)
        XCTAssertEqual(groups[0].count, 6)
    }

    func testAPlayRefusalReturnsTheHandTheEmptyMeldAndTheLock() {
        let set = exhibitSet()
        let joker = card(id: 60, suit: .none, rank: .joker, locked: 1)
        let two = card(id: 70, suit: .clubs, rank: .two, locked: 0)
        let table = HeldCards(hand: [joker, two], board: [set, []])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [two, joker]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[two, joker]]))
            return .refused
        }
        XCTAssertEqual(next, table)
        XCTAssertEqual(next.hand.map(\.id), [60, 70])
        XCTAssertEqual(next.board.count, 2)
        XCTAssertTrue(next.board[1].isEmpty)
        XCTAssertEqual(CardFace(next.hand[0]).spoken, "Joker, locked")
    }

    func testANegativeIndexAndALargeIndexAreDifferentHits() {
        let three = card(id: 3, suit: .spades, rank: .three, locked: 0)
        let hand = [three]
        let low = moveIntent(drag: CardDrag(cards: [three]), onto: .meld(-1), hand: hand)
        let high = moveIntent(drag: CardDrag(cards: [three]), onto: .meld(Int.max), hand: hand)
        XCTAssertEqual(low, .hitMeld([MeldDrop(meldIndex: -1, cards: [three])]))
        XCTAssertEqual(high, .hitMeld([MeldDrop(meldIndex: Int.max, cards: [three])]))
        XCTAssertNotEqual(low, high)
    }

    func testAnEmptyHandDoesNotAskTheEngine() {
        let four = card(id: 1, suit: .hearts, rank: .four, locked: 0)
        let table = HeldCards(hand: [], board: [[four]])
        let named = settleDrop(table: table, drag: CardDrag(cards: [four]), onto: .meld(0)) { _ in
            XCTFail("an empty hand asked the engine")
            return .accepted(HeldCards(hand: [four], board: []))
        }
        let empty = settleDrop(table: table, drag: CardDrag(cards: []), onto: .newMeld) { _ in
            XCTFail("an empty drag asked the engine")
            return .gameError
        }
        XCTAssertNil(moveIntent(drag: CardDrag(cards: [four]), onto: .newMeld, hand: []))
        XCTAssertEqual(named, table)
        XCTAssertEqual(empty, table)
        XCTAssertTrue(named.hand.isEmpty)
        XCTAssertEqual(named.board, [[four]])
    }

    func testARepeatBetweenTwoCardsDoesNotAskTheEngine() {
        let hearts = card(id: 40, suit: .hearts, rank: .four, locked: 0)
        let spades = card(id: 42, suit: .spades, rank: .four, locked: 0)
        let again = card(id: 40, suit: .clubs, rank: .five, locked: 0)
        let table = HeldCards(hand: [hearts, spades], board: [])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [spades, hearts, again]),
            onto: .meld(1)
        ) { _ in
            XCTFail("a repeated id between two cards asked the engine")
            return .refused
        }
        XCTAssertNil(
            moveIntent(
                drag: CardDrag(cards: [spades, hearts, again]),
                onto: .newMeld,
                hand: table.hand
            )
        )
        XCTAssertEqual(next, table)
    }

    func testAnAcceptThatStillHoldsTheDraggedCardReturnsThatEngineTable() {
        let set = exhibitSet()
        let eight = card(id: 80, suit: .hearts, rank: .eight, locked: 0)
        let sentinel = card(id: 999, suit: .clubs, rank: .ace, locked: 0)
        let table = HeldCards(hand: [eight], board: [set])
        let engine = HeldCards(hand: [eight, sentinel], board: [set])
        let next = settleDrop(
            table: table,
            drag: CardDrag(cards: [eight]),
            onto: .meld(0)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])]))
            return .accepted(engine)
        }
        XCTAssertEqual(next, engine)
        XCTAssertEqual(next.hand.map(\.id), [80, 999])
        XCTAssertEqual(CardFace(next.hand[1]).spoken, "Ace of Clubs")
        XCTAssertFalse(CardFace(next.hand[1]).isRed)
    }

    func testAfterTheCardLeavesASecondDropDoesNotAsk() {
        let set = exhibitSet()
        let run = exhibitRun()
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let joker = card(id: 60, suit: .none, rank: .joker, locked: 2)
        let start = HeldCards(hand: [king, joker], board: [set, run])
        let engine = HeldCards(hand: [joker], board: [set + [king], run])
        let accepted = settleDrop(
            table: start,
            drag: CardDrag(cards: [king]),
            onto: .meld(0)
        ) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(accepted, engine)
        var asked = 0
        let again = settleDrop(
            table: accepted,
            drag: CardDrag(cards: [king]),
            onto: .newMeld
        ) { _ in
            asked += 1
            return .gameError
        }
        XCTAssertEqual(asked, 0)
        XCTAssertEqual(again, engine)
        XCTAssertEqual(CardFace(again.hand[0]).spoken, "Joker, locked")

        let stillThere = settleDrop(
            table: accepted,
            drag: CardDrag(cards: [joker]),
            onto: .meld(1)
        ) { intent in
            asked += 1
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [joker])]))
            return .gameError
        }
        XCTAssertEqual(asked, 1)
        XCTAssertEqual(stillThere, engine)
        XCTAssertEqual(stillThere.board[0].map(\.id), [1, 2, 3, 50])
    }

    func testCardDragRoundTripsThroughJSONTransfer() async throws {
        let joker = card(id: 9, suit: .hearts, rank: .joker, locked: 0)
        let locked = card(id: 5, suit: .clubs, rank: .queen, locked: 4)
        let diamond = card(id: 8, suit: .diamonds, rank: .two, locked: 0)
        let drag = CardDrag(cards: [joker, locked, diamond])
        guard #available(iOS 18.2, macOS 15.2, *) else {
            throw XCTSkip("JSON transfer export is unavailable")
        }
        XCTAssertTrue(CardDrag.exportedContentTypes().contains(.json))
        let data = try await drag.exported(as: .json)
        let back = try await CardDrag(importing: data, contentType: .json)
        XCTAssertEqual(back, drag)
        XCTAssertEqual(back.cards.map(\.id), [9, 5, 8])
        XCTAssertEqual(CardFace(back.cards[0]).spoken, "Joker")
        XCTAssertFalse(CardFace(back.cards[0]).isRed)
        XCTAssertEqual(CardFace(back.cards[1]).spoken, "Queen of Clubs, locked")
        XCTAssertFalse(CardFace(back.cards[1]).isRed)
        XCTAssertTrue(CardFace(back.cards[1]).isLocked)
        XCTAssertEqual(CardFace(back.cards[2]).spoken, "Two of Diamonds")
        XCTAssertTrue(CardFace(back.cards[2]).isRed)
    }

    /// Chain: published exhibit → card face → board rows → drag → PlayMeld
    /// → HitMeld → GameError snap-back. The shoe line stays 108.
    func testPublishedExhibitDragPlayThenHitSnapsBackAndKeepsTheRows() {
        let set = exhibitSet()
        let run = exhibitRun()
        var loads = 0
        let table = PublishedTable {
            loads += 1
            return TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 2, board: [set, run])
        }
        let screen = GameBoardScreen(table: table)
        XCTAssertEqual(loads, 1)
        XCTAssertEqual(screen.versionLine, "0.3.2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(screen.board.layout.rows.count, 2)
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.spoken), [
            "Four of Hearts",
            "Four of Spades",
            "Four of Clubs",
        ])
        XCTAssertEqual(CardView(card: set[0]).face, screen.board.layout.rows[0].faces[0])

        let hearts = card(id: 40, suit: .hearts, rank: .eight, locked: 0)
        let diamonds = card(id: 41, suit: .diamonds, rank: .eight, locked: 0)
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let joker = card(id: 60, suit: .none, rank: .joker, locked: 2)
        let start = HeldCards(hand: [hearts, king, diamonds, joker], board: table.picture.board)
        XCTAssertFalse(table.picture.board.flatMap { $0 }.map(\.id).contains(50))

        var signals = 0
        let sub = table.objectWillChange.sink { signals += 1 }
        let snapped = settleDrop(
            table: start,
            drag: CardDrag(cards: [diamonds, hearts]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[diamonds, hearts]]))
            return .gameError
        }
        table.reload()
        XCTAssertEqual(snapped, start)
        XCTAssertEqual(signals, 0)
        XCTAssertEqual(loads, 2)
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])
        XCTAssertEqual(CardFace(snapped.hand[3]).spoken, "Joker, locked")

        let engine = HeldCards(
            hand: [hearts, diamonds, joker],
            board: [set, run + [king]]
        )
        let hit = settleDrop(
            table: start,
            drag: CardDrag(cards: [king]),
            onto: .meld(1)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [king])]))
            return .accepted(engine)
        }
        XCTAssertEqual(hit, engine)
        XCTAssertEqual(hit.board[0].map(\.id), [1, 2, 3])
        XCTAssertEqual(CardFace(hit.board[1][4]).spoken, "King of Spades")
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])

        let still = settleDrop(
            table: hit,
            drag: CardDrag(cards: [joker]),
            onto: .meld(0)
        ) { _ in
            .gameError
        }
        XCTAssertEqual(still, hit)
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(screen.versionLine, "0.3.2")
        sub.cancel()
    }

    /// Chain: empty published shoe → exhibit faces → PlayMeld refusal →
    /// HitMeld accept → the hand stays off the picture.
    func testEmptyShoeThenExhibitFacesPlayRefusalHitAcceptAndTheHandStaysOffThePicture() {
        let set = exhibitSet()
        let run = exhibitRun()
        var loads = 0
        let table = PublishedTable {
            loads += 1
            if loads == 1 {
                return TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 1, board: [])
            }
            return TablePicture(version: "0.3.2", deckSize: 108, roundNumber: 2, board: [set, run])
        }
        let screen = GameBoardScreen(table: table)
        XCTAssertEqual(loads, 1)
        XCTAssertEqual(screen.versionLine, "0.3.2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 1")
        XCTAssertEqual(screen.board.emptyTitle, "No melds")
        XCTAssertTrue(screen.board.layout.rows.isEmpty)

        var signals = 0
        let sub = table.objectWillChange.sink { signals += 1 }
        table.reload()
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(loads, 2)
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertNil(screen.board.emptyTitle)
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.spoken), [
            "Four of Hearts",
            "Four of Spades",
            "Four of Clubs",
        ])
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.isRed), [true, false, false])
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])
        XCTAssertTrue(screen.board.layout.rows[1].faces.allSatisfy { $0.isRed && !$0.isLocked })
        XCTAssertEqual(CardView(card: set[2]).face, screen.board.layout.rows[0].faces[2])

        let hearts = card(id: 40, suit: .hearts, rank: .four, locked: 0)
        let diamonds = card(id: 41, suit: .diamonds, rank: .four, locked: 0)
        let spades = card(id: 42, suit: .spades, rank: .four, locked: 0)
        let clubs = card(id: 43, suit: .clubs, rank: .four, locked: 0)
        let eight = card(id: 80, suit: .hearts, rank: .eight, locked: 0)
        let two = card(id: 70, suit: .clubs, rank: .two, locked: 0)
        let joker = card(id: 60, suit: .none, rank: .joker, locked: 2)
        let king = card(id: 50, suit: .spades, rank: .king, locked: 0)
        let start = HeldCards(
            hand: [hearts, diamonds, spades, clubs, eight, two, joker, king],
            board: table.picture.board
        )
        let handIds: Set<UInt32> = [40, 41, 42, 43, 80, 70, 60, 50]
        XCTAssertTrue(handIds.isDisjoint(with: table.picture.board.flatMap { $0 }.map(\.id)))

        let untouched = settleDrop(table: start, drag: CardDrag(cards: []), onto: .newMeld) { _ in
            XCTFail("empty drag asked the engine")
            return .gameError
        }
        let outsider = card(id: 99, suit: .hearts, rank: .ace, locked: 0)
        let missed = settleDrop(
            table: start,
            drag: CardDrag(cards: [outsider]),
            onto: .meld(0)
        ) { _ in
            XCTFail("a card outside the hand asked the engine")
            return .accepted(HeldCards(hand: [], board: [[outsider]]))
        }
        let repeated = settleDrop(
            table: start,
            drag: CardDrag(cards: [spades, hearts, card(id: 42, suit: .hearts, rank: .ace, locked: 0)]),
            onto: .newMeld
        ) { _ in
            XCTFail("a repeated id asked the engine")
            return .refused
        }
        XCTAssertEqual(untouched, start)
        XCTAssertEqual(missed, start)
        XCTAssertEqual(repeated, start)
        XCTAssertEqual(signals, 1)

        let refused = settleDrop(
            table: start,
            drag: CardDrag(cards: [spades, hearts, diamonds]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[spades, hearts, diamonds]]))
            return .refused
        }
        XCTAssertEqual(refused, start)
        XCTAssertEqual(screen.board.emptyTitle, nil)
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])

        let engine = HeldCards(
            hand: [hearts, diamonds, spades, clubs, two, joker, king],
            board: [set, run + [eight]]
        )
        let forged = card(id: 80, suit: .clubs, rank: .king, locked: 9)
        let accepted = settleDrop(
            table: start,
            drag: CardDrag(cards: [forged]),
            onto: .meld(1)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [eight])]))
            return .accepted(engine)
        }
        XCTAssertEqual(accepted, engine)
        XCTAssertEqual(CardFace(accepted.board[1][4]).spoken, "Eight of Hearts")
        XCTAssertTrue(CardFace(accepted.board[1][4]).isRed)
        XCTAssertEqual(BoardView(melds: accepted.board).layout.rows[1].faces.map(\.pip), [
            "4", "5", "6", "7", "8",
        ])
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.suitSymbol), ["♥", "♠", "♣"])
        XCTAssertTrue(handIds.isDisjoint(with: table.picture.board.flatMap { $0 }.map(\.id)))
        XCTAssertEqual(screen.versionLine, "0.3.2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(signals, 1)

        let snapped = settleDrop(
            table: accepted,
            drag: CardDrag(cards: [joker]),
            onto: .meld(0)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [joker])]))
            return .gameError
        }
        XCTAssertEqual(snapped, accepted)
        XCTAssertEqual(CardFace(snapped.hand[5]).spoken, "Joker, locked")
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(loads, 2)
        sub.cancel()
    }
}

private func exhibitSet() -> [BoardCard] {
    [
        card(id: 1, suit: .hearts, rank: .four, locked: 0),
        card(id: 2, suit: .spades, rank: .four, locked: 0),
        card(id: 3, suit: .clubs, rank: .four, locked: 0),
    ]
}

private func exhibitRun() -> [BoardCard] {
    [
        card(id: 10, suit: .hearts, rank: .four, locked: 0),
        card(id: 11, suit: .hearts, rank: .five, locked: 0),
        card(id: 12, suit: .hearts, rank: .six, locked: 0),
        card(id: 13, suit: .hearts, rank: .seven, locked: 0),
    ]
}

private func card(id: UInt32, suit: BoardSuit, rank: BoardRank, locked: UInt32) -> BoardCard {
    BoardCard(id: id, suit: suit, rank: rank, lockedUntilTurn: locked)
}
