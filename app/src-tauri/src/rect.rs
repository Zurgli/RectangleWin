//! Rectangle types: Win32 RECT and engine Rect with conversions.

/// Win32 RECT (left, top, right, bottom) in screen coordinates.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn is_empty(&self) -> bool {
        self.left == 0 && self.top == 0 && self.right == 0 && self.bottom == 0
    }

    /// True if this rect matches other within tolerance (for "window is still snapped" check).
    pub fn approximately_equals(&self, other: &Rect, tolerance: i32) -> bool {
        (self.left - other.left).abs() <= tolerance
            && (self.top - other.top).abs() <= tolerance
            && (self.right - other.right).abs() <= tolerance
            && (self.bottom - other.bottom).abs() <= tolerance
    }
}

/// Platform-agnostic rect for engine calculations (same as C# WindowEngine.Rect).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EngineRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl EngineRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.left == 0 && self.top == 0 && self.right == 0 && self.bottom == 0
    }

    pub fn from_rect(r: &Rect) -> Self {
        Self {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }

    pub fn to_rect(&self) -> Rect {
        Rect {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
        }
    }
}

impl From<Rect> for EngineRect {
    fn from(r: Rect) -> Self {
        Self::from_rect(&r)
    }
}

impl From<EngineRect> for Rect {
    fn from(r: EngineRect) -> Self {
        r.to_rect()
    }
}

#[cfg(test)]
mod tests {
    use super::{EngineRect, Rect};

    #[test]
    fn rect_dimensions_and_empty_state_are_reported_correctly() {
        let rect = Rect {
            left: 10,
            top: 15,
            right: 50,
            bottom: 65,
        };
        assert_eq!(rect.width(), 40);
        assert_eq!(rect.height(), 50);
        assert!(!rect.is_empty());
        assert!(Rect::default().is_empty());
    }

    #[test]
    fn rect_tolerance_check_respects_threshold() {
        let a = Rect {
            left: 10,
            top: 10,
            right: 100,
            bottom: 100,
        };
        let b = Rect {
            left: 12,
            top: 9,
            right: 103,
            bottom: 98,
        };

        assert!(a.approximately_equals(&b, 3));
        assert!(!a.approximately_equals(&b, 1));
    }

    #[test]
    fn engine_rect_dimensions_and_empty_state_are_reported_correctly() {
        let rect = EngineRect {
            left: 5,
            top: 7,
            right: 25,
            bottom: 31,
        };
        assert_eq!(rect.width(), 20);
        assert_eq!(rect.height(), 24);
        assert!(!rect.is_empty());
        assert!(EngineRect::default().is_empty());
    }

    #[test]
    fn rect_conversions_round_trip() {
        let rect = Rect {
            left: 1,
            top: 2,
            right: 30,
            bottom: 40,
        };

        let engine = EngineRect::from_rect(&rect);
        assert_eq!(
            engine,
            EngineRect {
                left: 1,
                top: 2,
                right: 30,
                bottom: 40,
            }
        );
        assert_eq!(engine.to_rect(), rect);
        assert_eq!(EngineRect::from(rect), engine);
        assert_eq!(Rect::from(engine), rect);
    }
}
