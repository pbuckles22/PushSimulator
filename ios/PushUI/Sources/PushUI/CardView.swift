import SwiftUI

/// One card. The face follows the suit and the rank.
public struct CardView: View {
    public let card: BoardCard

    public init(card: BoardCard) {
        self.card = card
    }

    public var face: CardFace {
        CardFace(card)
    }

    public var body: some View {
        let face = self.face
        VStack(spacing: 4) {
            Text(face.pip)
                .font(.title2.bold())
            if !face.suitSymbol.isEmpty {
                Text(face.suitSymbol)
                    .font(.title3)
            }
            if face.isLocked {
                Text("locked")
                    .font(.caption2)
            }
        }
        .foregroundStyle(face.isRed ? Color.red : Color.black)
        .frame(minWidth: 44, minHeight: 64)
        .padding(.horizontal, 6)
        .padding(.vertical, 8)
        .background(Color.white)
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.black, lineWidth: 1)
        )
        .accessibilityLabel(face.spoken)
    }
}
