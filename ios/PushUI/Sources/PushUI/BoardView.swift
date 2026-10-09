import SwiftUI

/// One row per meld, cards left to right in the order they were given.
public struct BoardLayout: Equatable {
    public struct Row: Equatable {
        public var faces: [CardFace]
    }

    public var rows: [Row]

    public init(melds: [[BoardCard]]) {
        rows = melds.map { meld in
            Row(faces: meld.map { CardFace($0) })
        }
    }
}

/// Sets and runs, each meld on its own row. An empty board says so.
/// `onDrop` receives the drag and the row index. Nil leaves the rows as pictures.
public struct BoardView: View {
    public let melds: [[BoardCard]]
    public var onDrop: ((CardDrag, Int) -> Bool)?

    public init(melds: [[BoardCard]], onDrop: ((CardDrag, Int) -> Bool)? = nil) {
        self.melds = melds
        self.onDrop = onDrop
    }

    public var layout: BoardLayout {
        BoardLayout(melds: melds)
    }

    /// Set when there are no melds. An empty meld is still a row.
    public var emptyTitle: String? {
        melds.isEmpty ? "No melds" : nil
    }

    public var body: some View {
        if let emptyTitle {
            Text(emptyTitle)
                .accessibilityIdentifier("empty-board")
        } else {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(Array(melds.enumerated()), id: \.offset) { index, meld in
                    HStack(spacing: 8) {
                        ForEach(Array(meld.enumerated()), id: \.offset) { _, card in
                            CardView(card: card)
                        }
                    }
                    .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
                    .contentShape(Rectangle())
                    .dropDestination(for: CardDrag.self) { items, _ in
                        guard let drag = items.first, let onDrop else {
                            return false
                        }
                        return onDrop(drag, index)
                    }
                }
            }
            .accessibilityIdentifier("board")
        }
    }
}
