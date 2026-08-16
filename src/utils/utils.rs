use ratatui::style::Color;

/**
 * A function for converting a [String] into a [ratatui::style::Color]. Defaults to `Color::Reset`
 */
pub fn string_to_color(string: String) -> Color {
    match string.to_lowercase().as_str() {
        // This code is Trademark and copyright 2021 - 2023 GutHub user uttereyan21, used
        // and modified under the terms of the MIT licensce. No I won't include it here
        // you can find the deed in the project root just pretend that it's 2021 - 2023 on
        //  the year and uttereyan21 on the name pls dont sue me :>.
        "reset" | "" => Color::Reset,

        "red" => Color::Red,
        "green" => Color::Green,
        "black" => Color::Black,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "white" => Color::White,

        "darkgray" => Color::DarkGray,
        "lightblue" => Color::LightBlue,
        "lightcyan" => Color::LightCyan,
        "lightgreen" => Color::LightGreen,
        "lightmagenta" => Color::LightMagenta,
        "lightred" => Color::LightRed,
        "lightyellow" => Color::LightYellow,

        _ => {
            match string.len() {
                3 => {
                    let i = string.parse::<u8>();
                    if let Ok(j) = i {
                        Color::Indexed(j)
                    } else {
                        Color::Reset
                    }
                }
                4 | 7 => {
                    let mut d = string.clone();
                    if d.remove(0) != '#' {
                        Color::Reset
                    } else {
                        let r;
                        let g;
                        let b;
                        // This part of code (though modified) has the same license.
                        match d.len() {
                            6 => {
                                r = u8::from_str_radix(&d[0..2], 16);
                                g = u8::from_str_radix(&d[2..4], 16);
                                b = u8::from_str_radix(&d[4..6], 16);
                            }
                            3 => {
                                r = u8::from_str_radix(&d[0..1], 16).map(|r| r * 17);
                                g = u8::from_str_radix(&d[1..2], 16).map(|g| g * 17);
                                b = u8::from_str_radix(&d[2..3], 16).map(|b| b * 17);
                            }
                            _ => {
                                // I don't know how this would happen but I guess put it there.
                                r = Ok(0);
                                g = Ok(0);
                                b = Ok(0);
                            }
                        }
                        match (r, g, b) {
                            (Ok(r), Ok(g), Ok(b)) => Color::Rgb(r, g, b),
                            _ => Color::Reset,
                        }
                    }
                }
                _ => Color::Reset,
            }
        }
    }
}
