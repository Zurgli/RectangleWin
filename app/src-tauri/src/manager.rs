//! Window manager: execute actions (halves, thirds, etc.) with history for Undo.
//! Windows-only; uses win32 and engine.

#[cfg(windows)]
use crate::config::Config;
#[cfg(windows)]
use crate::engine::{self, apply_gaps, calculate, CalculationParams, WindowAction};
#[cfg(windows)]
use crate::rect::{EngineRect, Rect};
#[cfg(windows)]
use crate::win32::{
    enum_monitors, get_cursor_pos, get_foreground_window, get_monitor_from_point,
    get_monitor_from_window, get_process_image_name, set_cursor_pos, set_foreground_window,
    set_window_bounds, try_get_monitor_info, try_get_window_bounds,
};
#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::time::{Duration, Instant};

/// Options for Execute (from config).
#[derive(Clone, Debug)]
pub struct ExecuteOptions {
    pub use_cursor_screen: bool,
    pub move_cursor_after_snap: bool,
    pub move_cursor_across_displays: bool,
    pub gap_size: f32,
    pub update_restore_rect: bool,
    pub disabled_process_names: Option<Vec<String>>,
    pub screen_edge_gap_top: f32,
    pub screen_edge_gap_bottom: f32,
    pub screen_edge_gap_left: f32,
    pub screen_edge_gap_right: f32,
    pub screen_edge_gaps_on_main_screen_only: bool,
    pub taskbar_gap_compensation: i32,
    pub taskbar_gap_compensation_left: i32,
    pub taskbar_gap_compensation_right: i32,
    pub apply_gaps_to_maximize: bool,
    pub apply_gaps_to_maximize_height: bool,
    pub thirds_layout_mode: String,
}

impl Default for ExecuteOptions {
    fn default() -> Self {
        Self {
            use_cursor_screen: false,
            move_cursor_after_snap: false,
            move_cursor_across_displays: false,
            gap_size: 0.0,
            update_restore_rect: true,
            disabled_process_names: None,
            screen_edge_gap_top: 0.0,
            screen_edge_gap_bottom: 0.0,
            screen_edge_gap_left: 0.0,
            screen_edge_gap_right: 0.0,
            screen_edge_gaps_on_main_screen_only: false,
            taskbar_gap_compensation: 0,
            taskbar_gap_compensation_left: 0,
            taskbar_gap_compensation_right: 0,
            apply_gaps_to_maximize: true,
            apply_gaps_to_maximize_height: true,
            thirds_layout_mode: "Thirds".into(),
        }
    }
}

impl From<&Config> for ExecuteOptions {
    fn from(c: &Config) -> Self {
        Self {
            gap_size: c.gap_size,
            screen_edge_gap_top: c.screen_edge_gap_top,
            screen_edge_gap_bottom: c.screen_edge_gap_bottom,
            screen_edge_gap_left: c.screen_edge_gap_left,
            screen_edge_gap_right: c.screen_edge_gap_right,
            screen_edge_gaps_on_main_screen_only: c.screen_edge_gaps_on_main_screen_only,
            taskbar_gap_compensation: c.taskbar_gap_compensation,
            taskbar_gap_compensation_left: c.taskbar_gap_compensation_left,
            taskbar_gap_compensation_right: c.taskbar_gap_compensation_right,
            apply_gaps_to_maximize: c.apply_gaps_to_maximize,
            apply_gaps_to_maximize_height: c.apply_gaps_to_maximize_height,
            thirds_layout_mode: c.thirds_layout.clone(),
            ..Default::default()
        }
    }
}

#[cfg(windows)]
fn hwnd_to_key(hwnd: windows::Win32::Foundation::HWND) -> isize {
    hwnd.0 as isize
}

/// True if the window is in a free (non-snapped) state: no last action, or current bounds differ from last applied rect.
#[cfg(windows)]
fn is_window_free(manager: &WindowManager, key: isize, current_rect: &Rect) -> bool {
    const TOLERANCE: i32 = 5;
    match manager.last_actions.get(&key) {
        None => true,
        Some((_, last_rect)) => !current_rect.approximately_equals(last_rect, TOLERANCE),
    }
}

