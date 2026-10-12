/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
Chain: their solitaire launches → the screen version → the deck pile shows
 the undealt shoe → one natural rank and suit draws with their face →
 a joker draws with their face → a two and that joker show a wild mark →
 the discard top draws with their face.
 The retired "Deck size 108" line stays absent.
*/

import XCTest

final class LaunchVersionUITests: XCTestCase {
    func testLaunchShowsSolitaireVersionWithNoPush() throws {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))

        let window = app.windows.firstMatch
        XCTAssertEqual(window.title, "Solitaire")

        XCTAssertTrue(app.staticTexts["0.3.15"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.staticTexts["0.3.14"].exists)
        XCTAssertFalse(app.staticTexts["Deck size 108"].exists)

        let newGame = app.buttons["New"]
        XCTAssertTrue(newGame.waitForExistence(timeout: 5))
        newGame.click()

        let pile = app.descendants(matching: .any)["deck-pile"]
        XCTAssertTrue(pile.waitForExistence(timeout: 5))
        XCTAssertEqual(pile.label, "108")
        XCTAssertFalse(app.staticTexts["Deck size 108"].exists)

        let face = app.descendants(matching: .any)["natural-face"]
        XCTAssertTrue(face.waitForExistence(timeout: 5))
        XCTAssertEqual(face.label, "4♥️")

        let joker = app.descendants(matching: .any)["joker-face"]
        XCTAssertTrue(joker.waitForExistence(timeout: 5))
        XCTAssertEqual(joker.label, "Joker")
        XCTAssertFalse(app.staticTexts["Joker wild"].exists)
        XCTAssertFalse(app.staticTexts["Joker, locked"].exists)

        let jokerMark = app.descendants(matching: .any)["joker-wild-mark"]
        XCTAssertTrue(jokerMark.waitForExistence(timeout: 5))
        XCTAssertEqual(jokerMark.label, "wild")

        let two = app.descendants(matching: .any)["two-face"]
        XCTAssertTrue(two.waitForExistence(timeout: 5))
        XCTAssertEqual(two.label, "2♣️")

        let twoMark = app.descendants(matching: .any)["two-wild-mark"]
        XCTAssertTrue(twoMark.waitForExistence(timeout: 5))
        XCTAssertEqual(twoMark.label, "wild")
        XCTAssertFalse(app.descendants(matching: .any)["natural-wild-mark"].exists)

        let discard = app.descendants(matching: .any)["discard-face"]
        XCTAssertTrue(discard.waitForExistence(timeout: 5))
        XCTAssertEqual(discard.label, "Q♦️")
        XCTAssertFalse(app.descendants(matching: .any)["discard-wild-mark"].exists)
    }
}
