//! Window layout calculations: halves, quarters, thirds, center, maximize.
//! Matches C# WindowEngine behavior.

use crate::rect::EngineRect;

/// Window actions (matches C# WindowAction enum).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowAction {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    Maximize,
    Center,
    Undo,
    LowerLeft,
    LowerRight,
    UpperLeft,
    UpperRight,
    NextDisplay,
    PreviousDisplay,
    FirstThird,   // left 1 section
    FirstTwoThirds, // left 2 sections
    CenterThird,  // center 1 section
    LastTwoThirds, // right 2 sections
    LastThird,    // right 1 section
}

impl WindowAction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "LeftHalf" => Some(Self::LeftHalf),
            "RightHalf" => Some(Self::RightHalf),
            "TopHalf" => Some(Self::TopHalf),
            "BottomHalf" => Some(Self::BottomHalf),
            "Maximize" => Some(Self::Maximize),
            "Center" => Some(Self::Center),
            "Undo" | "Restore" => Some(Self::Undo),
            "LowerLeft" => Some(Self::LowerLeft),
            "LowerRight" => Some(Self::LowerRight),
            "UpperLeft" => Some(Self::UpperLeft),
            "UpperRight" => Some(Self::UpperRight),
            "NextDisplay" => Some(Self::NextDisplay),
            "PreviousDisplay" => Some(Self::PreviousDisplay),
            "FirstThird" => Some(Self::FirstThird),
            "FirstTwoThirds" => Some(Self::FirstTwoThirds),
            "CenterThird" => Some(Self::CenterThird),
            "LastTwoThirds" => Some(Self::LastTwoThirds),
            "LastThird" => Some(Self::LastThird),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::LeftHalf => "LeftHalf",
            Self::RightHalf => "RightHalf",
            Self::TopHalf => "TopHalf",
            Self::BottomHalf => "BottomHalf",
            Self::Maximize => "Maximize",
            Self::Center => "Center",
            Self::Undo => "Undo",
            Self::LowerLeft => "LowerLeft",
            Self::LowerRight => "LowerRight",
            Self::UpperLeft => "UpperLeft",
            Self::UpperRight => "UpperRight",
            Self::NextDisplay => "NextDisplay",
            Self::PreviousDisplay => "PreviousDisplay",
            Self::FirstThird => "FirstThird",
            Self::FirstTwoThirds => "FirstTwoThirds",
            Self::CenterThird => "CenterThird",
            Self::LastTwoThirds => "LastTwoThirds",
            Self::LastThird => "LastThird",
        }
    }

    pub fn has_calculation(&self) -> bool {
        !matches!(
            self,
            Self::Undo | Self::NextDisplay | Self::PreviousDisplay
        )
    }

    /// True if this is one of the 5 section actions (Left, Left two, Center, Right two, Right).
    pub fn is_section_action(&self) -> bool {
        matches!(
            self,
            Self::FirstThird
                | Self::FirstTwoThirds
                | Self::CenterThird
                | Self::LastTwoThirds
                | Self::LastThird
        )
    }
}