#[cfg(windows)]
fn inset_work_area_by_screen_edge_gaps(work: Rect, opts: &ExecuteOptions) -> Rect {
    let t = opts.screen_edge_gap_top as i32;
    let b = opts.screen_edge_gap_bottom as i32;
    let l = opts.screen_edge_gap_left as i32;
    let r = opts.screen_edge_gap_right as i32;
    if t == 0 && b == 0 && l == 0 && r == 0 {
        return work;
    }
    Rect {
        left: work.left + l,
        top: work.top + t,
        right: work.right - r,
        bottom: work.bottom - b,
    }
}

#[cfg(windows)]
fn get_current_and_adjacent_work_areas(
    hwnd: windows::Win32::Foundation::HWND,
    use_cursor_screen: bool,
) -> (Rect, Option<Rect>, Option<Rect>) {
    let monitors = enum_monitors();
    if monitors.is_empty() {
        return (Rect::default(), None, None);
    }
    let current_hmon = if use_cursor_screen {
        if let Some((x, y)) = get_cursor_pos() {
            get_monitor_from_point(x, y)
        } else {
            get_monitor_from_window(hwnd)
        }
    } else {
        get_monitor_from_window(hwnd)
    };
    let (_mon, current_work) =
        try_get_monitor_info(current_hmon).unwrap_or((Rect::default(), Rect::default()));
    let mut idx = None;
    let current_raw = current_hmon.0;
    for (i, m) in monitors.iter().enumerate() {
        if m.hmonitor.0 == current_raw {
            idx = Some(i);
            break;
        }
    }
    let (prev, next) = match idx {
        Some(i) => (
            if i > 0 {
                Some(monitors[i - 1].work_area)
            } else {
                None
            },
            if i + 1 < monitors.len() {
                Some(monitors[i + 1].work_area)
            } else {
                None
            },
        ),
        None => (None, None),
    };
    (current_work, prev, next)
}

/// Cycle order for repeat section hotkey: Fourths -> Fifths -> Thirds -> Fourths.
const SECTION_LAYOUT_CYCLE: &[&str] = &["Fourths", "Fifths", "Thirds"];
#[cfg(windows)]
const SECTION_CYCLE_RESET_TIMEOUT: Duration = Duration::from_millis(1500);

