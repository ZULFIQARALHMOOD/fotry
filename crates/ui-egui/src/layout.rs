//! Shared responsive layout thresholds and sizing rules.

use egui::{Context, Id, Pos2, Rect, Vec2, pos2, vec2};

pub const PHONE_MAX_WIDTH: f32 = 600.0;
pub const COMPACT_MAX_WIDTH: f32 = 1100.0;
pub const TOUCH_TARGET: f32 = 44.0;
pub const VIEWPORT_MARGIN: f32 = 12.0;
pub const DRAWER_MAX_WIDTH: f32 = 360.0;
pub const RIBBON_CONTENT_HEIGHT: f32 = 66.0;
pub const RIBBON_LABEL_HEIGHT: f32 = 18.0;
pub const COMPACT_TITLE_HEIGHT: f32 = TOUCH_TARGET + 4.0;
pub const COMPACT_TABS_HEIGHT: f32 = TOUCH_TARGET + 4.0;
pub const DESKTOP_TITLE_HEIGHT: f32 = 38.0;
pub const DESKTOP_TABS_HEIGHT: f32 = 30.0;
pub const STATUS_HEIGHT: f32 = 26.0;

fn touch_device_id() -> Id {
    Id::new("layout_touch_device")
}

#[cfg(test)]
fn frame_rects_id() -> Id {
    Id::new("layout_frame_rects")
}

pub fn is_phone(ctx: &Context) -> bool {
    ctx.content_rect().width() < PHONE_MAX_WIDTH
}

pub fn is_compact(ctx: &Context) -> bool {
    ctx.content_rect().width() < COMPACT_MAX_WIDTH
}

pub fn touch_device(ctx: &Context) -> bool {
    if ctx.input(|input| input.any_touches()) {
        ctx.data_mut(|data| data.insert_temp(touch_device_id(), true));
    }
    ctx.data(|data| data.get_temp::<bool>(touch_device_id()).unwrap_or(false))
}

pub fn set_touch_device(ctx: &Context, enabled: bool) {
    ctx.data_mut(|data| data.insert_temp(touch_device_id(), enabled));
}

pub fn apply_touch_spacing(ctx: &Context) {
    if !touch_device(ctx) {
        return;
    }
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.style_mut_of(theme, |style| {
            style.spacing.interact_size = style.spacing.interact_size.max(vec2(TOUCH_TARGET, TOUCH_TARGET));
            style.spacing.button_padding = style.spacing.button_padding.max(vec2(8.0, 6.0));
        });
    }
}

pub fn target_size(ctx: &Context, size: Vec2) -> Vec2 {
    if touch_device(ctx) { size.max(Vec2::splat(TOUCH_TARGET)) } else { size }
}

pub fn ribbon_content_height(ctx: &Context) -> f32 {
    if touch_device(ctx) { (3.0 * TOUCH_TARGET + 2.0).max(RIBBON_CONTENT_HEIGHT) } else { RIBBON_CONTENT_HEIGHT }
}

pub fn ribbon_height(ctx: &Context) -> f32 {
    ribbon_content_height(ctx) + RIBBON_LABEL_HEIGHT + 10.0
}

pub fn dialog_max_size(ctx: &Context) -> Vec2 {
    dialog_area(ctx, false).size()
}

pub fn dialog_area(ctx: &Context, ribbon_collapsed: bool) -> Rect {
    let viewport = ctx.content_rect();
    let compact = is_compact(ctx);
    let title = if compact { COMPACT_TITLE_HEIGHT } else { target_size(ctx, vec2(1.0, DESKTOP_TITLE_HEIGHT)).y };
    let tabs = if compact { COMPACT_TABS_HEIGHT } else { target_size(ctx, vec2(1.0, DESKTOP_TABS_HEIGHT)).y };
    let ribbon = if ribbon_collapsed { 0.0 } else { ribbon_height(ctx) };
    let top = (viewport.top() + title + tabs + ribbon + VIEWPORT_MARGIN).min(viewport.bottom());
    let status = target_size(ctx, vec2(1.0, STATUS_HEIGHT)).y;
    let bottom = (viewport.bottom() - status - VIEWPORT_MARGIN).max(top);
    Rect::from_min_max(pos2(viewport.left() + VIEWPORT_MARGIN, top), pos2(viewport.right() - VIEWPORT_MARGIN, bottom))
}

pub fn drawer_size(ctx: &Context) -> Vec2 {
    let rect = ctx.content_rect();
    vec2(DRAWER_MAX_WIDTH.min((rect.width() - 2.0 * VIEWPORT_MARGIN).max(0.0)), (rect.height() - 2.0 * VIEWPORT_MARGIN).max(0.0))
}

pub fn popup_position(anchor: Rect, size: Vec2, viewport: Rect) -> Pos2 {
    let max_size = vec2((viewport.width() - 2.0 * VIEWPORT_MARGIN).max(0.0), (viewport.height() - 2.0 * VIEWPORT_MARGIN).max(0.0));
    let size = size.min(max_size);
    let below = anchor.bottom() + size.y <= viewport.bottom() - VIEWPORT_MARGIN;
    let y = if below { anchor.bottom() } else { anchor.top() - size.y };
    let clamp_origin = |value: f32, min: f32, max: f32| {
        let min = min.min(max);
        let max = max.max(min);
        value.clamp(min, max)
    };
    pos2(
        clamp_origin(anchor.left(), viewport.left() + VIEWPORT_MARGIN, viewport.right() - size.x - VIEWPORT_MARGIN),
        clamp_origin(y, viewport.top() + VIEWPORT_MARGIN, viewport.bottom() - size.y - VIEWPORT_MARGIN),
    )
}

