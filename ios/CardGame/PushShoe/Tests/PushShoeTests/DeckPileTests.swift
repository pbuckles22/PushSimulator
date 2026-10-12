import PushFfi
import PushShoe
import XCTest

final class DeckPileTests: XCTestCase {
    /// The pile shows the digits of the count. It does not use the retired label.
    func testDeckPileTextIsTheDigitsOfTheUndealtCount() {
        XCTAssertEqual(DeckPile.text(undealt: 108), "108")
        XCTAssertEqual(DeckPile.text(undealt: 87), "87")
        XCTAssertEqual(DeckPile.text(undealt: 0), "0")
        XCTAssertNotEqual(DeckPile.text(undealt: 108), "Deck size 108")
    }

    /// One natural rank and suit uses the symbols, image, and raw values their face view draws.
    func testOneNaturalRankAndSuitDrawsTheirFace() {
        let fourOfHearts = CardSnapshot(id: 1, suit: .hearts, rank: .four, lockedUntilTurn: 0)
        let face = NaturalFace.draw(fourOfHearts)

        XCTAssertEqual(face?.rankSymbol, "4")
        XCTAssertEqual(face?.suitSymbol, "♥️")
        XCTAssertEqual(face?.systemImageName, "suit.heart.fill")
        XCTAssertEqual(face?.isRed, true)
        XCTAssertEqual(face?.rankRaw, 4)
        XCTAssertEqual(face?.suitRaw, 2)
        XCTAssertEqual(face?.spoken, "4♥️")
        XCTAssertNotEqual(face?.spoken, "4♥")
    }

    /// A joker has no natural face. Their joker face is a separate drawing.
    func testJokerDoesNotDrawTheirNaturalFace() {
        let joker = CardSnapshot(id: 23, suit: .none, rank: .joker, lockedUntilTurn: 2)
        let suitedJoker = CardSnapshot(id: 23, suit: .hearts, rank: .joker, lockedUntilTurn: 0)

        XCTAssertNil(NaturalFace.draw(joker))
        XCTAssertNil(NaturalFace.draw(suitedJoker))
    }

    /// A joker draws the symbols their face view uses. A suit on that card does not become a heart.
    func testAJokerDrawsTheirFace() {
        let joker = CardSnapshot(id: 23, suit: .none, rank: .joker, lockedUntilTurn: 2)
        let suited = CardSnapshot(id: 23, suit: .hearts, rank: .joker, lockedUntilTurn: 0)
        let unlocked = CardSnapshot(id: 99, suit: .none, rank: .joker, lockedUntilTurn: 0)
        let face = JokerFace.draw(joker)

        XCTAssertEqual(face?.rankSymbol, "Joker")
        XCTAssertEqual(face?.suitSymbol, "")
        XCTAssertEqual(face?.systemImageName, "star.fill")
        XCTAssertEqual(face?.isRed, false)
        XCTAssertEqual(face?.spoken, "Joker")
        XCTAssertNotEqual(face?.spoken, "Joker wild")
        XCTAssertNotEqual(face?.spoken, "Joker, locked")
        XCTAssertFalse(face?.systemImageName.hasPrefix("suit.") ?? true)
        XCTAssertEqual(JokerFace.draw(suited), face)
        XCTAssertEqual(JokerFace.draw(unlocked), face)
        XCTAssertNotEqual(face?.spoken, "4♥️")
    }

    /// A natural rank and suit is not a joker face.
    func testANaturalCardDoesNotDrawTheJokerFace() {
        let four = CardSnapshot(id: 1, suit: .hearts, rank: .four, lockedUntilTurn: 0)
        let two = CardSnapshot(id: 24, suit: .clubs, rank: .two, lockedUntilTurn: 0)

        XCTAssertNil(JokerFace.draw(four))
        XCTAssertNil(JokerFace.draw(two))
        XCTAssertNil(JokerFace.one(in: Game()))
    }

    /// A missing suit is not a natural rank and suit.
    func testSuitNoneDoesNotDrawTheirFace() {
        let noSuit = CardSnapshot(id: 1, suit: .none, rank: .four, lockedUntilTurn: 0)
        XCTAssertNil(NaturalFace.draw(noSuit))
    }

