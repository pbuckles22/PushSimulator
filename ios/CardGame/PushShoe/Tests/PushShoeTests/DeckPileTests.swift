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
}