pub fn clear_frame_rects(ctx: &Context) {
    #[cfg(test)]
    ctx.data_mut(|data| data.remove::<Vec<(String, Rect)>>(frame_rects_id()));
    #[cfg(not(test))]
    let _ = ctx;
}

pub fn record_rect(ctx: &Context, name: &str, rect: Rect) {
    #[cfg(test)]
    ctx.data_mut(|data| {
        let mut rects = data.get_temp::<Vec<(String, Rect)>>(frame_rects_id()).unwrap_or_default();
        rects.push((name.to_string(), rect));
        data.insert_temp(frame_rects_id(), rects);
    });
    #[cfg(not(test))]
    let _ = (ctx, name, rect);
}

#[cfg(test)]
pub fn frame_rects(ctx: &Context) -> Vec<(String, Rect)> {
    ctx.data(|data| data.get_temp::<Vec<(String, Rect)>>(frame_rects_id()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popups_flip_and_stay_inside_viewport() {
        let viewport = Rect::from_min_size(Pos2::ZERO, vec2(360.0, 640.0));
        let size = vec2(240.0, 280.0);
        let below = popup_position(Rect::from_min_size(pos2(30.0, 40.0), vec2(100.0, 30.0)), size, viewport);
        assert_eq!(below, pos2(30.0, 70.0));

        let above = popup_position(Rect::from_min_size(pos2(310.0, 580.0), vec2(40.0, 30.0)), size, viewport);
        assert_eq!(above, pos2(108.0, 300.0));
        assert!(Rect::from_min_size(above, size).is_positive());
        assert!(viewport.contains_rect(Rect::from_min_size(above, size)));
    }

    #[test]
    fn responsive_breakpoints_and_size_limits_cover_requested_viewports() {
        let expected = [(360.0, 640.0, true, true), (768.0, 1024.0, true, false), (1024.0, 768.0, true, false), (1920.0, 1080.0, false, false)];
        for (width, height, compact, phone) in expected {
            let ctx = Context::default();
            let rect = Rect::from_min_size(Pos2::ZERO, vec2(width, height));
            ctx.run_ui(egui::RawInput { screen_rect: Some(rect), ..Default::default() }, |_| {}).drop_without_applying_deltas();
            assert_eq!(is_compact(&ctx), compact, "compact breakpoint at {width}x{height}");
            assert_eq!(is_phone(&ctx), phone, "phone breakpoint at {width}x{height}");
            assert!(rect.contains_rect(dialog_area(&ctx, false)));
            assert!(drawer_size(&ctx).x <= DRAWER_MAX_WIDTH);
        }
    }

    #[test]
    fn app_chrome_and_dialogs_fit_viewports_in_ltr_and_rtl() {
        let cases = [(360.0, 640.0), (768.0, 1024.0), (1024.0, 768.0), (1920.0, 1080.0)];
        for (width, height) in cases {
            for language in ["en-US", "ar"] {
                let screen = Rect::from_min_size(Pos2::ZERO, vec2(width, height));
                let mut app = crate::WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
                app.ui.language = language.to_string();
                let ctx = Context::default();
                crate::theme::install_fonts_with(&ctx, false, false);
                set_touch_device(&ctx, true);
                app.run("ui.dialog", serde_json::json!({"name": "insertTable"})).expect("insert table dialog");

                for frame in 0..6 {
                    clear_frame_rects(&ctx);
                    let input = egui::RawInput { screen_rect: Some(screen), time: Some(f64::from(frame) / 60.0), ..Default::default() };
                    let output = ctx.run_ui(input, |ui| {
                        app.logic(ui.ctx());
                        app.ui(ui);
                    });
                    output.drop_without_applying_deltas();
                }

                let rects = frame_rects(&ctx);
                let rect = |name: &str| rects.iter().find(|(key, _)| key == name).map(|(_, rect)| *rect);
                let title = rect("title_bar").expect("title bar rect");
                let tabs = rect("tabs").expect("tab strip rect");
                let ribbon = rect("ribbon").expect("ribbon rect");
                let dialog = rect("dialog").expect("dialog rect");
                assert!(
                    screen.contains_rect(title) && screen.contains_rect(tabs) && screen.contains_rect(ribbon),
                    "chrome escaped at {width}x{height} in {language}"
                );
                assert!(
                    rects.iter().all(|(_, rect)| screen.contains_rect(*rect)),
                    "a tracked widget escaped at {width}x{height} in {language}: {rects:?}"
                );
                assert!(title.bottom() <= tabs.top(), "title and tabs overlap at {width}x{height} in {language}");
                assert!(tabs.bottom() <= ribbon.top(), "tabs and ribbon overlap at {width}x{height} in {language}");
                assert!(
                    dialog_area(&ctx, app.ui.ribbon_collapsed).contains_rect(dialog),
                    "dialog escaped content area at {width}x{height} in {language}"
                );
                assert!(!dialog.intersects(ribbon), "dialog overlaps ribbon at {width}x{height} in {language}");
            }
        }
    }
}
