/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A view that shows a pile of cards and a layout that adds enough offset to
 see the value of each card.
*/

import SwiftUI
import SwiftData

struct PileView: View {
    var game: Game
    var index: Int
    @Query var cards: [Card]

    init(game: Game, index: Int) {
        self.game = game
        self.index = index
        let id = game.persistentModelID
        let group = Card.Group.pile(index).intValue
        _cards = Query(
            filter: #Predicate<Card> {
                $0.game.persistentModelID == id && $0.group == group
                    && $0.groupIndex == index
            },
            sort: \.position)
    }

    var body: some View {
        ZStack(alignment: .topLeading) {
            CardPlaceholderView()
            PileLayout {
                let index = firstFaceUpIndex
                ForEach(cards[..<index]) { card in
                    CardView(card: card)
                }
                ForEach(cards[index...], id: \.value) { card in
                    CardView(card: card)
                }
                .reorderable(collectionID: Card.Group.pile(index))
            }
        }
    }

    var firstFaceUpIndex: Int {
        cards.firstIndex { !$0.isFaceDown } ?? cards.endIndex
    }
}

// MARK: - Stacking Alignment Guide

extension VerticalAlignment {
    private enum StackingGuide: AlignmentID {
        static func defaultValue(in dimensions: ViewDimensions) -> CGFloat {
            dimensions[.top]
        }
    }

    /// Alignment guide indicating where to place the top of the next card
    /// when building a fanned stack.
    static let stackingGuide = VerticalAlignment(StackingGuide.self)
}

extension Alignment {
    static let stackingGuide
        = Alignment(horizontal: .center, vertical: .stackingGuide)
}

// MARK: - Pile Layout

private struct PileLayout: Layout {
    struct Cache {
        var size: CGSize = .zero
        var offsets: [CGFloat] = []
    }

    func sizeThatFits(
        proposal: ProposedViewSize, subviews: Subviews, cache: inout Cache
    ) -> CGSize { cache.size }

    func placeSubviews(
        in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews,
        cache: inout Cache
    ) {
        for index in 0..<subviews.count {
            subviews[index].place(
                at: CGPoint(
                    x: bounds.origin.x,
                    y: bounds.origin.y + cache.offsets[index]),
                proposal: proposal)
        }
    }

    func makeCache(subviews: Subviews) -> Cache {
        guard !subviews.isEmpty else { return .init() }

        var cache = Cache()
        var nextOffset = 0.0
        var lastSize = CGSize.zero

        for index in 0..<subviews.count {
            let subview = subviews[index]
            let dimensions = subview.dimensions(in: .unspecified)
            defer { nextOffset += dimensions[.stackingGuide] }
            lastSize = subview.sizeThatFits(.unspecified)

            cache.offsets.append(nextOffset)
            cache.size.width = max(lastSize.width, cache.size.width)
            cache.size.height = nextOffset
        }

        cache.size.height += lastSize.height
        return cache
    }
}

#Preview {
    PileLayout {
        ForEach(CardValue.Suit.diamonds.cards.reversed()) { card in
            CardFaceView(card: card)
        }
        .frame(width: 180)
    }
    .frame(width: 300, height: 600, alignment: .top)
    .padding()
    .background(Color.green)
}