fn section_layout_cycle_index(layout: &str) -> usize {
    if layout.eq_ignore_ascii_case("Fifths") {
        1
    } else if layout.eq_ignore_ascii_case("Thirds") {
        2
    } else {
        0
    } // Fourths or default
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SectionCycleSession {
    window_key: isize,
    action: WindowAction,
    next_layout_index: usize,
    last_triggered_at: Instant,
}

#[cfg(windows)]
fn resolve_section_layout_mode(
    session: Option<SectionCycleSession>,
    window_key: isize,
    action: WindowAction,
    configured_layout: &str,
    now: Instant,
) -> (String, Option<SectionCycleSession>) {
    if !action.is_section_action() {
        return (configured_layout.to_string(), None);
    }

    if let Some(existing) = session {
        if existing.window_key == window_key
            && existing.action == action
            && matches!(
                now.checked_duration_since(existing.last_triggered_at),
                Some(elapsed) if elapsed <= SECTION_CYCLE_RESET_TIMEOUT
            )
        {
            let layout = SECTION_LAYOUT_CYCLE[existing.next_layout_index].to_string();
            let next_session = SectionCycleSession {
                next_layout_index: (existing.next_layout_index + 1) % SECTION_LAYOUT_CYCLE.len(),
                last_triggered_at: now,
                ..existing
            };
            return (layout, Some(next_session));
        }
    }

    let next_layout_index =
        (section_layout_cycle_index(configured_layout) + 1) % SECTION_LAYOUT_CYCLE.len();
    let next_session = SectionCycleSession {
        window_key,
        action,
        next_layout_index,
        last_triggered_at: now,
    };
    (configured_layout.to_string(), Some(next_session))
}

#[cfg(windows)]
pub struct WindowManager {
    restore_rects: HashMap<isize, Rect>,
    last_actions: HashMap<isize, (WindowAction, Rect)>,
    section_cycle_session: Option<SectionCycleSession>,
}

#[cfg(windows)]
impl WindowManager {
    pub fn new() -> Self {
        Self {
            restore_rects: HashMap::new(),
            last_actions: HashMap::new(),
            section_cycle_session: None,
        }
    }

    /// Restore rect for the given window key (for move/size-end hook).
    pub fn get_restore_rect(&self, key: isize) -> Option<Rect> {
        self.restore_rects.get(&key).copied()
    }

    /// Last applied rect for the given window key (for move/size-end hook).
    pub fn get_last_action_rect(&self, key: isize) -> Option<Rect> {
        self.last_actions.get(&key).map(|(_, r)| *r)
    }

    /// Remove last action for the given window (after restoring on unsnap).
    pub fn remove_last_action(&mut self, key: isize) {
        self.last_actions.remove(&key);
    }

    /// Execute action on foreground window (or None to use foreground). Returns true if applied.
    pub fn execute(
        &mut self,
        action: WindowAction,
        _hwnd_override: Option<windows::Win32::Foundation::HWND>,
        options: &ExecuteOptions,
    ) -> bool {
        let hwnd = match _hwnd_override.or_else(get_foreground_window) {
            Some(h) => h,
            None => return false,
        };
        let key = hwnd_to_key(hwnd);

        if let Some(ref names) = options.disabled_process_names {
            if !names.is_empty() {
                if let Some(name) = get_process_image_name(hwnd) {
                    if names
                        .iter()
                        .any(|n: &String| n.eq_ignore_ascii_case(name.as_str()))
                    {
                        return false;
                    }
                }
            }
        }

        if action == WindowAction::Undo {
            self.section_cycle_session = None;
            let restore = match self.restore_rects.get(&key) {
                Some(r) => *r,
                None => return false,
            };
            let ok = set_window_bounds(hwnd, &restore, false, false);
            if ok {
                self.last_actions.remove(&key);
            }
            return ok;
        }

        if action == WindowAction::NextDisplay || action == WindowAction::PreviousDisplay {
            self.section_cycle_session = None;
            let current_rect = match try_get_window_bounds(hwnd, false) {
                Some(r) => r,
                None => return false,
            };
            let (_work_area, _prev, _next) =
                get_current_and_adjacent_work_areas(hwnd, options.use_cursor_screen);
            let target_work = if action == WindowAction::NextDisplay {
                _next
            } else {
                _prev
            };
            let mut dest = match target_work {
                Some(d) => d,
                None => return false,
            };
            dest = inset_work_area_by_screen_edge_gaps(dest, options);
            if options.update_restore_rect && is_window_free(self, key, &current_rect) {
                self.restore_rects.insert(key, current_rect);
            }
            let mut engine_rect = EngineRect::from_rect(&dest);
            if options.gap_size != 0.0 {
                engine_rect = apply_gaps(engine_rect, options.gap_size);
            }
            let dest_rect: Rect = engine_rect.into();
            let ok = set_window_bounds(hwnd, &dest_rect, false, false);
            if ok {
                self.last_actions
                    .insert(key, (WindowAction::Maximize, dest_rect));
                if options.move_cursor_across_displays {
                    set_cursor_pos(
                        dest_rect.left + dest_rect.width() / 2,
                        dest_rect.top + dest_rect.height() / 2,
                    );
                }
                set_foreground_window(hwnd);
            }
            return ok;
        }

        if !action.has_calculation() {
            return false;
        }

        let window_rect = match try_get_window_bounds(hwnd, false) {
            Some(r) => r,
            None => return false,
        };
        let (work_area, _, _) =
            get_current_and_adjacent_work_areas(hwnd, options.use_cursor_screen);
        let work = inset_work_area_by_screen_edge_gaps(work_area, options);
        if work.is_empty() {
            return false;
        }

        let last_info = self
            .last_actions
            .get(&key)
            .map(|(a, r)| engine::LastActionInfo {
                rect: EngineRect::from_rect(r),
                action: *a,
            });

        let (thirds_layout_mode, next_section_cycle_session) = resolve_section_layout_mode(
            self.section_cycle_session,
            key,
            action,
            &options.thirds_layout_mode,
            Instant::now(),
        );
        self.section_cycle_session = next_section_cycle_session;

        let params = CalculationParams {
            window_rect: EngineRect::from_rect(&window_rect),
            work_area: EngineRect::from_rect(&work),
            action,
            last_action: last_info,
            thirds_layout_mode,
        };

        let result = match calculate(&params) {
            Some(r) => r,
            None => return false,
        };

        let mut target_rect = result.rect;
        let mut do_apply_gaps = options.gap_size != 0.0;
        if do_apply_gaps && action == WindowAction::Maximize && !options.apply_gaps_to_maximize {
            do_apply_gaps = false;
        }
        if do_apply_gaps && action == WindowAction::Center {
            do_apply_gaps = false;
        }
        if do_apply_gaps {
            target_rect = apply_gaps(target_rect, options.gap_size);
        }

        if options.update_restore_rect && is_window_free(self, key, &window_rect) {
            self.restore_rects.insert(key, window_rect);
        }

        let target: Rect = target_rect.into();
        let applied = set_window_bounds(hwnd, &target, false, true);
        if applied {
            self.last_actions
                .insert(key, (result.resulting_action, target));
            if options.move_cursor_after_snap {
                set_cursor_pos(
                    target.left + target.width() / 2,
                    target.top + target.height() / 2,
                );
            }
        }
        applied
    }
}

#[cfg(not(windows))]
pub struct WindowManager;

#[cfg(not(windows))]
impl WindowManager {
    pub fn new() -> Self {
        Self
    }
    pub fn execute(
        &mut self,
        _action: engine::WindowAction,
        _hwnd: Option<()>,
        _options: &ExecuteOptions,
    ) -> bool {
        false
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::{
        resolve_section_layout_mode, section_layout_cycle_index, SectionCycleSession,
        SECTION_CYCLE_RESET_TIMEOUT, SECTION_LAYOUT_CYCLE,
    };
    use crate::engine::WindowAction;
    use std::time::{Duration, Instant};

    #[test]
    fn first_section_action_uses_configured_layout_and_seeds_next_layout() {
        let now = Instant::now();
        let (layout, session) =
            resolve_section_layout_mode(None, 1, WindowAction::FirstThird, "Thirds", now);

        assert_eq!(layout, "Thirds");
        assert_eq!(
            session,
            Some(SectionCycleSession {
                window_key: 1,
                action: WindowAction::FirstThird,
                next_layout_index: 0,
                last_triggered_at: now,
            })
        );
    }

    #[test]
    fn repeated_same_action_same_window_cycles_within_timeout() {
        let now = Instant::now();
        let session = Some(SectionCycleSession {
            window_key: 1,
            action: WindowAction::FirstThird,
            next_layout_index: 0,
            last_triggered_at: now,
        });

        let (layout, next_session) = resolve_section_layout_mode(
            session,
            1,
            WindowAction::FirstThird,
            "Thirds",
            now + Duration::from_millis(250),
        );

        assert_eq!(layout, "Fourths");
        assert_eq!(
            next_session,
            Some(SectionCycleSession {
                window_key: 1,
                action: WindowAction::FirstThird,
                next_layout_index: 1,
                last_triggered_at: now + Duration::from_millis(250),
            })
        );
    }

    #[test]
    fn repeat_after_timeout_resets_to_configured_layout() {
        let now = Instant::now();
        let session = Some(SectionCycleSession {
            window_key: 1,
            action: WindowAction::FirstThird,
            next_layout_index: 1,
            last_triggered_at: now,
        });

        let (layout, next_session) = resolve_section_layout_mode(
            session,
            1,
            WindowAction::FirstThird,
            "Thirds",
            now + SECTION_CYCLE_RESET_TIMEOUT + Duration::from_millis(1),
        );

        assert_eq!(layout, "Thirds");
        assert_eq!(
            next_session,
            Some(SectionCycleSession {
                window_key: 1,
                action: WindowAction::FirstThird,
                next_layout_index: 0,
                last_triggered_at: now + SECTION_CYCLE_RESET_TIMEOUT + Duration::from_millis(1),
            })
        );
    }

    #[test]
    fn different_window_resets_to_configured_layout() {
        let now = Instant::now();
        let session = Some(SectionCycleSession {
            window_key: 1,
            action: WindowAction::FirstThird,
            next_layout_index: 2,
            last_triggered_at: now,
        });

        let (layout, next_session) = resolve_section_layout_mode(
            session,
            2,
            WindowAction::FirstThird,
            "Fourths",
            now + Duration::from_millis(100),
        );

        assert_eq!(layout, "Fourths");
        assert_eq!(
            next_session,
            Some(SectionCycleSession {
                window_key: 2,
                action: WindowAction::FirstThird,
                next_layout_index: 1,
                last_triggered_at: now + Duration::from_millis(100),
            })
        );
    }

    #[test]
    fn different_section_action_resets_to_configured_layout() {
        let now = Instant::now();
        let session = Some(SectionCycleSession {
            window_key: 1,
            action: WindowAction::FirstThird,
            next_layout_index: 2,
            last_triggered_at: now,
        });

        let (layout, next_session) = resolve_section_layout_mode(
            session,
            1,
            WindowAction::CenterThird,
            "Fifths",
            now + Duration::from_millis(100),
        );

        assert_eq!(layout, "Fifths");
        assert_eq!(
            next_session,
            Some(SectionCycleSession {
                window_key: 1,
                action: WindowAction::CenterThird,
                next_layout_index: 2,
                last_triggered_at: now + Duration::from_millis(100),
            })
        );
    }

    #[test]
    fn non_section_action_clears_active_session() {
        let now = Instant::now();
        let session = Some(SectionCycleSession {
            window_key: 1,
            action: WindowAction::FirstThird,
            next_layout_index: 0,
            last_triggered_at: now,
        });

        let (layout, next_session) = resolve_section_layout_mode(
            session,
            1,
            WindowAction::LeftHalf,
            "Thirds",
            now + Duration::from_millis(100),
        );

        assert_eq!(layout, "Thirds");
        assert_eq!(next_session, None);
    }

    #[test]
    fn timeout_boundary_includes_exact_timeout_only() {
        let now = Instant::now();
        let session = Some(SectionCycleSession {
            window_key: 1,
            action: WindowAction::FirstThird,
            next_layout_index: 1,
            last_triggered_at: now,
        });

        let (exact_layout, exact_session) = resolve_section_layout_mode(
            session,
            1,
            WindowAction::FirstThird,
            "Thirds",
            now + SECTION_CYCLE_RESET_TIMEOUT,
        );
        assert_eq!(exact_layout, SECTION_LAYOUT_CYCLE[1]);
        let exact_session = exact_session.unwrap();
        assert_eq!(exact_session.next_layout_index, 2);

        let (late_layout, late_session) = resolve_section_layout_mode(
            Some(exact_session),
            1,
            WindowAction::FirstThird,
            "Thirds",
            exact_session.last_triggered_at
                + SECTION_CYCLE_RESET_TIMEOUT
                + Duration::from_millis(1),
        );
        assert_eq!(late_layout, "Thirds");
        assert_eq!(late_session.unwrap().next_layout_index, 0);
    }

    #[test]
    fn configured_starting_modes_seed_expected_next_layout() {
        let now = Instant::now();

        for layout in ["Thirds", "Fourths", "Fifths"] {
            let (resolved_layout, session) =
                resolve_section_layout_mode(None, 1, WindowAction::LastThird, layout, now);
            assert_eq!(resolved_layout, layout);
            assert_eq!(
                session.unwrap().next_layout_index,
                (section_layout_cycle_index(layout) + 1) % SECTION_LAYOUT_CYCLE.len()
            );
        }
    }
}
