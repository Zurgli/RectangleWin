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
    FirstThird,
    FirstTwoThirds,
    CenterThird,
    LastTwoThirds,
    LastThird,
    CenterTwoThirds,
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
            "CenterTwoThirds" => Some(Self::CenterTwoThirds),
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
            Self::CenterTwoThirds => "CenterTwoThirds",
        }
    }

    pub fn has_calculation(&self) -> bool {
        !matches!(
            self,
            Self::Undo | Self::NextDisplay | Self::PreviousDisplay
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

fn is_fifths(mode: &str) -> bool {
    mode.eq_ignore_ascii_case("Fifths")
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
            if is_fifths(&params.thirds_layout_mode) {
                let unit = width / 5;
                EngineRect {
                    left: w.left,
                    top: w.top,
                    right: w.left + unit,
                    bottom: w.bottom,
                }
            } else {
                let third = width / 3;
                EngineRect {
                    left: w.left,
                    top: w.top,
                    right: w.left + third,
                    bottom: w.bottom,
                }
            }
        }
        FirstTwoThirds => {
            if is_fifths(&params.thirds_layout_mode) {
                let unit = width / 5;
                EngineRect {
                    left: w.left,
                    top: w.top,
                    right: w.left + 4 * unit,
                    bottom: w.bottom,
                }
            } else {
                let third = width / 3;
                EngineRect {
                    left: w.left,
                    top: w.top,
                    right: w.left + 2 * third,
                    bottom: w.bottom,
                }
            }
        }
        CenterThird => {
            if is_fifths(&params.thirds_layout_mode) {
                let unit = width / 5;
                EngineRect {
                    left: w.left + unit,
                    top: w.top,
                    right: w.left + 4 * unit,
                    bottom: w.bottom,
                }
            } else {
                let third = width / 3;
                EngineRect {
                    left: w.left + third,
                    top: w.top,
                    right: w.left + 2 * third,
                    bottom: w.bottom,
                }
            }
        }
        LastTwoThirds => {
            if is_fifths(&params.thirds_layout_mode) {
                let unit = width / 5;
                EngineRect {
                    left: w.left + unit,
                    top: w.top,
                    right: w.right,
                    bottom: w.bottom,
                }
            } else {
                let third = width / 3;
                EngineRect {
                    left: w.left + third,
                    top: w.top,
                    right: w.right,
                    bottom: w.bottom,
                }
            }
        }
        LastThird => {
            if is_fifths(&params.thirds_layout_mode) {
                let unit = width / 5;
                EngineRect {
                    left: w.left + 4 * unit,
                    top: w.top,
                    right: w.right,
                    bottom: w.bottom,
                }
            } else {
                let third = width / 3;
                EngineRect {
                    left: w.left + 2 * third,
                    top: w.top,
                    right: w.right,
                    bottom: w.bottom,
                }
            }
        }
        CenterTwoThirds => {
            if is_fifths(&params.thirds_layout_mode) {
                let tenth = width / 10;
                EngineRect {
                    left: w.left + tenth,
                    top: w.top,
                    right: w.left + 9 * tenth,
                    bottom: w.bottom,
                }
            } else {
                let sixth = width / 6;
                EngineRect {
                    left: w.left + sixth,
                    top: w.top,
                    right: w.left + 5 * sixth,
                    bottom: w.bottom,
                }
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
