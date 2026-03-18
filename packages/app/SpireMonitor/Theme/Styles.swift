import SwiftUI

struct SectionCard: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding(14)
            .background(Color.stsCard)
            .clipShape(RoundedRectangle(cornerRadius: 6))
            .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.stsBorder, lineWidth: 1))
    }
}

extension View {
    func sectionCard() -> some View {
        modifier(SectionCard())
    }
}

struct SectionHeader: View {
    let title: String

    var body: some View {
        Text(title)
            .font(.stsBody)
            .fontWeight(.semibold)
            .foregroundStyle(Color.stsTextDim)
            .textCase(.uppercase)
    }
}
