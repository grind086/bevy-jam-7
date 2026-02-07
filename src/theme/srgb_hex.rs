use bevy::color::Color;

/// Creates an sRGB color from the given hex string.
///
/// Strings may be in any of the following formats. The leading "#" is optional:
/// - `#RGB`
/// - `#RRGGBB`
/// - `#RRGGBBAA`
/// 
/// # Example
/// 
/// ```ignore
/// # use bevy::color::Color;
/// # use client::theme::srgb_hex;
/// const RED: Color = srgb_hex("#F00");
/// const GREEN: Color = srgb_hex("#00FF00");
/// const BLUE: Color = srgb_hex("#0000FFFF");
/// ```
pub const fn srgb_hex(hex: &str) -> Color {
    try_srgb_hex(hex).expect("invalid color string")
}

/// Faillibly creates an sRGB color from the given hex string.
///
/// See [`srgb_hex`] for more.
pub const fn try_srgb_hex(hex: &str) -> Option<Color> {
    const fn hex_digit(v: u8) -> Option<u8> {
        match v {
            b'0'..=b'9' => Some(v - b'0'),
            b'A'..=b'F' => Some(v - b'A' + 10),
            b'a'..=b'f' => Some(v - b'a' + 10),
            _ => None,
        }
    }

    const fn hex_pair(u: u8, l: u8) -> Option<u8> {
        match (hex_digit(u), hex_digit(l)) {
            (Some(u), Some(l)) => Some(u << 4 | l),
            _ => None,
        }
    }

    let mut b = hex.as_bytes();
    if b[0] == b'#' {
        // SAFETY: We are re-creating the slice with one element removed.
        b = unsafe { core::slice::from_raw_parts(b.as_ptr().add(1), b.len() - 1) };
        // TODO: Replace the above unsafe with this when const slice indexing is stable
        // b = &b[1..];
    }

    if b.len() == 3 {
        // Format: RGB
        if let (Some(r), Some(g), Some(b)) = (
            hex_pair(b[0], b[0]),
            hex_pair(b[1], b[1]),
            hex_pair(b[2], b[2]),
        ) {
            return Some(Color::srgb_u8(r, g, b));
        }
    } else if b.len() == 6 {
        // Format: RRGGBB
        if let (Some(r), Some(g), Some(b)) = (
            hex_pair(b[0], b[1]),
            hex_pair(b[2], b[3]),
            hex_pair(b[4], b[5]),
        ) {
            return Some(Color::srgb_u8(r, g, b));
        }
    } else if b.len() == 8 {
        // Format: RRGGBBAA
        if let (Some(r), Some(g), Some(b), Some(a)) = (
            hex_pair(b[0], b[1]),
            hex_pair(b[2], b[3]),
            hex_pair(b[4], b[5]),
            hex_pair(b[6], b[7]),
        ) {
            return Some(Color::srgba_u8(r, g, b, a));
        }
    }

    None
}