    /// The mapper is not a constant four of hearts. Each suit keeps their image and color.
    func testEachNaturalSuitDrawsTheirFaceImage() {
        let clubs = NaturalFace.draw(CardSnapshot(id: 3, suit: .clubs, rank: .four, lockedUntilTurn: 0))
        let diamonds = NaturalFace.draw(CardSnapshot(id: 21, suit: .diamonds, rank: .eight, lockedUntilTurn: 0))
        let spades = NaturalFace.draw(CardSnapshot(id: 2, suit: .spades, rank: .four, lockedUntilTurn: 0))
        let ten = NaturalFace.draw(CardSnapshot(id: 30, suit: .spades, rank: .ten, lockedUntilTurn: 0))
        let ace = NaturalFace.draw(CardSnapshot(id: 31, suit: .clubs, rank: .ace, lockedUntilTurn: 0))

        XCTAssertEqual(clubs?.suitSymbol, "♣️")
        XCTAssertEqual(clubs?.systemImageName, "suit.club.fill")
        XCTAssertEqual(clubs?.isRed, false)
        XCTAssertEqual(clubs?.suitRaw, 0)
        XCTAssertEqual(diamonds?.rankSymbol, "8")
        XCTAssertEqual(diamonds?.suitSymbol, "♦️")
        XCTAssertEqual(diamonds?.systemImageName, "suit.diamond.fill")
        XCTAssertEqual(diamonds?.isRed, true)
        XCTAssertEqual(diamonds?.rankRaw, 8)
        XCTAssertEqual(diamonds?.suitRaw, 1)
        XCTAssertEqual(spades?.suitSymbol, "♠️")
        XCTAssertEqual(spades?.systemImageName, "suit.spade.fill")
        XCTAssertEqual(spades?.isRed, false)
        XCTAssertEqual(spades?.suitRaw, 3)
        XCTAssertEqual(ten?.rankSymbol, "10")
        XCTAssertEqual(ten?.rankRaw, 10)
        XCTAssertEqual(ace?.rankSymbol, "A")
        XCTAssertEqual(ace?.rankRaw, 1)
    }

    /// Face ranks use their letters. A two keeps their face and has no wild mark.
    func testFaceRanksAndATwoDrawTheirFaceWithoutAWildMark() {
        let jack = NaturalFace.draw(CardSnapshot(id: 40, suit: .hearts, rank: .jack, lockedUntilTurn: 0))
        let queen = NaturalFace.draw(CardSnapshot(id: 41, suit: .diamonds, rank: .queen, lockedUntilTurn: 0))
        let king = NaturalFace.draw(CardSnapshot(id: 22, suit: .spades, rank: .king, lockedUntilTurn: 0))
        let two = NaturalFace.draw(CardSnapshot(id: 24, suit: .clubs, rank: .two, lockedUntilTurn: 0))

        XCTAssertEqual(jack?.rankSymbol, "J")
        XCTAssertEqual(jack?.rankRaw, 11)
        XCTAssertEqual(queen?.rankSymbol, "Q")
        XCTAssertEqual(queen?.rankRaw, 12)
        XCTAssertEqual(king?.rankSymbol, "K")
        XCTAssertEqual(king?.rankRaw, 13)
        XCTAssertEqual(king?.suitSymbol, "♠️")
        XCTAssertEqual(two?.rankSymbol, "2")
        XCTAssertEqual(two?.rankRaw, 2)
        XCTAssertEqual(two?.suitSymbol, "♣️")
        XCTAssertEqual(two?.spoken, "2♣️")
        XCTAssertNotEqual(two?.spoken, "2♣ wild")
    }

    /// Chain: `Deck::new` → UniFFI `getDeckSize` → exhibit shoe → deck pile text.
    func testDeckNewGetDeckSizeExhibitShoeDeckPileShows108() {
        let fresh = Game().getDeckSize()
        let again = Game().getDeckSize()
        let exhibit = Game.exhibitSetAndRun().getDeckSize()

        XCTAssertEqual(fresh, 108)
        XCTAssertEqual(again, fresh)
        XCTAssertEqual(exhibit, fresh)
        XCTAssertNotEqual(fresh, 87)
        XCTAssertEqual(UndealtShoe.count(), fresh)
        XCTAssertEqual(DeckPileCount.shown(), DeckPile.text(undealt: fresh))
        XCTAssertEqual(DeckPileCount.shown(), "108")
        XCTAssertNotEqual(DeckPileCount.shown(), "Deck size 108")
    }

