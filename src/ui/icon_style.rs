#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconStyle {
    Round,
    Ascii,
    None,
}

/// Per-style glyph set consumed by the list renderer.
pub struct Glyphs {
    pub unselected: &'static str,
    pub running: &'static str,
    pub cursor: &'static str,
}

impl IconStyle {
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("round") {
            Some(Self::Round)
        } else if s.eq_ignore_ascii_case("ascii") {
            Some(Self::Ascii)
        } else if s.eq_ignore_ascii_case("none") {
            Some(Self::None)
        } else {
            None
        }
    }

    pub fn glyphs(self) -> Glyphs {
        match self {
            Self::Round => Glyphs {
                unselected: "○",
                running: "●",
                cursor: "▶",
            },
            Self::Ascii => Glyphs {
                unselected: "o",
                running: "*",
                cursor: ">",
            },
            Self::None => Glyphs {
                unselected: "",
                running: "",
                cursor: "",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_names_case_insensitive() {
        assert_eq!(IconStyle::parse("round"), Some(IconStyle::Round));
        assert_eq!(IconStyle::parse("ASCII"), Some(IconStyle::Ascii));
        assert_eq!(IconStyle::parse("None"), Some(IconStyle::None));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(IconStyle::parse("fancy"), None);
    }

    #[test]
    fn glyphs_by_style() {
        let r = IconStyle::Round.glyphs();
        assert_eq!(r.unselected, "○");
        assert_eq!(r.running, "●");
        assert_eq!(r.cursor, "▶");

        let a = IconStyle::Ascii.glyphs();
        assert_eq!(a.unselected, "o");
        assert_eq!(a.running, "*");
        assert_eq!(a.cursor, ">");

        let n = IconStyle::None.glyphs();
        assert_eq!(n.unselected, "");
        assert_eq!(n.running, "");
        assert_eq!(n.cursor, "");
    }
}
