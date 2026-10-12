/*
See the LICENSE.txt file for this sample's licensing information.

Abstract:
A value that describes the suit and rank of a playing card.
*/

import Foundation
import CoreTransferable
import UniformTypeIdentifiers

nonisolated struct CardValue: Hashable, Sendable, Codable {
    enum Rank: Int, Hashable, Sendable, Codable, CaseIterable {
        case ace = 1, two, three, four, five, six, seven, eight, nine, ten,
             jack, queen, king
    }

    enum Suit: Int, Hashable, Sendable, Codable, CaseIterable {
        case clubs = 0, diamonds, hearts, spades
    }

    var rank: Rank
    var suit: Suit
}

extension CardValue: Identifiable {
    var id: Self { self }
}

extension CardValue.Suit: Identifiable {
    var id: Self { self }
}

extension CardValue: Transferable {
    nonisolated static var transferRepresentation: some TransferRepresentation {
        CodableRepresentation(contentType: .cardValue)
    }
}

extension UTType {
    /// A playing card's value, such as "ace of spades."
    nonisolated static let cardValue = UTType(
        exportedAs: "com.example.apple-samplecode.SwiftUI-Reorderable.card",
        conformingTo: .data)
}

// MARK: String Values

extension CardValue.Rank {
    var symbol: String {
        switch self {
        case .ace: "A"
        case .two: "2"
        case .three: "3"
        case .four: "4"
        case .five: "5"
        case .six: "6"
        case .seven: "7"
        case .eight: "8"
        case .nine: "9"
        case .ten: "10"
        case .jack: "J"
        case .queen: "Q"
        case .king: "K"
        }
    }
}

extension CardValue.Suit {
    var symbol: String {
        switch self {
        case .clubs: "♣️"
        case .diamonds: "♦️"
        case .hearts: "♥️"
        case .spades: "♠️"
        }
    }
}

extension CardValue {
    var symbol: String {
        "\(rank.symbol)\(suit.symbol)"
    }
}

extension CardValue.Suit {
    var systemImageName: String {
        switch self {
        case .clubs: "suit.club.fill"
        case .diamonds: "suit.diamond.fill"
        case .hearts: "suit.heart.fill"
        case .spades: "suit.spade.fill"
        }
    }
}

// MARK: Suit Color

extension CardValue.Suit {
    nonisolated enum PlayingColor: Hashable, Sendable { case red, black }

    nonisolated var playingColor: PlayingColor {
        switch self {
        case .clubs, .spades: .black
        case .diamonds, .hearts: .red
        }
    }
}

// MARK: Card Deck

extension CardValue.Suit {
    var cards: [CardValue] {
        CardValue.Rank.allCases.map { rank in
            CardValue(rank: rank, suit: self)
        }
    }
}

extension CardValue {
    static var deck: [CardValue] {
        CardValue.Suit.allCases.flatMap { suit in
            suit.cards
        }
    }
}
