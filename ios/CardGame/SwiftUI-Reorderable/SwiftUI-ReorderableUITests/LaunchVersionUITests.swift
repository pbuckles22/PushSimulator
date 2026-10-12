/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
Story 3.1. The sample launches as their solitaire, and the screen shows 0.3.10.
*/

import XCTest

final class LaunchVersionUITests: XCTestCase {
    func testLaunchShowsSolitaireVersionWithNoPush() throws {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))

        let window = app.windows.firstMatch
        XCTAssertEqual(window.title, "Solitaire")

        XCTAssertTrue(app.staticTexts["0.3.10"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.staticTexts["Deck size 108"].exists)
    }
}