    /// Chain: the 3.2 shoe, then the exhibit's first natural rank and suit drawn as their face.
    func testDeckNewGetDeckSizeExhibitShoeDeckPileShows108AndOneNaturalRankAndSuitDrawsTheirFace() {
        let fresh = Game()
        let exhibit = Game.exhibitSetAndRun()
        let table = exhibit.tableSnapshot()
        let card = table.board[0].cards[0]
        let face = NaturalFace.draw(card)

        XCTAssertEqual(fresh.getDeckSize(), 108)
        XCTAssertEqual(exhibit.getDeckSize(), 108)
        XCTAssertEqual(UndealtShoe.count(), 108)
        XCTAssertEqual(DeckPileCount.shown(), "108")
        XCTAssertNil(NaturalFace.one(in: fresh))
        XCTAssertEqual(card.id, 1)
        XCTAssertEqual(card.rank, .four)
        XCTAssertEqual(card.suit, .hearts)
        XCTAssertNotEqual(card.rank, .joker)
        XCTAssertEqual(NaturalFace.one(in: exhibit), face)
        XCTAssertEqual(NaturalFace.shown(), face)
        XCTAssertEqual(face?.spoken, "4♥️")
        XCTAssertEqual(face?.systemImageName, "suit.heart.fill")
        XCTAssertNotEqual(face, NaturalFace.draw(table.hand[0]))
        XCTAssertEqual(table.hand[2].rank, .joker)
        XCTAssertNil(NaturalFace.draw(table.hand[2]))
    }

    /// Chain: the 3.2 shoe, the 3.3 four of hearts, then the exhibit joker drawn with their face.
    func testDeckNewGetDeckSizeExhibitShoeDeckPileShows108AndOneNaturalRankAndSuitDrawsTheirFaceAndAJokerDrawsTheirFace() {
        let fresh = Game()
        let exhibit = Game.exhibitSetAndRun()
        let table = exhibit.tableSnapshot()
        let four = table.board[0].cards[0]
        let jokerCard = table.hand[2]
        let face = NaturalFace.draw(four)
        let joker = JokerFace.draw(jokerCard)

        XCTAssertEqual(fresh.getDeckSize(), 108)
        XCTAssertEqual(exhibit.getDeckSize(), 108)
        XCTAssertEqual(UndealtShoe.count(), 108)
        XCTAssertEqual(DeckPileCount.shown(), "108")
        XCTAssertNil(NaturalFace.one(in: fresh))
        XCTAssertNil(JokerFace.one(in: fresh))
        XCTAssertEqual(four.id, 1)
        XCTAssertEqual(four.rank, .four)
        XCTAssertEqual(four.suit, .hearts)
        XCTAssertEqual(NaturalFace.one(in: exhibit), face)
        XCTAssertEqual(NaturalFace.shown(), face)
        XCTAssertEqual(face?.spoken, "4♥️")
        XCTAssertEqual(face?.systemImageName, "suit.heart.fill")
        XCTAssertEqual(jokerCard.id, 23)
        XCTAssertEqual(jokerCard.suit, .none)
        XCTAssertEqual(jokerCard.rank, .joker)
        XCTAssertEqual(jokerCard.lockedUntilTurn, 2)
        XCTAssertNil(NaturalFace.draw(jokerCard))
        XCTAssertNil(JokerFace.draw(four))
        XCTAssertEqual(JokerFace.one(in: exhibit), joker)
        XCTAssertEqual(JokerFace.shown(), joker)
        XCTAssertEqual(joker?.rankSymbol, "Joker")
        XCTAssertEqual(joker?.suitSymbol, "")
        XCTAssertEqual(joker?.systemImageName, "star.fill")
        XCTAssertEqual(joker?.isRed, false)
        XCTAssertEqual(joker?.spoken, "Joker")
        XCTAssertNotEqual(joker?.spoken, face?.spoken)
        XCTAssertNotEqual(joker?.spoken, "Joker wild")
        XCTAssertNotEqual(joker?.spoken, "Joker, locked")
    }
}