/// Parameters for layout calculation (matches RectCalculationParameters).
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CalculationParams {
    pub window_rect: EngineRect,
    pub work_area: EngineRect,
    pub action: WindowAction,
    pub last_action: Option<LastActionInfo>,
    pub thirds_layout_mode: String,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct LastActionInfo {
    pub rect: EngineRect,
    pub action: WindowAction,
}

#[derive(Clone, Copy, Debug)]
pub struct CalculationResult {
    pub rect: EngineRect,
    pub resulting_action: WindowAction,
}

/// Section layout: (left_part, center_part, right_part). Thirds 1|1|1, Fourths 1|2|1, Fifths 1|3|1.
fn section_ratios(mode: &str) -> (i32, i32, i32) {
    if mode.eq_ignore_ascii_case("Fifths") {
        (1, 3, 1)
    } else if mode.eq_ignore_ascii_case("Fourths") {
        (1, 2, 1)
    } else {
        (1, 1, 1)
    }
}

/// Compute the target rect for the given action and params. Returns None for Undo/NextDisplay/PreviousDisplay.
pub fn calculate(params: &CalculationParams) -> Option<CalculationResult> {
    use WindowAction::*;
    let w = &params.work_area;
    let width = w.width();
    let height = w.height();

    let rect = match params.action {
        LeftHalf => EngineRect {
            left: w.left,
            top: w.top,
            right: w.left + width / 2,
            bottom: w.bottom,
        },
        RightHalf => EngineRect {
            left: w.left + width / 2,
            top: w.top,
            right: w.right,
            bottom: w.bottom,
        },
        TopHalf => EngineRect {
            left: w.left,
            top: w.top,
            right: w.right,
            bottom: w.top + height / 2,
        },
        BottomHalf => EngineRect {
            left: w.left,
            top: w.top + height / 2,
            right: w.right,
            bottom: w.bottom,
        },
        Maximize => *w,
        Center => {
            let win = &params.window_rect;
            let x = w.left + (width - win.width()) / 2;
            let y = w.top + (height - win.height()) / 2;
            EngineRect {
                left: x,
                top: y,
                right: x + win.width(),
                bottom: y + win.height(),
            }
        }
        UpperLeft => EngineRect {
            left: w.left,
            top: w.top,
            right: w.left + width / 2,
            bottom: w.top + height / 2,
        },
        UpperRight => EngineRect {
            left: w.left + width / 2,
            top: w.top,
            right: w.right,
            bottom: w.top + height / 2,
        },
        LowerLeft => EngineRect {
            left: w.left,
            top: w.top + height / 2,
            right: w.left + width / 2,
            bottom: w.bottom,
        },
        LowerRight => EngineRect {
            left: w.left + width / 2,
            top: w.top + height / 2,
            right: w.right,
            bottom: w.bottom,
        },
        FirstThird => {
            let (l, c, r) = section_ratios(&params.thirds_layout_mode);
            let total = l + c + r;
            let right_x = w.left + (width * l) / total;
            EngineRect {
                left: w.left,
                top: w.top,
                right: right_x,
                bottom: w.bottom,
            }
        }
        FirstTwoThirds => {
            let (l, c, r) = section_ratios(&params.thirds_layout_mode);
            let total = l + c + r;
            let right_x = w.left + (width * (l + c)) / total;
            EngineRect {
                left: w.left,
                top: w.top,
                right: right_x,
                bottom: w.bottom,
            }
        }
        CenterThird => {
            let (l, c, r) = section_ratios(&params.thirds_layout_mode);
            let total = l + c + r;
            let left_x = w.left + (width * l) / total;
            let right_x = w.left + (width * (l + c)) / total;
            EngineRect {
                left: left_x,
                top: w.top,
                right: right_x,
                bottom: w.bottom,
            }
        }
        LastTwoThirds => {
            let (l, c, r) = section_ratios(&params.thirds_layout_mode);
            let total = l + c + r;
            let left_x = w.left + (width * l) / total;
            EngineRect {
                left: left_x,
                top: w.top,
                right: w.right,
                bottom: w.bottom,
            }
        }
        LastThird => {
            let (l, c, r) = section_ratios(&params.thirds_layout_mode);
            let total = l + c + r;
            let left_x = w.left + (width * (l + c)) / total;
            EngineRect {
                left: left_x,
                top: w.top,
                right: w.right,
                bottom: w.bottom,
            }
        }
        Undo | NextDisplay | PreviousDisplay => return None,
    };

    Some(CalculationResult {
        rect,
        resulting_action: params.action,
    })
}

/// Inset rect by gap (positive) or outset (negative). Dimension and Edge for shared edges; we use Both and None for simplicity like the C# default.
pub fn apply_gaps(rect: EngineRect, gap_size: f32) -> EngineRect {
    if gap_size == 0.0 {
        return rect;
    }
    let g = gap_size as i32;
    EngineRect {
        left: rect.left + g,
        top: rect.top + g,
        right: rect.right - g,
        bottom: rect.bottom - g,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_gaps, calculate, section_ratios, CalculationParams, WindowAction,
    };
    use crate::rect::EngineRect;

    fn params(action: WindowAction, thirds_layout_mode: &str) -> CalculationParams {
        CalculationParams {
            window_rect: EngineRect {
                left: 10,
                top: 20,
                right: 30,
                bottom: 40,
            },
            work_area: EngineRect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 90,
            },
            action,
            last_action: None,
            thirds_layout_mode: thirds_layout_mode.into(),
        }
    }

    #[test]
    fn window_action_string_conversions_cover_known_actions() {
        let actions = [
            ("LeftHalf", WindowAction::LeftHalf),
            ("RightHalf", WindowAction::RightHalf),
            ("TopHalf", WindowAction::TopHalf),
            ("BottomHalf", WindowAction::BottomHalf),
            ("Maximize", WindowAction::Maximize),
            ("Center", WindowAction::Center),
            ("Undo", WindowAction::Undo),
            ("Restore", WindowAction::Undo),
            ("LowerLeft", WindowAction::LowerLeft),
            ("LowerRight", WindowAction::LowerRight),
            ("UpperLeft", WindowAction::UpperLeft),
            ("UpperRight", WindowAction::UpperRight),
            ("NextDisplay", WindowAction::NextDisplay),
            ("PreviousDisplay", WindowAction::PreviousDisplay),
            ("FirstThird", WindowAction::FirstThird),
            ("FirstTwoThirds", WindowAction::FirstTwoThirds),
            ("CenterThird", WindowAction::CenterThird),
            ("LastTwoThirds", WindowAction::LastTwoThirds),
            ("LastThird", WindowAction::LastThird),
        ];

        for (name, action) in actions {
            assert_eq!(WindowAction::from_str(name), Some(action));
            if name != "Restore" {
                assert_eq!(action.name(), name);
            }
        }

        assert_eq!(WindowAction::from_str("MissingAction"), None);
    }

    #[test]
    fn action_categories_match_expected_behavior() {
        assert!(WindowAction::LeftHalf.has_calculation());
        assert!(WindowAction::CenterThird.has_calculation());
        assert!(!WindowAction::Undo.has_calculation());
        assert!(!WindowAction::NextDisplay.has_calculation());
        assert!(!WindowAction::PreviousDisplay.has_calculation());

        assert!(WindowAction::FirstThird.is_section_action());
        assert!(WindowAction::CenterThird.is_section_action());
        assert!(WindowAction::LastThird.is_section_action());
        assert!(!WindowAction::LeftHalf.is_section_action());
        assert!(!WindowAction::Undo.is_section_action());
    }

    #[test]
    fn section_ratios_normalize_layout_modes() {
        assert_eq!(section_ratios("Thirds"), (1, 1, 1));
        assert_eq!(section_ratios("Fourths"), (1, 2, 1));
        assert_eq!(section_ratios("Fifths"), (1, 3, 1));
        assert_eq!(section_ratios("unknown"), (1, 1, 1));
    }

    #[test]
    fn calculate_returns_expected_rects_for_core_actions() {
        let left = calculate(&params(WindowAction::LeftHalf, "Thirds")).unwrap();
        assert_eq!(
            left.rect,
            EngineRect {
                left: 0,
                top: 0,
                right: 50,
                bottom: 90,
            }
        );
        assert_eq!(left.resulting_action, WindowAction::LeftHalf);

        let center = calculate(&params(WindowAction::Center, "Thirds")).unwrap();
        assert_eq!(
            center.rect,
            EngineRect {
                left: 40,
                top: 35,
                right: 60,
                bottom: 55,
            }
        );

        let maximize = calculate(&params(WindowAction::Maximize, "Thirds")).unwrap();
        assert_eq!(
            maximize.rect,
            EngineRect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 90,
            }
        );
    }

    #[test]
    fn calculate_uses_section_layout_ratios() {
        let first = calculate(&params(WindowAction::FirstThird, "Fifths")).unwrap();
        assert_eq!(first.rect.right, 20);

        let first_two = calculate(&params(WindowAction::FirstTwoThirds, "Fifths")).unwrap();
        assert_eq!(first_two.rect.right, 80);

        let center = calculate(&params(WindowAction::CenterThird, "Fifths")).unwrap();
        assert_eq!(
            center.rect,
            EngineRect {
                left: 20,
                top: 0,
                right: 80,
                bottom: 90,
            }
        );

        let last = calculate(&params(WindowAction::LastThird, "Fifths")).unwrap();
        assert_eq!(
            last.rect,
            EngineRect {
                left: 80,
                top: 0,
                right: 100,
                bottom: 90,
            }
        );
    }

    #[test]
    fn calculate_returns_none_for_non_calculated_actions() {
        assert!(calculate(&params(WindowAction::Undo, "Thirds")).is_none());
        assert!(calculate(&params(WindowAction::NextDisplay, "Thirds")).is_none());
        assert!(calculate(&params(WindowAction::PreviousDisplay, "Thirds")).is_none());
    }

    #[test]
    fn apply_gaps_supports_positive_negative_and_zero_values() {
        let rect = EngineRect {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        };

        assert_eq!(apply_gaps(rect, 0.0), rect);
        assert_eq!(
            apply_gaps(rect, 4.0),
            EngineRect {
                left: 4,
                top: 4,
                right: 96,
                bottom: 96,
            }
        );
        assert_eq!(
            apply_gaps(rect, -2.0),
            EngineRect {
                left: -2,
                top: -2,
                right: 102,
                bottom: 102,
            }
        );
    }
}
