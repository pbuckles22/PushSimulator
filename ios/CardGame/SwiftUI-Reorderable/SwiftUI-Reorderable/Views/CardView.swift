/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A view that shows the current state of a card instance, as well as the
 component views used to show the front, back, and placeholder
 representations of a card.
*/

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
    var card: CardValue

    var body: some View {
        ZStack {
            // Background color
            Color.white
            // Suit image, on its own layer so that it's centered.
            Image(systemName: card.suit.systemImageName)
                .resizable()
                .scaledToFit()
                .foregroundStyle(card.suit.color.gradient)
                .padding()
            // The header with the rank and suit emoji.
            VStack {
                HStack {
                    Text(verbatim: card.rank.symbol)
                    Spacer()
                    Text(verbatim: card.suit.symbol)
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
        .foregroundStyle(card.suit.color)
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
