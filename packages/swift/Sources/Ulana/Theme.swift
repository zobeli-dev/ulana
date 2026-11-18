import SwiftUI

/// Everything the renderer draws with. A view only names tokens ("md",
/// "accent"); the theme decides what they look like.
///
/// The default follows the system: the app's accent colour, semantic text
/// colours, Dynamic Type, and dark mode.
public struct UlanaTheme: Sendable {
    public struct ToneColors: Sendable {
        public var fg: Color
        public var soft: Color
        public init(fg: Color, soft: Color) {
            self.fg = fg
            self.soft = soft
        }
    }

    public var surface: Color
    public var border: Color
    public var tones: [Token.Tone: ToneColors]
    public var space: [Token.Space: CGFloat]
    public var fonts: [Token.TextStyle: Font]
    public var cardRadius: CGFloat
    /// The smallest side of anything pressable, in points.
    public var hitSize: CGFloat

    public init(
        surface: Color = UlanaTheme.systemSurface,
        border: Color = Color.secondary.opacity(0.25),
        tones: [Token.Tone: ToneColors] = UlanaTheme.systemTones,
        space: [Token.Space: CGFloat] = [.none: 0, .xs: 4, .sm: 8, .md: 16, .lg: 24, .xl: 32],
        // Without `.display`, display text is 52 pt rounded bold, grown with Dynamic Type.
        fonts: [Token.TextStyle: Font] = [
            .title: .largeTitle.bold(),
            .heading: .title3.weight(.semibold),
            .body: .body,
            .caption: .footnote,
            .mono: .body.monospaced(),
        ],
        cardRadius: CGFloat = 14,
        hitSize: CGFloat = 44
    ) {
        self.surface = surface
        self.border = border
        self.tones = tones
        self.space = space
        self.fonts = fonts
        self.cardRadius = cardRadius
        self.hitSize = hitSize
    }

    public static let system = UlanaTheme()

    public static var systemTones: [Token.Tone: ToneColors] {
        [
            .default: ToneColors(fg: .primary, soft: Color.secondary.opacity(0.12)),
            .muted: ToneColors(fg: .secondary, soft: Color.secondary.opacity(0.12)),
            .accent: ToneColors(fg: .accentColor, soft: Color.accentColor.opacity(0.15)),
            .success: ToneColors(fg: .green, soft: Color.green.opacity(0.15)),
            .warning: ToneColors(fg: .orange, soft: Color.orange.opacity(0.15)),
            .danger: ToneColors(fg: .red, soft: Color.red.opacity(0.15)),
        ]
    }

    public static var systemSurface: Color {
        #if canImport(UIKit)
        Color(uiColor: .secondarySystemGroupedBackground)
        #else
        Color(nsColor: .controlBackgroundColor)
        #endif
    }

    func tone(_ tone: Token.Tone?) -> ToneColors {
        tones[tone ?? .default] ?? ToneColors(fg: .primary, soft: .clear)
    }

    func space(_ s: Token.Space?) -> CGFloat {
        space[s ?? .none] ?? 0
    }

    func font(_ style: Token.TextStyle?) -> Font {
        fonts[style ?? .body] ?? .body
    }
}
