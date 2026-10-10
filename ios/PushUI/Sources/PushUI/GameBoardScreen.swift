import SwiftUI
import UniformTypeIdentifiers

/// Version, shoe size, round, the board, and the local hand. The picture is the published table.
public struct GameBoardScreen: View {
    @ObservedObject public var table: PublishedTable
    private let engine: ((MoveIntent) -> DropAnswer)?

    public init(
        table: PublishedTable,
        engine: ((MoveIntent) -> DropAnswer)? = nil
    ) {
        self.table = table
        self.engine = engine
    }

    private func answer(_ intent: MoveIntent) -> DropAnswer {
        if let engine {
            return engine(intent)
        }
        return standInAnswer(picture: table.picture, intent: intent)
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
        BoardView(melds: table.picture.board) { drag, index in
            drop(drag, onto: .meld(index)) { intent in
                answer(intent)
            }
        }
    }

    public var hand: HandView {
        HandView(cards: table.picture.hand)
    }

    /// The new-meld zone, then one zone for each row. An empty meld is still a zone.
    public var dropTargets: [DropTarget] {
        [.newMeld] + table.picture.board.indices.map { .meld($0) }
    }

    /// Asks the engine through `settleDrop`. The picture changes only on an accept
    /// that returns a different hand or board. Returns whether the picture changed.
    @discardableResult
    public func drop(
        _ drag: CardDrag,
        onto target: DropTarget,
        engine: (MoveIntent) -> DropAnswer
    ) -> Bool {
        let next = applyScreenDrop(
            picture: table.picture,
            drag: drag,
            onto: target,
            engine: engine
        )
        let changed = next != table.picture
        table.show(next)
        return changed
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(versionLine)
                .accessibilityIdentifier("version")
            Text(deckLine)
                .accessibilityIdentifier("deck-size")
            Text(roundLine)
                .accessibilityIdentifier("round")
            NewMeldZone { drag in
                drop(drag, onto: .newMeld) { intent in
                    answer(intent)
                }
            }
            board
            Spacer(minLength: 16)
            hand
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(Color.white)
    }
}

/// The new-meld target. Its highlight state stays here, so a drag across it
/// does not rebuild the hand.
private struct NewMeldZone: View {
    let onDrop: (CardDrag) -> Bool
    @State private var targeted = false

    var body: some View {
        Text("New meld")
            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            .padding(.horizontal, 8)
            .background(targeted ? Color(white: 0.9) : Color.white)
            .contentShape(Rectangle())
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.black, lineWidth: targeted ? 2 : 1)
            )
            .accessibilityIdentifier("new-meld")
            .onDrop(of: [.json], isTargeted: $targeted) { providers in
                acceptLiftedCards(providers, perform: onDrop)
            }
    }
}
