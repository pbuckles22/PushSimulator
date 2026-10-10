import Combine
import XCTest
@testable import PushUI

/// On-screen drag. A lift is a card in the local hand. A drop calls `settleDrop`.
/// `GameError` and a refusal leave that card in the hand. The picture changes only
/// when the stand-in accepts. The stand-in is the closure. Rust is not on this path.
final class OnScreenDragTests: XCTestCase {
    func testAHandWithCardsLiftsAndAnEmptyHandDoesNot() {
        let empty = HandView(cards: [])
        XCTAssertFalse(empty.allowsDrag)
        XCTAssertFalse(empty.canLift(exhibitHand()[0]))
        XCTAssertEqual(empty.emptyTitle, "No cards")

        let hand = HandView(cards: exhibitHand())
        XCTAssertTrue(hand.allowsDrag)
        XCTAssertNil(hand.emptyTitle)
        XCTAssertEqual(hand.layout.faces.map(\.spoken), [
            "Eight of Diamonds",
            "King of Spades",
            "Joker, locked",
            "Two of Clubs",
        ])
        for card in exhibitHand() {
            XCTAssertTrue(hand.canLift(card))
        }
    }

    func testALockedJokerAndAWildLiftAndAnOpponentOrABoardCardDoesNot() {
        let hand = HandView(cards: exhibitHand())
        let joker = exhibitHand()[2]
        let two = exhibitHand()[3]
        XCTAssertTrue(hand.canLift(joker))
        XCTAssertTrue(hand.canLift(two))
        XCTAssertEqual(CardFace(joker).spoken, "Joker, locked")
        XCTAssertEqual(CardFace(two).spoken, "Two of Clubs")

        let opponent = card(id: 90, suit: .hearts, rank: .ace, locked: 0)
        let boardFour = exhibitSet()[0]
        XCTAssertFalse(hand.canLift(opponent))
        XCTAssertFalse(hand.canLift(boardFour))
        XCTAssertFalse(hand.cards.map(\.id).contains(opponent.id))
        XCTAssertFalse(hand.cards.map(\.id).contains(boardFour.id))
    }

    func testLiftingTheEightHidesOnlyThatSlotAndTheKingStaysUntilTheLiftEnds() {
        let eight = exhibitHand()[0].id
        let king = exhibitHand()[1].id
        let two = exhibitHand()[3].id
        XCTAssertFalse(handSlotIsHidden(cardId: eight, liftedId: nil))
        XCTAssertTrue(handSlotIsHidden(cardId: eight, liftedId: eight))
        XCTAssertFalse(handSlotIsHidden(cardId: king, liftedId: eight))
        XCTAssertFalse(handSlotIsHidden(cardId: two, liftedId: eight))
        XCTAssertTrue(handSlotIsHidden(cardId: king, liftedId: king))
        XCTAssertFalse(handSlotIsHidden(cardId: eight, liftedId: king))
        XCTAssertFalse(handSlotIsHidden(cardId: two, liftedId: king))
        XCTAssertFalse(handSlotIsHidden(cardId: king, liftedId: nil))
    }

    func testADragSessionHidesThatHandSlotAndTheEndShowsIt() {
        let eight = exhibitHand()[0].id
        let king = exhibitHand()[1].id
        XCTAssertEqual(HandSlotVisibility.duringDrag(of: eight).alpha, 0)
        XCTAssertEqual(HandSlotVisibility.afterDrag(of: eight).alpha, 1)
        XCTAssertEqual(HandSlotVisibility.duringDrag(of: king).alpha, 0)
        XCTAssertEqual(HandSlotVisibility.afterDrag(of: king).alpha, 1)
    }

    /// Chain: the lifted two's drag bytes → the card the drop reads → the hit on the fours.
    func testLiftedTwoOfClubsLoadsAsTheHitOntoTheFours() {
        let two = exhibitHand()[3]
        let expectation = expectation(description: "lifted two")
        let provider = handCardItemProvider(for: CardDrag(cards: [two]))
        var loaded: CardDrag?
        let accepted = acceptLiftedCards([provider]) { drag in
            loaded = drag
            expectation.fulfill()
            return true
        }
        XCTAssertTrue(accepted)
        wait(for: [expectation], timeout: 2)
        XCTAssertEqual(loaded, CardDrag(cards: [two]))
        XCTAssertEqual(
            moveIntent(drag: loaded ?? CardDrag(cards: []), onto: .meld(0), hand: exhibitHand()),
            .hitMeld([MeldDrop(meldIndex: 0, cards: [two])])
        )
    }

    func testAnInAppLiftDropsTheTwoImmediatelyAndASecondDropDoesNotRun() {
        let two = exhibitHand()[3]
        let eight = exhibitHand()[0]
        XCTAssertEqual(handCardPreviewLines(eight), ["8", "♦"])
        XCTAssertEqual(handCardPreviewLines(exhibitHand()[1]), ["K", "♠"])
        XCTAssertEqual(handCardPreviewLines(exhibitHand()[2]), ["Joker", "locked"])
        XCTAssertEqual(handCardPreviewLines(two), ["2", "♣"])
        LiftedHandCard.begin(CardDrag(cards: [two]))
        var calls = 0
        let accepted = acceptLiftedCards([]) { drag in
            calls += 1
            XCTAssertEqual(drag, CardDrag(cards: [two]))
            return true
        }
        XCTAssertTrue(accepted)
        XCTAssertEqual(calls, 1)
        let again = acceptLiftedCards([]) { _ in
            calls += 1
            return true
        }
        XCTAssertFalse(again)
        XCTAssertEqual(calls, 1)
        LiftedHandCard.begin(CardDrag(cards: [eight]))
        let refused = acceptLiftedCards([]) { _ in
            false
        }
        XCTAssertFalse(refused)
        LiftedHandCard.end()
    }

