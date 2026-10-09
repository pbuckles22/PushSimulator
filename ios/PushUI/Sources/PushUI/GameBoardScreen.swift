import SwiftUI

/// Version, shoe size, round, the board, and the local hand. The picture is the published table.
public struct GameBoardScreen: View {
    @ObservedObject public var table: PublishedTable

    public init(table: PublishedTable) {
        self.table = table
    }

    public var versionLine: String {
        table.picture.version
    }

    public var deckLine: String {
        table.deckSizeText
    }

    public var roundLine: String {
        "Round \(table.picture.roundNumber)"
    }

    public var board: BoardView {
        BoardView(melds: table.picture.board)
    }

    public var hand: HandView {
        HandView(cards: table.picture.hand)
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(versionLine)
                .accessibilityIdentifier("version")
            Text(deckLine)
                .accessibilityIdentifier("deck-size")
            Text(roundLine)
                .accessibilityIdentifier("round")
            board
            Spacer(minLength: 16)
            hand
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(Color.white)
    }
}
