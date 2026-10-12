/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
Chain: their solitaire launches → the screen version → the deck pile shows
 the undealt shoe. The retired "Deck size 108" line stays absent.
*/

import XCTest

final class LaunchVersionUITests: XCTestCase {
    func testLaunchShowsSolitaireVersionWithNoPush() throws {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))

        let window = app.windows.firstMatch
        XCTAssertEqual(window.title, "Solitaire")

        XCTAssertTrue(app.staticTexts["0.3.11"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.staticTexts["Deck size 108"].exists)

        let newGame = app.buttons["New"]
        XCTAssertTrue(newGame.waitForExistence(timeout: 5))
        newGame.click()

        let pile = app.descendants(matching: .any)["deck-pile"]
        XCTAssertTrue(pile.waitForExistence(timeout: 5))
        XCTAssertEqual(pile.label, "108")
        XCTAssertFalse(app.staticTexts["Deck size 108"].exists)
    }
}
