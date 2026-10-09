@main
enum DeckSizeCheck {
    static func main() {
        let size = Game().getDeckSize()
        precondition(size == 108, "deck size \(size)")
        print(size)
    }
}
