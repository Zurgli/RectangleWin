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
