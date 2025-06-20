use bevy::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Colored;

impl Colored {

    pub const TRANSPARENT: Color = Color::NONE;

    pub const ALICE_BLUE: Color = Color::Srgba(Srgba::new(0.941, 0.973, 1.0, 1.0));
    pub const ANTIQUE_WHITE: Color = Color::Srgba(Srgba::new(0.980, 0.922, 0.843, 1.0));
    pub const AQUA: Color = Color::Srgba(Srgba::new(0.0, 1.0, 1.0, 1.0));
    pub const AQUAMARINE: Color = Color::Srgba(Srgba::new(0.498, 1.0, 0.831, 1.0));
    pub const AZURE: Color = Color::Srgba(Srgba::new(0.941, 1.0, 1.0, 1.0));

    pub const BEIGE: Color = Color::Srgba(Srgba::new(0.961, 0.961, 0.863, 1.0));
    pub const BISQUE: Color = Color::Srgba(Srgba::new(1.0, 0.894, 0.769, 1.0));
    pub const BLACK: Color = Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 1.0));
    pub const BLANCHED_ALMOND: Color = Color::Srgba(Srgba::new(1.0, 0.922, 0.804, 1.0));
    pub const BLUE: Color = Color::Srgba(Srgba::new(0.0, 0.0, 1.0, 1.0));
    pub const BLUE_VIOLET: Color = Color::Srgba(Srgba::new(0.541, 0.169, 0.886, 1.0));
    pub const BROWN: Color = Color::Srgba(Srgba::new(0.647, 0.165, 0.165, 1.0));
    pub const BURLYWOOD: Color = Color::Srgba(Srgba::new(0.871, 0.722, 0.529, 1.0));

    pub const CADET_BLUE: Color = Color::Srgba(Srgba::new(0.373, 0.620, 0.627, 1.0));
    pub const CHARTREUSE: Color = Color::Srgba(Srgba::new(0.498, 1.0, 0.0, 1.0));
    pub const CHOCOLATE: Color = Color::Srgba(Srgba::new(0.824, 0.412, 0.118, 1.0));
    pub const CORAL: Color = Color::Srgba(Srgba::new(1.0, 0.498, 0.314, 1.0));
    pub const CORNFLOWER_BLUE: Color = Color::Srgba(Srgba::new(0.392, 0.584, 0.929, 1.0));
    pub const CORNSILK: Color = Color::Srgba(Srgba::new(1.0, 0.973, 0.863, 1.0));
    pub const CRIMSON: Color = Color::Srgba(Srgba::new(0.863, 0.078, 0.235, 1.0));
    pub const CYAN: Color = Color::Srgba(Srgba::new(0.0, 1.0, 1.0, 1.0));

    pub const DARK_BLUE: Color = Color::Srgba(Srgba::new(0.0, 0.0, 0.545, 1.0));
    pub const DARK_CYAN: Color = Color::Srgba(Srgba::new(0.0, 0.545, 0.545, 1.0));
    pub const DARK_GOLDENROD: Color = Color::Srgba(Srgba::new(0.722, 0.525, 0.043, 1.0));
    pub const DARK_GRAY: Color = Color::Srgba(Srgba::new(0.663, 0.663, 0.663, 1.0));
    pub const DARK_GREEN: Color = Color::Srgba(Srgba::new(0.0, 0.392, 0.0, 1.0));
    pub const DARK_GREY: Color = Self::DARK_GRAY;
    pub const DARK_KHAKI: Color = Color::Srgba(Srgba::new(0.741, 0.718, 0.420, 1.0));
    pub const DARK_MAGENTA: Color = Color::Srgba(Srgba::new(0.545, 0.0, 0.545, 1.0));
    pub const DARK_OLIVE_GREEN: Color = Color::Srgba(Srgba::new(0.333, 0.420, 0.184, 1.0));
    pub const DARK_ORANGE: Color = Color::Srgba(Srgba::new(1.0, 0.549, 0.0, 1.0));
    pub const DARK_ORCHID: Color = Color::Srgba(Srgba::new(0.600, 0.196, 0.800, 1.0));
    pub const DARK_RED: Color = Color::Srgba(Srgba::new(0.545, 0.0, 0.0, 1.0));
    pub const DARK_SALMON: Color = Color::Srgba(Srgba::new(0.914, 0.588, 0.478, 1.0));
    pub const DARK_SEA_GREEN: Color = Color::Srgba(Srgba::new(0.561, 0.737, 0.561, 1.0));
    pub const DARK_SLATE_BLUE: Color = Color::Srgba(Srgba::new(0.282, 0.239, 0.545, 1.0));
    pub const DARK_SLATE_GRAY: Color = Color::Srgba(Srgba::new(0.184, 0.310, 0.310, 1.0));
    pub const DARK_SLATE_GREY: Color = Self::DARK_SLATE_GRAY;
    pub const DARK_TURQUOISE: Color = Color::Srgba(Srgba::new(0.0, 0.808, 0.820, 1.0));
    pub const DARK_VIOLET: Color = Color::Srgba(Srgba::new(0.580, 0.0, 0.827, 1.0));
    pub const DEEP_PINK: Color = Color::Srgba(Srgba::new(1.0, 0.078, 0.576, 1.0));
    pub const DEEP_SKY_BLUE: Color = Color::Srgba(Srgba::new(0.0, 0.749, 1.0, 1.0));
    pub const DIM_GRAY: Color = Color::Srgba(Srgba::new(0.412, 0.412, 0.412, 1.0));
    pub const DIM_GREY: Color = Self::DIM_GRAY;
    pub const DODGER_BLUE: Color = Color::Srgba(Srgba::new(0.118, 0.565, 1.0, 1.0));

    pub const FIREBRICK: Color = Color::Srgba(Srgba::new(0.698, 0.133, 0.133, 1.0));
    pub const FLORAL_WHITE: Color = Color::Srgba(Srgba::new(1.0, 0.980, 0.941, 1.0));
    pub const FOREST_GREEN: Color = Color::Srgba(Srgba::new(0.133, 0.545, 0.133, 1.0));

    pub const FUCHSIA: Color = Color::Srgba(Srgba::new(1.0, 0.0, 1.0, 1.0));
    pub const GAINSBORO: Color = Color::Srgba(Srgba::new(0.863, 0.863, 0.863, 1.0));
    pub const GHOST_WHITE: Color = Color::Srgba(Srgba::new(0.973, 0.973, 1.0, 1.0));
    pub const GOLD: Color = Color::Srgba(Srgba::new(1.0, 0.843, 0.0, 1.0));
    pub const GOLDENROD: Color = Color::Srgba(Srgba::new(0.855, 0.647, 0.125, 1.0));
    pub const GREEN: Color = Color::Srgba(Srgba::new(0.0, 1.0, 0.0, 1.0));
    pub const GREEN_YELLOW: Color = Color::Srgba(Srgba::new(0.678, 1.0, 0.184, 1.0));
    pub const GREY: Color = Color::Srgba(Srgba::new(0.502, 0.502, 0.502, 1.0));
    pub const GRAY: Color = Self::GREY;

    pub const HONEYDEW: Color = Color::Srgba(Srgba::new(0.941, 1.0, 0.941, 1.0));
    pub const HOT_PINK: Color = Color::Srgba(Srgba::new(1.0, 0.412, 0.706, 1.0));
    pub const INDIAN_RED: Color = Color::Srgba(Srgba::new(0.804, 0.361, 0.361, 1.0));
    pub const INDIGO: Color = Color::Srgba(Srgba::new(0.294, 0.0, 0.509, 1.0));
    pub const IVORY: Color = Color::Srgba(Srgba::new(1.0, 1.0, 0.941, 1.0));

    pub const KHAKI: Color = Color::Srgba(Srgba::new(0.765, 0.675, 0.375, 1.0));
    pub const LAVENDER: Color = Color::Srgba(Srgba::new(0.902, 0.902, 0.980, 1.0));
    pub const LAVENDER_BLUSH: Color = Color::Srgba(Srgba::new(1.0, 0.941, 0.961, 1.0));
    pub const LAWN_GREEN: Color = Color::Srgba(Srgba::new(0.486, 0.988, 0.0, 1.0));
    pub const LEMON_CHIFFON: Color = Color::Srgba(Srgba::new(1.0, 0.980, 0.804, 1.0));

    pub const LIGHT_BLUE: Color = Color::Srgba(Srgba::new(0.678, 0.847, 0.902, 1.0));
    pub const LIGHT_CORAL: Color = Color::Srgba(Srgba::new(0.941, 0.502, 0.502, 1.0));
    pub const LIGHT_CYAN: Color = Color::Srgba(Srgba::new(0.878, 1.0, 1.0, 1.0));
    pub const LIGHT_GOLDENROD_YELLOW: Color = Color::Srgba(Srgba::new(0.980, 0.980, 0.824, 1.0));
    pub const LIGHT_GREEN: Color = Color::Srgba(Srgba::new(0.565, 0.933, 0.565, 1.0));
    pub const LIGHT_GREY: Color = Color::Srgba(Srgba::new(0.827, 0.827, 0.827, 1.0));
    pub const LIGHT_PINK: Color = Color::Srgba(Srgba::new(1.0, 0.714, 0.757, 1.0));
    pub const LIGHT_SALMON: Color = Color::Srgba(Srgba::new(1.0, 0.627, 0.478, 1.0));
    pub const LIGHT_SEA_GREEN: Color = Color::Srgba(Srgba::new(0.125, 0.698, 0.667, 1.0));
    pub const LIGHT_SKY_BLUE: Color = Color::Srgba(Srgba::new(0.529, 0.808, 0.980, 1.0));
    pub const LIGHT_SLATE_GRAY: Color = Color::Srgba(Srgba::new(0.467, 0.533, 0.600, 1.0));
    pub const LIGHT_SLATE_GREY: Color = Self::LIGHT_SLATE_GRAY;

    pub const LIGHT_STEEL_BLUE: Color = Color::Srgba(Srgba::new(0.690, 0.769, 0.871, 1.0));
    pub const LIGHT_YELLOW: Color = Color::Srgba(Srgba::new(1.0, 1.0, 0.804, 1.0));

    pub const MAGENTA: Color = Color::Srgba(Srgba::new(1.0, 0.0, 1.0, 1.0));
    pub const MISTY_ROSE: Color = Color::Srgba(Srgba::new(1.0, 0.894, 0.882, 1.0));
    pub const MOCCASIN: Color = Color::Srgba(Srgba::new(1.0, 0.894, 0.710, 1.0));
    pub const NAVY: Color = Color::Srgba(Srgba::new(0.0, 0.0, 0.502, 1.0));
    pub const OLD_LACE: Color = Color::Srgba(Srgba::new(0.992, 0.961, 0.902, 1.0));
    pub const OLIVE: Color = Color::Srgba(Srgba::new(0.502, 0.502, 0.0, 1.0));
    pub const OLIVE_DRAB: Color = Color::Srgba(Srgba::new(0.420, 0.557, 0.137, 1.0));
    pub const ORANGE: Color = Color::Srgba(Srgba::new(1.0, 0.647, 0.0, 1.0));
    pub const ORANGE_RED: Color = Color::Srgba(Srgba::new(1.0, 0.271, 0.0, 1.0));
    pub const ORCHID: Color = Color::Srgba(Srgba::new(0.855, 0.439, 0.839, 1.0));

    pub const PALE_GOLDENROD: Color = Color::Srgba(Srgba::new(0.933, 0.910, 0.667, 1.0));
    pub const PALE_GREEN: Color = Color::Srgba(Srgba::new(0.596, 0.984, 0.596, 1.0));
    pub const PALE_TURQUOISE: Color = Color::Srgba(Srgba::new(0.686, 0.933, 0.933, 1.0));
    pub const PALE_VIOLET_RED: Color = Color::Srgba(Srgba::new(0.859, 0.439, 0.576, 1.0));
    pub const PAPAYA_WHIP: Color = Color::Srgba(Srgba::new(1.0, 0.937, 0.835, 1.0));
    pub const PEACH_PUFF: Color = Color::Srgba(Srgba::new(1.0, 0.855, 0.725, 1.0));
    pub const PERU: Color = Color::Srgba(Srgba::new(0.804, 0.522, 0.247, 1.0));
    pub const PINK: Color = Color::Srgba(Srgba::new(1.0, 0.753, 0.796, 1.0));
    pub const PLUM: Color = Color::Srgba(Srgba::new(0.867, 0.627, 0.867, 1.0));
    pub const POWDER_BLUE: Color = Color::Srgba(Srgba::new(0.690, 0.878, 0.902, 1.0));

    pub const PURPLE: Color = Color::Srgba(Srgba::new(0.502, 0.0, 0.502, 1.0));
    pub const REBECCA_PURPLE: Color = Color::Srgba(Srgba::new(0.290, 0.0, 0.509, 1.0));
    pub const RED: Color = Color::Srgba(Srgba::new(1.0, 0.0, 0.0, 1.0));
    pub const ROSY_BROWN: Color = Color::Srgba(Srgba::new(0.737, 0.561, 0.561, 1.0));
    pub const ROYAL_BLUE: Color = Color::Srgba(Srgba::new(0.255, 0.412, 0.882, 1.0));
    pub const SADDLE_BROWN: Color = Color::Srgba(Srgba::new(0.545, 0.271, 0.075, 1.0));
    pub const SALMON: Color = Color::Srgba(Srgba::new(0.980, 0.502, 0.447, 1.0));
    pub const SANDY_BROWN: Color = Color::Srgba(Srgba::new(0.957, 0.643, 0.376, 1.0));
    pub const SEA_GREEN: Color = Color::Srgba(Srgba::new(0.180, 0.545, 0.341, 1.0));
    pub const SEA_SHELL: Color = Color::Srgba(Srgba::new(1.0, 0.961, 0.933, 1.0));

    pub const TAN: Color = Color::Srgba(Srgba::new(0.824, 0.706, 0.549, 1.0));
    pub const TEAL: Color = Color::Srgba(Srgba::new(0.0,   0.502, 0.502, 1.0));
    pub const THISTLE: Color = Color::Srgba(Srgba::new(0.847, 0.749, 0.847, 1.0));
    pub const TOMATO: Color = Color::Srgba(Srgba::new(1.0,   0.388, 0.278, 1.0));
    pub const TURQUOISE: Color = Color::Srgba(Srgba::new(0.251, 0.878, 0.816, 1.0));

    pub const VIOLET: Color = Color::Srgba(Srgba::new(0.933, 0.510, 0.933, 1.0));

    pub const WHEAT: Color = Color::Srgba(Srgba::new(0.961, 0.871, 0.702, 1.0));
    pub const WHITE: Color = Color::Srgba(Srgba::new(1.0,   1.0,   1.0,   1.0));
    pub const WHITE_SMOKE: Color = Color::Srgba(Srgba::new(0.961, 0.961, 0.961, 1.0));

    pub const YELLOW: Color = Color::Srgba(Srgba::new(1.0,   1.0,   0.0,   1.0));
    pub const YELLOW_GREEN: Color = Color::Srgba(Srgba::new(0.604, 0.804, 0.196, 1.0));

    pub fn hex_to_color(hex: &str) -> Color {
        // Remove the "#" prefix if it exists
        let hex = hex.trim_start_matches('#');

        // Ensure the hex string is either 3, 6, or 8 characters long
        if hex.len() != 3 && hex.len() != 4 && hex.len() != 6 && hex.len() != 8 {
            panic!("Invalid hex string length: {}", hex);
        }

        // If the length is 3, expand it to 6 (e.g. "fff" -> "ffffff")
        let hex = match hex.len() {
            3 => format!("{}{}{}{}{}{}",
                         &hex[0..1], &hex[0..1],
                         &hex[1..2], &hex[1..2],
                         &hex[2..3], &hex[2..3]),
            4 => format!("{}{}{}{}{}{}{}{}",
                         &hex[0..1], &hex[0..1],
                         &hex[1..2], &hex[1..2],
                         &hex[2..3], &hex[2..3],
                         &hex[3..4], &hex[3..4]),
            6 | 8 => hex.to_string(),
            _ => unreachable!(),
        };

        // Parse the hex string into the RGBA components (values between 0 and 255)
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        // If there's an alpha component (e.g., #ff5733ff)
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
        } else {
            255 // Default to fully opaque if no alpha is provided
        };
        
        // Convert the RGBA components to the [0.0, 1.0] range and return a Color
        Color::Srgba(Srgba {
            red: r as f32 / 255.0,
            green: g as f32 / 255.0,
            blue: b as f32 / 255.0,
            alpha: a as f32 / 255.0,
        })
    }

    pub fn named(name: &str) -> Option<Color> {
        match name.to_lowercase().as_str() {
            "aliceblue" => Some(Self::ALICE_BLUE),
            "antiquewhite" => Some(Self::ANTIQUE_WHITE),
            "aqua" => Some(Self::AQUA),
            "aquamarine" => Some(Self::AQUAMARINE),
            "azure" => Some(Self::AZURE),
            "beige" => Some(Self::BEIGE),
            "bisque" => Some(Self::BISQUE),
            "black" => Some(Self::BLACK),
            "blanchedalmond" => Some(Self::BLANCHED_ALMOND),
            "blue" => Some(Self::BLUE),
            "blueviolet" => Some(Self::BLUE_VIOLET),
            "brown" => Some(Self::BROWN),
            "burlywood" => Some(Self::BURLYWOOD),
            "cadetblue" => Some(Self::CADET_BLUE),
            "chartreuse" => Some(Self::CHARTREUSE),
            "chocolate" => Some(Self::CHOCOLATE),
            "coral" => Some(Self::CORAL),
            "cornflowerblue" => Some(Self::CORNFLOWER_BLUE),
            "cornsilk" => Some(Self::CORNSILK),
            "crimson" => Some(Self::CRIMSON),
            "cyan" => Some(Self::CYAN),
            "darkblue" => Some(Self::DARK_BLUE),
            "darkcyan" => Some(Self::DARK_CYAN),
            "darkgoldenrod" => Some(Self::DARK_GOLDENROD),
            "darkgray" | "darkgrey" => Some(Self::DARK_GRAY),
            "darkgreen" => Some(Self::DARK_GREEN),
            "darkkhaki" => Some(Self::DARK_KHAKI),
            "darkmagenta" => Some(Self::DARK_MAGENTA),
            "darkolivegreen" => Some(Self::DARK_OLIVE_GREEN),
            "darkorange" => Some(Self::DARK_ORANGE),
            "darkorchid" => Some(Self::DARK_ORCHID),
            "darkred" => Some(Self::DARK_RED),
            "darksalmon" => Some(Self::DARK_SALMON),
            "darkseagreen" => Some(Self::DARK_SEA_GREEN),
            "darkslateblue" => Some(Self::DARK_SLATE_BLUE),
            "darkslategray" | "darkslategrey" => Some(Self::DARK_SLATE_GRAY),
            "darkturquoise" => Some(Self::DARK_TURQUOISE),
            "darkviolet" => Some(Self::DARK_VIOLET),
            "deeppink" => Some(Self::DEEP_PINK),
            "deepskyblue" => Some(Self::DEEP_SKY_BLUE),
            "dimgray" | "dimgrey" => Some(Self::DIM_GRAY),
            "dodgerblue" => Some(Self::DODGER_BLUE),
            "firebrick" => Some(Self::FIREBRICK),
            "floralwhite" => Some(Self::FLORAL_WHITE),
            "forestgreen" => Some(Self::FOREST_GREEN),
            "fuchsia" => Some(Self::FUCHSIA),
            "gainsboro" => Some(Self::GAINSBORO),
            "ghostwhite" => Some(Self::GHOST_WHITE),
            "gold" => Some(Self::GOLD),
            "goldenrod" => Some(Self::GOLDENROD),
            "gray" | "grey" => Some(Self::GRAY),
            "green" => Some(Self::GREEN),
            "greenyellow" => Some(Self::GREEN_YELLOW),
            "honeydew" => Some(Self::HONEYDEW),
            "hotpink" => Some(Self::HOT_PINK),
            "indianred" => Some(Self::INDIAN_RED),
            "indigo" => Some(Self::INDIGO),
            "ivory" => Some(Self::IVORY),
            "khaki" => Some(Self::KHAKI),
            "lavender" => Some(Self::LAVENDER),
            "lavenderblush" => Some(Self::LAVENDER_BLUSH),
            "lawngreen" => Some(Self::LAWN_GREEN),
            "lemonchiffon" => Some(Self::LEMON_CHIFFON),
            "lightblue" => Some(Self::LIGHT_BLUE),
            "lightcoral" => Some(Self::LIGHT_CORAL),
            "lightcyan" => Some(Self::LIGHT_CYAN),
            "lightgoldenrodyellow" => Some(Self::LIGHT_GOLDENROD_YELLOW),
            "lightgreen" => Some(Self::LIGHT_GREEN),
            "lightgray" | "lightgrey" => Some(Self::LIGHT_GREY),
            "lightpink" => Some(Self::LIGHT_PINK),
            "lightsalmon" => Some(Self::LIGHT_SALMON),
            "lightseagreen" => Some(Self::LIGHT_SEA_GREEN),
            "lightskyblue" => Some(Self::LIGHT_SKY_BLUE),
            "lightslategray" | "lightslategrey" => Some(Self::LIGHT_SLATE_GRAY),
            "lightsteelblue" => Some(Self::LIGHT_STEEL_BLUE),
            "lightyellow" => Some(Self::LIGHT_YELLOW),
            "magenta" => Some(Self::MAGENTA),
            "mistyrose" => Some(Self::MISTY_ROSE),
            "moccasin" => Some(Self::MOCCASIN),
            "navy" => Some(Self::NAVY),
            "oldlace" => Some(Self::OLD_LACE),
            "olive" => Some(Self::OLIVE),
            "olivedrab" => Some(Self::OLIVE_DRAB),
            "orange" => Some(Self::ORANGE),
            "orangered" => Some(Self::ORANGE_RED),
            "orchid" => Some(Self::ORCHID),
            "palegoldenrod" => Some(Self::PALE_GOLDENROD),
            "palegreen" => Some(Self::PALE_GREEN),
            "paleturquoise" => Some(Self::PALE_TURQUOISE),
            "palevioletred" => Some(Self::PALE_VIOLET_RED),
            "papayawhip" => Some(Self::PAPAYA_WHIP),
            "peachpuff" => Some(Self::PEACH_PUFF),
            "peru" => Some(Self::PERU),
            "pink" => Some(Self::PINK),
            "plum" => Some(Self::PLUM),
            "powderblue" => Some(Self::POWDER_BLUE),
            "purple" => Some(Self::PURPLE),
            "rebeccapurple" => Some(Self::REBECCA_PURPLE),
            "red" => Some(Self::RED),
            "rosybrown" => Some(Self::ROSY_BROWN),
            "royalblue" => Some(Self::ROYAL_BLUE),
            "saddlebrown" => Some(Self::SADDLE_BROWN),
            "salmon" => Some(Self::SALMON),
            "sandybrown" => Some(Self::SANDY_BROWN),
            "seagreen" => Some(Self::SEA_GREEN),
            "seashell" => Some(Self::SEA_SHELL),
            "tan" => Some(Self::TAN),
            "teal" => Some(Self::TEAL),
            "thistle" => Some(Self::THISTLE),
            "tomato" => Some(Self::TOMATO),
            "transparent" => Some(Self::TRANSPARENT),
            "turquoise" => Some(Self::TURQUOISE),
            "violet" => Some(Self::VIOLET),
            "wheat" => Some(Self::WHEAT),
            "white" => Some(Self::WHITE),
            "whitesmoke" => Some(Self::WHITE_SMOKE),
            "yellow" => Some(Self::YELLOW),
            "yellowgreen" => Some(Self::YELLOW_GREEN),
            _ => None,
        }
    }
}