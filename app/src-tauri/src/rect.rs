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