    func testTheSameFaceWithAnotherIdDoesNotLift() {
        let hand = HandView(cards: exhibitHand())
        let otherEight = card(id: 99, suit: .diamonds, rank: .eight, locked: 0)
        XCTAssertEqual(CardFace(otherEight).spoken, CardFace(exhibitHand()[0]).spoken)
        XCTAssertFalse(hand.canLift(otherEight))
    }

    func testASharedIdLiftsTheHandCard() {
        let local = card(id: 21, suit: .diamonds, rank: .eight, locked: 0)
        let hand = HandView(cards: [local])
        let sameIdOnTheBoard = card(id: 21, suit: .hearts, rank: .four, locked: 0)
        XCTAssertTrue(hand.canLift(sameIdOnTheBoard))
        XCTAssertEqual(CardFace(hand.cards[0]).spoken, "Eight of Diamonds")
    }

    func testAnIdOfZeroInTheHandLifts() {
        let zero = card(id: 0, suit: .clubs, rank: .three, locked: 0)
        let hand = HandView(cards: [zero])
        XCTAssertTrue(hand.allowsDrag)
        XCTAssertTrue(hand.canLift(zero))
        let sameId = card(id: 0, suit: .spades, rank: .king, locked: 1)
        XCTAssertTrue(hand.canLift(sameId))
        let next = applyScreenDrop(
            picture: TablePicture(
                version: "0.3.3",
                deckSize: 108,
                roundNumber: 2,
                board: [],
                hand: [zero]
            ),
            drag: CardDrag(cards: [sameId]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[zero]]))
            return .refused
        }
        XCTAssertEqual(next.hand, [zero])
        XCTAssertEqual(CardFace(next.hand[0]).spoken, "Three of Clubs")
    }

    func testDropTargetsAreTheNewMeldZoneAndEachRowIncludingAnEmptyMeld() {
        let hosted = host(exhibitPicture())
        XCTAssertEqual(hosted.screen.dropTargets, [.newMeld, .meld(0), .meld(1)])

        let withAGap = TablePicture(
            version: "0.3.3",
            deckSize: 108,
            roundNumber: 2,
            board: [exhibitSet(), [], exhibitRun()],
            hand: exhibitHand()
        )
        XCTAssertEqual(host(withAGap).screen.dropTargets, [
            .newMeld, .meld(0), .meld(1), .meld(2),
        ])
        XCTAssertNil(BoardView(melds: withAGap.board).emptyTitle)
        XCTAssertTrue(BoardView(melds: withAGap.board).layout.rows[1].faces.isEmpty)
    }

    func testAnEmptyBoardStillHasTheNewMeldZone() {
        let hosted = host(TablePicture(
            version: "0.3.3",
            deckSize: 108,
            roundNumber: 1,
            board: [],
            hand: []
        ))
        XCTAssertEqual(hosted.screen.dropTargets, [.newMeld])
        XCTAssertEqual(hosted.screen.board.emptyTitle, "No melds")
        XCTAssertFalse(hosted.screen.hand.allowsDrag)
    }

    func testANewMeldDropCallsPlayMeldInDragOrderWithTheHandCard() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let two = picture.hand[3]
        let forgedEight = card(id: eight.id, suit: .spades, rank: .king, locked: 9)
        var asked = 0
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [two, forgedEight]),
            onto: .newMeld
        ) { intent in
            asked += 1
            XCTAssertEqual(intent, .playMeld([[two, eight]]))
            return .refused
        }
        XCTAssertEqual(asked, 1)
        XCTAssertEqual(next, picture)
        XCTAssertEqual(CardFace(next.hand[0]).spoken, "Eight of Diamonds")
    }

    func testAMeldDropCallsHitMeldOfThatIndex() {
        let picture = exhibitPicture()
        let king = picture.hand[1]
        var asked = 0
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king]),
            onto: .meld(1)
        ) { intent in
            asked += 1
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [king])]))
            return .refused
        }
        XCTAssertEqual(asked, 1)
        XCTAssertEqual(next.hand.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(next.board[1].map(\.id), [10, 11, 12, 13])
    }

    func testHitIndexZeroAndIndexOneStayApartOnThePicture() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let king = picture.hand[1]
        let onTheSet = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [eight]),
            onto: .meld(0)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])]))
            return .accepted(HeldCards(
                hand: [king, picture.hand[2], picture.hand[3]],
                board: [picture.board[0] + [eight], picture.board[1]]
            ))
        }
        let onTheRun = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king]),
            onto: .meld(1)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [king])]))
            return .accepted(HeldCards(
                hand: [eight, picture.hand[2], picture.hand[3]],
                board: [picture.board[0], picture.board[1] + [king]]
            ))
        }
        XCTAssertEqual(onTheSet.board[0].map(\.id), [1, 2, 3, 21])
        XCTAssertEqual(onTheSet.hand.map(\.id), [22, 23, 24])
        XCTAssertEqual(onTheRun.board[1].map(\.id), [10, 11, 12, 13, 22])
        XCTAssertEqual(onTheRun.hand.map(\.id), [21, 23, 24])
        XCTAssertNotEqual(onTheSet, onTheRun)
        XCTAssertEqual(onTheSet.version, picture.version)
        XCTAssertEqual(onTheRun.deckSize, 108)
    }

    func testGameErrorLeavesThePictureTheLockAndTheCard() {
        let picture = exhibitPicture()
        let joker = picture.hand[2]
        let tempting = HeldCards(hand: [], board: [picture.board[0] + [joker], picture.board[1]])
        var asked = 0
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [joker]),
            onto: .meld(0)
        ) { intent in
            asked += 1
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [joker])]))
            _ = tempting
            return .gameError
        }
        XCTAssertEqual(asked, 1)
        XCTAssertEqual(next, picture)
        XCTAssertEqual(CardFace(next.hand[2]).spoken, "Joker, locked")
        XCTAssertEqual(next.hand[2].lockedUntilTurn, 2)
        XCTAssertEqual(next.board[0].map(\.id), [1, 2, 3])
        XCTAssertEqual(next.version, "0.3.3")
        XCTAssertEqual(next.deckSize, 108)
        XCTAssertEqual(next.roundNumber, 2)
    }

    func testARefusalLeavesThePictureEvenWhenTheStandInBuiltAnotherTable() {
        let picture = exhibitPicture()
        let king = picture.hand[1]
        let other = HeldCards(
            hand: [picture.hand[0]],
            board: [picture.board[0], picture.board[1] + [king]]
        )
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[king]]))
            _ = other
            return .refused
        }
        XCTAssertEqual(next, picture)
        XCTAssertEqual(next.hand.map(\.id), [21, 22, 23, 24])
        XCTAssertFalse(next.board.flatMap { $0 }.map(\.id).contains(king.id))
    }

    func testAnAcceptShowsTheEngineHandAndBoardAndKeepsTheShoe() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let forged = card(id: eight.id, suit: .clubs, rank: .ace, locked: 4)
        let engine = HeldCards(
            hand: Array(picture.hand.dropFirst()),
            board: [picture.board[0] + [eight], picture.board[1]]
        )
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [forged]),
            onto: .meld(0)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])]))
            return .accepted(engine)
        }
        XCTAssertEqual(next.hand, engine.hand)
        XCTAssertEqual(next.board, engine.board)
        XCTAssertEqual(next.version, "0.3.3")
        XCTAssertEqual(next.deckSize, 108)
        XCTAssertEqual(next.roundNumber, 2)
        XCTAssertEqual(CardFace(next.board[0][3]).spoken, "Eight of Diamonds")
        XCTAssertEqual(CardView(card: next.board[0][3]).face, CardFace(eight))
        XCTAssertFalse(next.hand.map(\.id).contains(eight.id))
        XCTAssertEqual(HandView(cards: next.hand).layout.faces.map(\.spoken), [
            "King of Spades",
            "Joker, locked",
            "Two of Clubs",
        ])
    }

    func testAnAcceptKeepsAShoeOfZeroAndTheRound() {
        let picture = TablePicture(
            version: "0.3.3",
            deckSize: 0,
            roundNumber: 6,
            board: [exhibitRun()],
            hand: [exhibitHand()[3]]
        )
        let two = picture.hand[0]
        let engine = HeldCards(hand: [], board: [picture.board[0] + [two]])
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [two]),
            onto: .meld(0)
        ) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(next.deckSize, 0)
        XCTAssertEqual(next.roundNumber, 6)
        XCTAssertEqual(next.version, "0.3.3")
        XCTAssertTrue(next.hand.isEmpty)
        XCTAssertEqual(CardFace(next.board[0][4]).spoken, "Two of Clubs")
    }

    func testAnAcceptThatEmptiesTheHandStopsTheLift() {
        let only = exhibitHand()[1]
        let picture = TablePicture(
            version: "0.3.3",
            deckSize: 108,
            roundNumber: 2,
            board: [],
            hand: [only]
        )
        let engine = HeldCards(hand: [], board: [[only]])
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [only]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[only]]))
            return .accepted(engine)
        }
        let hand = HandView(cards: next.hand)
        XCTAssertEqual(hand.emptyTitle, "No cards")
        XCTAssertFalse(hand.allowsDrag)
        XCTAssertFalse(hand.canLift(only))
        XCTAssertNil(BoardView(melds: next.board).emptyTitle)
        XCTAssertEqual(BoardView(melds: next.board).layout.rows[0].faces.map(\.spoken), [
            "King of Spades",
        ])
    }

    func testAnAcceptThatReturnsAnEmptyMeldKeepsTheRow() {
        let picture = exhibitPicture()
        let engine = HeldCards(hand: picture.hand, board: [picture.board[0], []])
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [picture.hand[0]]),
            onto: .meld(1)
        ) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(next.board.count, 2)
        XCTAssertTrue(next.board[1].isEmpty)
        XCTAssertNil(BoardView(melds: next.board).emptyTitle)
        XCTAssertEqual(next.hand.map(\.id), picture.hand.map(\.id))
    }

    func testAnAcceptShowsTheEngineOrderNotALocalRemoval() {
        let picture = exhibitPicture()
        let king = picture.hand[1]
        let engine = HeldCards(
            hand: [picture.hand[3], picture.hand[0], picture.hand[2]],
            board: picture.board
        )
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king]),
            onto: .newMeld
        ) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(next.hand.map(\.id), [24, 21, 23])
        XCTAssertEqual(next.board.map { $0.map(\.id) }, picture.board.map { $0.map(\.id) })
        XCTAssertNotEqual(next.hand.map(\.id), [21, 23, 24])
    }

    func testAnAcceptThatLeavesTheCardInBothPlacesShowsBoth() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let engine = HeldCards(
            hand: picture.hand,
            board: [picture.board[0] + [eight], picture.board[1]]
        )
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [eight]),
            onto: .meld(0)
        ) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(next.hand.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(next.board[0].map(\.id), [1, 2, 3, 21])
    }

    func testAnAcceptThatOnlyReordersTheHandChangesThePicture() {
        let picture = exhibitPicture()
        let engine = HeldCards(hand: picture.hand.reversed(), board: picture.board)
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [picture.hand[0]]),
            onto: .newMeld
        ) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(next.hand.map(\.id), [24, 23, 22, 21])
        XCTAssertEqual(next.board, picture.board)
        XCTAssertNotEqual(next, picture)
    }

    func testAnIdenticalAcceptLeavesThePictureEqual() {
        let picture = exhibitPicture()
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [picture.hand[1]]),
            onto: .meld(0)
        ) { _ in
            .accepted(HeldCards(hand: picture.hand, board: picture.board))
        }
        XCTAssertEqual(next, picture)
    }

    func testAnAcceptedJokerShowsTheEngineLock() {
        let picture = exhibitPicture()
        let joker = picture.hand[2]
        let unlocked = card(id: joker.id, suit: .none, rank: .joker, locked: 0)
        let stillLocked = card(id: joker.id, suit: .none, rank: .joker, locked: 2)
        let unlockedPicture = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [joker]),
            onto: .newMeld
        ) { _ in
            .accepted(HeldCards(hand: [picture.hand[0]], board: [[unlocked]]))
        }
        XCTAssertEqual(CardFace(unlockedPicture.board[0][0]).spoken, "Joker")
        XCTAssertFalse(CardFace(unlockedPicture.board[0][0]).isLocked)

        let lockedPicture = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [joker]),
            onto: .meld(1)
        ) { _ in
            .accepted(HeldCards(
                hand: [picture.hand[0], picture.hand[1], picture.hand[3]],
                board: [picture.board[0], picture.board[1] + [stillLocked]]
            ))
        }
        XCTAssertEqual(CardFace(lockedPicture.board[1][4]).spoken, "Joker, locked")
        XCTAssertFalse(lockedPicture.hand.map(\.id).contains(joker.id))
    }

    func testAnEmptyDropDoesNotAskTheEngine() {
        let picture = exhibitPicture()
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: []),
            onto: .newMeld
        ) { _ in
            XCTFail("empty drag asked the engine")
            return .accepted(HeldCards(hand: [], board: []))
        }
        XCTAssertEqual(next, picture)
        XCTAssertNil(moveIntent(drag: CardDrag(cards: []), onto: .meld(0), hand: picture.hand))
    }

    func testACardOutsideTheHandDoesNotAskTheEngine() {
        let picture = exhibitPicture()
        let opponent = card(id: 90, suit: .hearts, rank: .ace, locked: 0)
        let boardFour = picture.board[0][0]
        let missedOpponent = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [opponent]),
            onto: .newMeld
        ) { _ in
            XCTFail("an opponent asked the engine")
            return .accepted(HeldCards(hand: [opponent], board: picture.board))
        }
        let missedBoard = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [boardFour]),
            onto: .meld(0)
        ) { _ in
            XCTFail("a board card asked the engine")
            return .gameError
        }
        XCTAssertEqual(missedOpponent, picture)
        XCTAssertEqual(missedBoard, picture)
        XCTAssertFalse(picture.hand.map(\.id).contains(90))
    }

    func testARepeatedIdDoesNotAskTheEngine() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let again = card(id: eight.id, suit: .hearts, rank: .ace, locked: 0)
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [eight, again]),
            onto: .newMeld
        ) { _ in
            XCTFail("a repeated id asked the engine")
            return .refused
        }
        XCTAssertEqual(next, picture)
    }

    func testOneMissingCardDoesNotAskAndTheHandStays() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let king = picture.hand[1]
        let missing = card(id: 99, suit: .clubs, rank: .four, locked: 0)
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king, missing, eight]),
            onto: .newMeld
        ) { _ in
            XCTFail("a partial hand miss asked the engine")
            return .accepted(HeldCards(hand: [], board: [[eight, king]]))
        }
        XCTAssertEqual(next, picture)
        XCTAssertEqual(next.hand.map(\.id), [21, 22, 23, 24])
    }

    func testAnOutOfRangeMeldStillAsksAndARefusalLeavesTheHand() {
        let picture = exhibitPicture()
        let king = picture.hand[1]
        var asked: [DropTarget] = []
        let low = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king]),
            onto: .meld(-1)
        ) { intent in
            asked.append(.meld(-1))
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: -1, cards: [king])]))
            return .refused
        }
        let high = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [king]),
            onto: .meld(Int.max)
        ) { intent in
            asked.append(.meld(Int.max))
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: Int.max, cards: [king])]))
            return .gameError
        }
        XCTAssertEqual(asked.count, 2)
        XCTAssertEqual(low, picture)
        XCTAssertEqual(high, picture)
        XCTAssertEqual(host(picture).screen.dropTargets, [.newMeld, .meld(0), .meld(1)])
    }

    func testASecondDropAfterTheCardLeftDoesNotAsk() {
        let picture = exhibitPicture()
        let eight = picture.hand[0]
        let accepted = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [eight]),
            onto: .meld(0)
        ) { _ in
            .accepted(HeldCards(
                hand: Array(picture.hand.dropFirst()),
                board: [picture.board[0] + [eight], picture.board[1]]
            ))
        }
        let again = applyScreenDrop(
            picture: accepted,
            drag: CardDrag(cards: [eight]),
            onto: .meld(0)
        ) { _ in
            XCTFail("a card that left the hand asked the engine")
            return .accepted(HeldCards(hand: [], board: []))
        }
        XCTAssertEqual(again, accepted)
        XCTAssertFalse(HandView(cards: again.hand).canLift(eight))
    }

    func testTwoCopiesDroppingOneLeavesTheOther() {
        let first = card(id: 7, suit: .hearts, rank: .four, locked: 0)
        let second = card(id: 8, suit: .hearts, rank: .four, locked: 0)
        let picture = TablePicture(
            version: "0.3.3",
            deckSize: 108,
            roundNumber: 2,
            board: [exhibitSet()],
            hand: [first, second]
        )
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [first]),
            onto: .meld(0)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [first])]))
            return .accepted(HeldCards(
                hand: [second],
                board: [picture.board[0] + [first]]
            ))
        }
        XCTAssertEqual(next.hand.map(\.id), [8])
        XCTAssertEqual(CardFace(next.hand[0]).spoken, "Four of Hearts")
        XCTAssertEqual(next.board[0].map(\.id), [1, 2, 3, 7])
        XCTAssertTrue(HandView(cards: next.hand).canLift(second))
        XCTAssertFalse(HandView(cards: next.hand).canLift(first))
    }

    func testTheSameIdTwiceUsesTheFirstHandCard() {
        let first = card(id: 7, suit: .hearts, rank: .four, locked: 3)
        let second = card(id: 7, suit: .spades, rank: .five, locked: 0)
        let picture = TablePicture(
            version: "0.3.3",
            deckSize: 108,
            roundNumber: 2,
            board: [],
            hand: [first, second]
        )
        let forged = card(id: 7, suit: .clubs, rank: .ace, locked: 0)
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [forged]),
            onto: .newMeld
        ) { intent in
            XCTAssertEqual(intent, .playMeld([[first]]))
            return .accepted(HeldCards(hand: [second], board: [[first]]))
        }
        XCTAssertEqual(CardFace(next.board[0][0]).spoken, "Four of Hearts, locked")
        XCTAssertEqual(next.hand.map(\.id), [7])
        XCTAssertEqual(CardFace(next.hand[0]).spoken, "Five of Spades")
    }

    func testARefusalOfTheWildTwoLeavesItInTheHand() {
        let picture = exhibitPicture()
        let two = picture.hand[3]
        let next = applyScreenDrop(
            picture: picture,
            drag: CardDrag(cards: [two]),
            onto: .meld(1)
        ) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [two])]))
            return .refused
        }
        XCTAssertEqual(next.hand[3], two)
        XCTAssertEqual(CardFace(next.hand[3]).spoken, "Two of Clubs")
        XCTAssertEqual(next.board[1].map(\.rank), [.four, .five, .six, .seven])
    }

    func testARefusalDoesNotPublishAndAnAcceptPublishesOnce() {
        let hosted = host(exhibitPicture())
        var signals = 0
        let sub = hosted.table.objectWillChange.sink { signals += 1 }
        let king = hosted.screen.hand.cards[1]

        hosted.screen.drop(CardDrag(cards: [king]), onto: .meld(1)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [king])]))
            return .refused
        }
        XCTAssertEqual(signals, 0)
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(hosted.table.picture.deckSize, 108)

        let eight = hosted.screen.hand.cards[0]
        let engine = HeldCards(
            hand: Array(hosted.table.picture.hand.dropFirst()),
            board: [hosted.table.picture.board[0] + [eight], hosted.table.picture.board[1]]
        )
        hosted.screen.drop(CardDrag(cards: [eight]), onto: .meld(0)) { _ in
            .accepted(engine)
        }
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [22, 23, 24])
        XCTAssertEqual(hosted.screen.board.layout.rows[0].faces.map(\.spoken).last, "Eight of Diamonds")
        XCTAssertEqual(hosted.screen.versionLine, "0.3.3")
        XCTAssertEqual(hosted.screen.deckLine, "Deck size 108")
        XCTAssertEqual(hosted.screen.roundLine, "Round 2")
        XCTAssertTrue(hosted.screen.hand.allowsDrag)
        sub.cancel()
    }

    func testAnInvalidDropDoesNotPublishOrAsk() {
        let hosted = host(exhibitPicture())
        var signals = 0
        let sub = hosted.table.objectWillChange.sink { signals += 1 }
        hosted.screen.drop(CardDrag(cards: []), onto: .newMeld) { _ in
            XCTFail("empty drag asked the engine")
            return .gameError
        }
        let outsider = card(id: 90, suit: .hearts, rank: .ace, locked: 0)
        hosted.screen.drop(CardDrag(cards: [outsider]), onto: .meld(0)) { _ in
            XCTFail("an outsider asked the engine")
            return .accepted(HeldCards(hand: [], board: [[outsider]]))
        }
        XCTAssertEqual(signals, 0)
        XCTAssertEqual(hosted.table.picture, exhibitPicture())
        XCTAssertEqual(hosted.loads.count, 1)
        sub.cancel()
    }

    func testAnIdenticalAcceptDoesNotPublish() {
        let hosted = host(exhibitPicture())
        var signals = 0
        let sub = hosted.table.objectWillChange.sink { signals += 1 }
        let king = hosted.screen.hand.cards[1]
        hosted.screen.drop(CardDrag(cards: [king]), onto: .meld(0)) { _ in
            .accepted(HeldCards(
                hand: hosted.table.picture.hand,
                board: hosted.table.picture.board
            ))
        }
        XCTAssertEqual(signals, 0)
        XCTAssertEqual(hosted.table.picture, exhibitPicture())
        sub.cancel()
    }

    func testADropDoesNotReadTheEngineAgain() {
        let hosted = host(exhibitPicture())
        XCTAssertEqual(hosted.loads.count, 1)
        let eight = hosted.screen.hand.cards[0]
        hosted.screen.drop(CardDrag(cards: [eight]), onto: .meld(0)) { _ in
            .gameError
        }
        hosted.screen.drop(CardDrag(cards: [eight]), onto: .newMeld) { _ in
            .refused
        }
        hosted.screen.drop(CardDrag(cards: [eight]), onto: .meld(0)) { _ in
            .accepted(HeldCards(
                hand: Array(exhibitHand().dropFirst()),
                board: [exhibitSet() + [eight], exhibitRun()]
            ))
        }
        XCTAssertEqual(hosted.loads.count, 1)
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [22, 23, 24])
    }

    func testReloadAfterAnAcceptRestoresTheStandInSnapshot() {
        let hosted = host(exhibitPicture())
        let eight = hosted.screen.hand.cards[0]
        hosted.screen.drop(CardDrag(cards: [eight]), onto: .meld(0)) { _ in
            .accepted(HeldCards(
                hand: Array(exhibitHand().dropFirst()),
                board: [exhibitSet() + [eight], exhibitRun()]
            ))
        }
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [22, 23, 24])
        hosted.table.reload()
        XCTAssertEqual(hosted.loads.count, 2)
        XCTAssertEqual(hosted.table.picture, exhibitPicture())
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(hosted.screen.board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])
        XCTAssertTrue(hosted.screen.hand.canLift(eight))
    }

    func testAnAcceptThenAGameErrorDoesNotRollBack() {
        let hosted = host(exhibitPicture())
        var signals = 0
        let sub = hosted.table.objectWillChange.sink { signals += 1 }
        let eight = hosted.screen.hand.cards[0]
        let afterEight = HeldCards(
            hand: Array(exhibitHand().dropFirst()),
            board: [exhibitSet() + [eight], exhibitRun()]
        )
        hosted.screen.drop(CardDrag(cards: [eight]), onto: .meld(0)) { _ in
            .accepted(afterEight)
        }
        let joker = hosted.screen.hand.cards[1]
        hosted.screen.drop(CardDrag(cards: [joker]), onto: .meld(0)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [joker])]))
            return .gameError
        }
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [22, 23, 24])
        XCTAssertEqual(CardFace(hosted.screen.hand.cards[1]).spoken, "Joker, locked")
        XCTAssertEqual(hosted.screen.board.layout.rows[0].faces.map(\.spoken).last, "Eight of Diamonds")
        XCTAssertEqual(hosted.loads.count, 1)
        sub.cancel()
    }

    func testTheStandInRefusesALockedCardAndMovesAnUnlockedCard() {
        let picture = exhibitPicture()
        let joker = picture.hand[2]
        XCTAssertEqual(
            standInAnswer(
                picture: picture,
                intent: .hitMeld([MeldDrop(meldIndex: 0, cards: [joker])])
            ),
            .refused
        )

        let king = picture.hand[1]
        let accepted = standInAnswer(picture: picture, intent: .playMeld([[king]]))
        guard case .accepted(let table) = accepted else {
            return XCTFail("king was refused")
        }
        XCTAssertEqual(table.hand.map(\.id), [21, 23, 24])
        XCTAssertEqual(table.board.count, 3)
        XCTAssertEqual(CardFace(table.board[2][0]).spoken, "King of Spades")
        XCTAssertEqual(
            standInAnswer(
                picture: picture,
                intent: .hitMeld([MeldDrop(meldIndex: 9, cards: [king])])
            ),
            .refused
        )
        let eight = picture.hand[0]
        XCTAssertEqual(
            standInAnswer(
                picture: picture,
                intent: .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])])
            ),
            .refused
        )
    }

    func testScreenDropUsesTheStandInAndAReloadRestoresTheExhibit() {
        let hosted = host(exhibitPicture())
        let king = hosted.screen.hand.cards[1]
        let changed = hosted.screen.drop(CardDrag(cards: [king]), onto: .newMeld) { intent in
            standInAnswer(picture: hosted.table.picture, intent: intent)
        }
        XCTAssertTrue(changed)
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [21, 23, 24])
        XCTAssertEqual(hosted.screen.board.layout.rows[2].faces.map(\.spoken), ["King of Spades"])
        XCTAssertEqual(hosted.screen.versionLine, "0.3.3")
        XCTAssertEqual(hosted.screen.deckLine, "Deck size 108")
        XCTAssertEqual(hosted.loads.count, 1)

        let joker = hosted.screen.hand.cards[1]
        let stayed = hosted.screen.drop(CardDrag(cards: [joker]), onto: .meld(0)) { intent in
            standInAnswer(picture: hosted.table.picture, intent: intent)
        }
        XCTAssertFalse(stayed)
        XCTAssertEqual(CardFace(hosted.screen.hand.cards[1]).spoken, "Joker, locked")
        XCTAssertEqual(hosted.screen.board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])

        let eight = hosted.screen.hand.cards[0]
        let forged = card(id: eight.id, suit: .spades, rank: .king, locked: 9)
        let hit = hosted.screen.drop(CardDrag(cards: [forged]), onto: .meld(0)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])]))
            return standInAnswer(picture: hosted.table.picture, intent: intent)
        }
        XCTAssertFalse(hit)
        XCTAssertEqual(hosted.screen.hand.cards.map(\.id), [21, 23, 24])
        XCTAssertEqual(hosted.screen.board.layout.rows[0].faces.map(\.pip), ["4", "4", "4"])

        hosted.table.reload()
        XCTAssertEqual(hosted.loads.count, 2)
        XCTAssertEqual(hosted.table.picture, exhibitPicture())
        XCTAssertTrue(hosted.screen.hand.canLift(eight))
    }

    /// Chain: empty published shoe → board rows → local hand → screen drop.
    /// A refusal leaves the card. An accept updates the picture. A later
    /// `GameError` does not roll back. The stand-in load is not written.
    func testPublishedShoeBoardRowsLocalHandThenScreenDropAcceptsAndARefusalLeavesTheCard() {
        var source = TablePicture(
            version: "0.3.3",
            deckSize: 108,
            roundNumber: 1,
            board: [],
            hand: []
        )
        let loads = LoadCounter()
        let table = PublishedTable {
            loads.count += 1
            return source
        }
        let screen = GameBoardScreen(table: table)
        XCTAssertEqual(loads.count, 1)
        XCTAssertEqual(screen.versionLine, "0.3.3")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 1")
        XCTAssertEqual(screen.board.emptyTitle, "No melds")
        XCTAssertEqual(screen.hand.emptyTitle, "No cards")
        XCTAssertFalse(screen.hand.allowsDrag)
        XCTAssertEqual(screen.dropTargets, [.newMeld])

        var signals = 0
        let sub = table.objectWillChange.sink { signals += 1 }
        let stranger = card(id: 90, suit: .hearts, rank: .ace, locked: 0)
        screen.drop(CardDrag(cards: [stranger]), onto: .newMeld) { _ in
            XCTFail("a card outside an empty hand asked the engine")
            return .accepted(HeldCards(hand: [], board: [[stranger]]))
        }
        XCTAssertEqual(signals, 0)
        XCTAssertEqual(loads.count, 1)
        XCTAssertTrue(screen.hand.cards.isEmpty)

        source = exhibitPicture()
        table.reload()
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(loads.count, 2)
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.versionLine, "0.3.3")
        XCTAssertNil(screen.board.emptyTitle)
        XCTAssertNil(screen.hand.emptyTitle)
        XCTAssertTrue(screen.hand.allowsDrag)
        XCTAssertEqual(screen.dropTargets, [.newMeld, .meld(0), .meld(1)])
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
        XCTAssertFalse(screen.hand.canLift(stranger))
        XCTAssertFalse(screen.hand.canLift(screen.board.melds[0][0]))
        XCTAssertTrue(screen.hand.cards.allSatisfy { screen.hand.canLift($0) })
        let handIds = Set(screen.hand.cards.map(\.id))
        let boardIds = Set(screen.board.melds.flatMap { $0 }.map(\.id))
        XCTAssertTrue(handIds.isDisjoint(with: boardIds))
        XCTAssertFalse(handIds.contains(stranger.id))

        let king = screen.hand.cards[1]
        screen.drop(CardDrag(cards: [king]), onto: .meld(1)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [king])]))
            return .refused
        }
        XCTAssertEqual(signals, 1)
        XCTAssertEqual(screen.hand.cards.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(CardFace(screen.hand.cards[1]).spoken, "King of Spades")
        XCTAssertEqual(screen.board.layout.rows[1].faces.map(\.pip), ["4", "5", "6", "7"])

        let eight = screen.hand.cards[0]
        let forgedEight = card(id: eight.id, suit: .spades, rank: .king, locked: 9)
        let afterEight = HeldCards(
            hand: [screen.hand.cards[1], screen.hand.cards[2], screen.hand.cards[3]],
            board: [screen.board.melds[0] + [eight], screen.board.melds[1]]
        )
        screen.drop(CardDrag(cards: [forgedEight]), onto: .meld(0)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [eight])]))
            return .accepted(afterEight)
        }
        XCTAssertEqual(signals, 2)
        XCTAssertEqual(screen.hand.cards.map(\.id), [22, 23, 24])
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.spoken).last, "Eight of Diamonds")
        XCTAssertEqual(CardView(card: screen.board.melds[0][3]).face, CardFace(eight))
        XCTAssertEqual(screen.versionLine, "0.3.3")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertTrue(screen.hand.allowsDrag)
        XCTAssertFalse(screen.hand.canLift(eight))
        XCTAssertFalse(screen.hand.cards.map(\.id).contains(stranger.id))

        screen.drop(CardDrag(cards: [eight]), onto: .meld(0)) { _ in
            XCTFail("the eight already left the hand")
            return .accepted(HeldCards(hand: [], board: []))
        }
        XCTAssertEqual(signals, 2)
        XCTAssertEqual(screen.hand.cards.map(\.id), [22, 23, 24])

        let joker = screen.hand.cards[1]
        screen.drop(CardDrag(cards: [joker]), onto: .meld(0)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [joker])]))
            return .gameError
        }
        XCTAssertEqual(signals, 2)
        XCTAssertEqual(CardFace(screen.hand.cards[1]).spoken, "Joker, locked")
        XCTAssertEqual(screen.hand.cards[1].lockedUntilTurn, 2)
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.spoken).last, "Eight of Diamonds")

        let two = screen.hand.cards[2]
        let afterTwo = HeldCards(
            hand: [screen.hand.cards[0], screen.hand.cards[1]],
            board: [screen.board.melds[0], screen.board.melds[1] + [two]]
        )
        screen.drop(CardDrag(cards: [two]), onto: .meld(1)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 1, cards: [two])]))
            return .accepted(afterTwo)
        }
        XCTAssertEqual(signals, 3)
        XCTAssertEqual(screen.hand.cards.map(\.id), [22, 23])
        XCTAssertEqual(CardFace(screen.board.melds[1].last!).spoken, "Two of Clubs")

        let kingNow = screen.hand.cards[0]
        screen.drop(CardDrag(cards: [kingNow]), onto: .meld(0)) { intent in
            XCTAssertEqual(intent, .hitMeld([MeldDrop(meldIndex: 0, cards: [kingNow])]))
            return .refused
        }
        XCTAssertEqual(signals, 3)
        XCTAssertEqual(screen.hand.cards.map(\.id), [22, 23])
        XCTAssertEqual(CardFace(screen.board.melds[1].last!).spoken, "Two of Clubs")
        XCTAssertEqual(screen.board.layout.rows[0].faces.map(\.spoken).last, "Eight of Diamonds")

        let played = HeldCards(
            hand: [screen.hand.cards[1]],
            board: [screen.board.melds[0], screen.board.melds[1], [kingNow]]
        )
        screen.drop(CardDrag(cards: [kingNow]), onto: .newMeld) { intent in
            XCTAssertEqual(intent, .playMeld([[kingNow]]))
            return .accepted(played)
        }
        XCTAssertEqual(signals, 4)
        XCTAssertEqual(screen.hand.cards.map(\.id), [23])
        XCTAssertEqual(CardFace(screen.hand.cards[0]).spoken, "Joker, locked")
        XCTAssertTrue(screen.hand.allowsDrag)
        XCTAssertEqual(screen.dropTargets, [.newMeld, .meld(0), .meld(1), .meld(2)])
        XCTAssertEqual(screen.board.layout.rows[2].faces.map(\.spoken), ["King of Spades"])
        XCTAssertEqual(screen.versionLine, "0.3.3")
        XCTAssertEqual(screen.deckLine, "Deck size 108")
        XCTAssertEqual(screen.roundLine, "Round 2")
        XCTAssertEqual(loads.count, 2)

        table.reload()
        XCTAssertEqual(loads.count, 3)
        XCTAssertEqual(signals, 5)
        XCTAssertEqual(table.picture, exhibitPicture())
        XCTAssertEqual(screen.hand.cards.map(\.id), [21, 22, 23, 24])
        XCTAssertEqual(screen.board.layout.rows.count, 2)
        XCTAssertTrue(screen.hand.canLift(eight))
        XCTAssertFalse(screen.hand.cards.map(\.id).contains(stranger.id))
        sub.cancel()
    }
}

private final class LoadCounter {
    var count = 0
}

private func host(
    _ picture: TablePicture
) -> (table: PublishedTable, screen: GameBoardScreen, loads: LoadCounter) {
    let loads = LoadCounter()
    let table = PublishedTable {
        loads.count += 1
        return picture
    }
    return (table, GameBoardScreen(table: table), loads)
}

private func exhibitPicture() -> TablePicture {
    TablePicture(
        version: "0.3.3",
        deckSize: 108,
        roundNumber: 2,
        board: [exhibitSet(), exhibitRun()],
        hand: exhibitHand()
    )
}

private func exhibitHand() -> [BoardCard] {
    [
        card(id: 21, suit: .diamonds, rank: .eight, locked: 0),
        card(id: 22, suit: .spades, rank: .king, locked: 0),
        card(id: 23, suit: .none, rank: .joker, locked: 2),
        card(id: 24, suit: .clubs, rank: .two, locked: 0),
    ]
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
