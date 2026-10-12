/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A view that shows the current state of a card instance, as well as the
 component views used to show the front, back, and placeholder
 representations of a card.
*/

import PushShoe
import SwiftUI

struct CardView: View {
    var card: Card

    var body: some View {
        CardFaceView(card: card.value)
            .overlay {
                CardBackView()
                    .opacity(card.isFaceDown ? 1 : 0)
            }
    }
}

struct CardFaceView: View {
    private let rankSymbol: String
    private let suitSymbol: String
    private let systemImageName: String
    private let tint: Color

    init(card: CardValue) {
        rankSymbol = card.rank.symbol
        suitSymbol = card.suit.symbol
        systemImageName = card.suit.systemImageName
        tint = card.suit.color
    }

    init(joker face: JokerFace) {
        rankSymbol = face.rankSymbol
        suitSymbol = face.suitSymbol
        systemImageName = face.systemImageName
        tint = face.isRed ? .red : .black
    }

    var body: some View {
        ZStack {
            // Background color
            Color.white
            // Suit image, on its own layer so that it's centered.
            Image(systemName: systemImageName)
                .resizable()
                .scaledToFit()
                .foregroundStyle(tint.gradient)
                .padding()
            // The header with the rank and suit emoji.
            VStack {
                HStack {
                    Text(verbatim: rankSymbol)
                    Spacer()
                    Text(verbatim: suitSymbol)
                }
                .alignmentGuide(.stackingGuide) { $0[.top] + $0.height }
                Spacer()
            }
            .font(.system(size: 18, weight: .bold))
            .padding(4)
            // The border for the card.
            RoundedRectangle(cornerRadius: 8)
                .strokeBorder(Color.black.opacity(0.2), lineWidth: 1)
        }
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .aspectRatio(0.7, contentMode: .fit)
        .foregroundStyle(tint)
    }
}

/// The word "wild" at the bottom of a two or a joker. The face underneath stays theirs.
struct WildMarkBadge: View {
    var spoken: String
    var identifier: String

    var body: some View {
        VStack {
            Spacer()
            Text(verbatim: spoken)
                .font(.system(size: 12, weight: .heavy))
                .foregroundStyle(.black)
                .padding(.horizontal, 5)
                .padding(.vertical, 1)
                .background(Color.white, in: Capsule())
                .overlay {
                    Capsule().strokeBorder(Color.black, lineWidth: 1)
                }
                .padding(.bottom, 6)
        }
        .accessibilityElement(children: .ignore)
        .accessibilityIdentifier(identifier)
        .accessibilityLabel(spoken)
        .allowsHitTesting(false)
    }
}

struct CardBackView: View {
    var body: some View {
        RoundedRectangle(cornerRadius: 8)
            .fill(.blue)
            .strokeBorder(Color.cyan, style: StrokeStyle())
            .aspectRatio(0.7, contentMode: .fit)
    }
}

struct CardPlaceholderView: View {
    var body: some View {
        Color.black
            .opacity(0.3)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .aspectRatio(0.7, contentMode: .fit)
    }
}

extension CardValue.Suit {
    var color: Color {
        switch self {
        case .clubs, .spades: .black
        case .diamonds, .hearts: .red
        }
    }
}

#Preview {
    HStack {
        CardPlaceholderView()
        CardBackView()
        CardFaceView(card: CardValue(rank: .ace, suit: .spades))
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .padding()
    .background(.green)
}
