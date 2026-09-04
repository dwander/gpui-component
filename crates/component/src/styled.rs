pub use crate::component_traits::{Collapsible, Disableable, Selectable};
pub use crate::sizing::{Sizable, Size, StyleSized};
use gpui::{
    App, BoxShadow, Corners, Edges, Hsla, ParentElement, Pixels, StyleRefinement, Styled, Window,
    div, hsla, px,
};
pub use gpui_base::{FocusableExt, RoleOverride, StyledExt, box_shadow, h_flex, v_flex};

use crate::ActiveTheme as _;

const FOCUS_RING_WIDTH: Pixels = px(3.);
const FOCUS_RING_OPACITY: f32 = 0.5;

/// Ink every layer of a surface's shadow carries — the `rgb(0 0 0 / 0.1)`
/// shadcn/ui spends at each elevation.
/// Themes override it through [`crate::Theme::popup_shadow_alpha`] /
/// [`crate::Theme::dialog_shadow_alpha`], which default to this.
pub(crate) const SURFACE_SHADOW_INK: f32 = 0.1;

/// Ink of the hairline ring standing in for a popover's border.
///
/// shadcn/ui draws no border on a popup surface at all: its edge is a 1px
/// `rgb(0 0 0 / 0.1)` ring spent as a shadow layer. Because the ring is
/// translucent the shadow shows *through* it, which is what makes the edge read
/// as part of one grounded surface rather than as an outline with a separate
/// shadow below it. An opaque border cannot reproduce that — a border composites
/// over the element's own background, not over the shadow.
const POPOVER_RING_INK: f32 = 0.1;

/// The colour of a popup surface's hairline ring in this theme.
///
/// shadcn spends black on it in light mode and white in dark
/// (`oklch(1 0 0 / 10%)`), so it follows the foreground rather than the border
/// token: a fixed black ring would all but vanish on a dark surface.
///
pub(crate) fn popover_ring(cx: &App) -> Hsla {
    cx.theme().foreground.alpha(POPOVER_RING_INK)
}

// ── 프로스티드 표면 (반투명 + 뒤 블러 + 유리 림) ──────────────────────────────
//
// 메뉴·드롭다운·컨텍스트 메뉴·확인 대화상자가 **같은 재질**로 보이게 하는 단일 출처.
// 값이 갈라지면 창 하나에 유리가 두 종류 뜬다 — 새 표면은 여기서 가져다 쓴다.

/// 표면 배경 불투명도 — 뒤를 블러가 눌러주므로 살짝 비치게 둔다.
pub const FROSTED_BG_ALPHA: f32 = 0.82;
/// 뒤 배경을 흐리는 반경(px). 크게 잡으면 뒤가 균일한 색으로 뭉개져 유리가 아니라 글로우로
/// 보인다 — 뒤 형체를 알아볼 수 있을 만큼만 흐린다.
pub const FROSTED_BLUR: f32 = 5.;
/// 유리 테두리(림) 두께(px). 1px 짜리 중성 보더는 표면과 그림자 사이에 아무 여백이 없어
/// **그림자가 테두리에서 곧바로 시작되는 것처럼** 보인다(가장자리가 지저분해진다). 조금 두꺼운
/// 밝은 띠를 두르면 그 띠가 표면과 그림자를 갈라, 두께가 있는 유리 모서리로 읽힌다.
pub const FROSTED_RIM: f32 = 2.5;
/// 림의 불투명도 — 흰 띠라 표면이 어두운 다크에서는 옅어도 또렷하고, 표면이 이미 밝은
/// 라이트에서는 거의 흰색이어야 띠로 읽힌다.
const FROSTED_RIM_ALPHA_DARK: f32 = 0.16;
const FROSTED_RIM_ALPHA_LIGHT: f32 = 0.8;
/// 표면 모서리 반경(px).
pub const FROSTED_RADIUS: f32 = 8.;

/// 유리 림 색 — 표면 가장자리에 두르는 밝은 띠. 어느 테마에서든 표면보다 밝아야 하므로
/// 흰색에 테마별 불투명도를 준다.
pub fn frosted_rim(cx: &App) -> Hsla {
    let alpha = if cx.theme().is_dark() {
        FROSTED_RIM_ALPHA_DARK
    } else {
        FROSTED_RIM_ALPHA_LIGHT
    };
    hsla(0., 0., 1., alpha)
}

/// 검정 그림자 레이어 하나 — y 오프셋·blur·spread·alpha 로 생성.
#[inline(always)]
fn shadow_layer(y: f32, blur: f32, spread: f32, a: f32) -> BoxShadow {
    box_shadow(0., y, blur, spread, hsla(0., 0., 0., a))
}

/// 팝업/드롭다운/메뉴 그림자 (gpui `shadow_md` 프로파일, 가변 alpha).
pub fn popup_shadow_vec(alpha: f32) -> Vec<BoxShadow> {
    vec![
        shadow_layer(4., 6., -1., alpha),
        shadow_layer(2., 4., -2., alpha),
    ]
}

/// 다이얼로그/모달 그림자 (gpui `shadow_lg` 프로파일, 가변 alpha).
pub fn dialog_shadow_vec(alpha: f32) -> Vec<BoxShadow> {
    vec![
        shadow_layer(10., 15., -3., alpha),
        shadow_layer(4., 6., -4., alpha),
    ]
}

/// shadcn/ui's popup surface shadow — a hairline `ring` plus `shadow-md` — at
/// `strength` of its full ink.
///
/// Callers animating a surface in pass a rising `strength`; a resting surface
/// passes `1.0`.
///
/// The two blurred layers use Tailwind's radii **halved**, which is the
/// conversion CSS requires and not a taste adjustment. CSS defines a box
/// shadow's blur radius as twice the gaussian's standard deviation, while GPUI's
/// shader takes the field as the deviation itself (`gaussian(y, sigma)`).
/// Copying Tailwind's `6px` and `4px` across therefore spreads the shadow over
/// twice the distance, which is why [`Styled::shadow_md`] reads as a wide grey
/// haze next to a browser's compact one.
///
/// Measured against shadcn's own render, this lands within a luminance step of
/// it the whole way down the falloff.
///
/// The ring and the ink are taken as values rather than read from the theme here
/// so that an animation can hold them across frames, where no `App` is in hand.
/// `ink` is [`crate::Theme::popup_shadow_alpha`]: a dark theme raises it, because
/// shadcn's `0.1` all but vanishes against a dark surface.
pub(crate) fn popover_shadow(ring: Hsla, ink: f32, strength: f32) -> Vec<BoxShadow> {
    let strength = strength.clamp(0., 1.);
    let ink = hsla(0., 0., 0., ink * strength);
    vec![
        // The ring, sitting in the 1px band outside the surface. No blur, so it
        // takes the shader's crisp path rather than the gaussian one.
        BoxShadow::new(px(0.), px(0.), ring.alpha(ring.a * strength))
            .blur_radius(px(0.))
            .spread_radius(px(1.)),
        BoxShadow::new(px(0.), px(4.), ink)
            .blur_radius(px(3.))
            .spread_radius(px(-1.)),
        BoxShadow::new(px(0.), px(2.), ink)
            .blur_radius(px(2.))
            .spread_radius(px(-2.)),
    ]
}

/// shadcn/ui's `shadow-lg`, the elevation it lifts a toast to, at `strength` of
/// its full ink.
///
/// A toast sits higher than a popover and is built differently: shadcn gives it
/// a real 1px border rather than the translucent ring it puts on a popup, so
/// there is no ring layer here. Its corner radius is left to the caller.
///
/// The radii are Tailwind's halved, for the reason [`popover_shadow`] explains.
///
/// `ink` is [`crate::Theme::dialog_shadow_alpha`], passed in rather than read
/// from the theme so an animation can hold it across frames.
pub(crate) fn toast_shadow(ink: f32, strength: f32) -> Vec<BoxShadow> {
    let ink = hsla(0., 0., 0., ink * strength.clamp(0., 1.));
    vec![
        BoxShadow::new(px(0.), px(10.), ink)
            .blur_radius(px(7.5))
            .spread_radius(px(-3.)),
        BoxShadow::new(px(0.), px(4.), ink)
            .blur_radius(px(3.))
            .spread_radius(px(-4.)),
    ]
}

/// shadcn/ui's `shadow-sm`, the elevation it spends on a control raised out of
/// the container it sits in — the active pill of a segmented tab bar — at full
/// ink.
///
/// Unlike a popover or a toast this surface is not floating over the page: it
/// sits *inside* a trough only a few pixels wider than itself, and that trough
/// clips. Both are reasons to keep the falloff tight — there is no room for a
/// wide one, and a wide one would read as grime against the trough wall rather
/// than as lift.
///
/// The radii are Tailwind's halved, for the reason [`popover_shadow`] explains:
/// CSS defines a box shadow's blur radius as twice the gaussian's standard
/// deviation, while GPUI's shader takes the field as the deviation itself
/// (`gaussian(y, sigma)`). Copying Tailwind's `3px` and `2px` across therefore
/// spreads the shadow over twice the distance, which is why
/// [`Styled::shadow_sm`] leaves a haze around a 24px pill where shadcn draws a
/// compact line.
pub(crate) fn raised_shadow() -> Vec<BoxShadow> {
    let ink = hsla(0., 0., 0., SURFACE_SHADOW_INK);
    vec![
        BoxShadow::new(px(0.), px(1.), ink).blur_radius(px(1.5)),
        BoxShadow::new(px(0.), px(1.), ink)
            .blur_radius(px(1.))
            .spread_radius(px(-1.)),
    ]
}

/// Finished styles that read the theme.
///
/// Separate from [`StyledExt`], which holds neutral helpers that make no
/// visual decisions. Everything here does: it reaches into the theme and
/// produces a specific look, which is why it belongs above the base layer.
pub trait ThemeStyled: Styled + Sized {
    /// Give this element the focus appearance the framework's own controls
    /// use: its border tinted with the focus colour, and the ring outside it.
    ///
    /// The ring is dropped when [`crate::Theme::focus_ring`] is off, leaving
    /// the tinted border — an application whose layout clips its containers can
    /// turn it off rather than finding room for the ring in each of them.
    ///
    /// Calling this turns the ring on; gate it with `when` for the conditions
    /// that decide whether the control shows one at all — its focus state,
    /// [`FocusableExt::focus_ring`], appearance, and so on.
    ///
    /// The ring sits outside the element's border, so an ancestor that clips
    /// its content will cut it off — leave it a few pixels of room, or don't
    /// clip.
    fn focus_ring_style(self, window: &Window, cx: &App) -> Self
    where
        Self: ParentElement,
    {
        if !cx.theme().focus_ring {
            return self.border_color(cx.theme().ring);
        }
        self.focus_ring_style_always(window, cx)
    }

    /// The focus ring, drawn whether or not [`crate::Theme::focus_ring`] is on.
    ///
    /// For controls whose focus would otherwise become invisible: the tinted
    /// border that [`Self::focus_ring_style`] falls back to needs a border to
    /// tint, and a filled control (a Primary or Danger button, say) draws none.
    /// Those keep the ring, so turning the theme switch off quiets the controls
    /// that do have a border — inputs — without leaving the rest unreadable.
    fn focus_ring_style_always(self, window: &Window, cx: &App) -> Self
    where
        Self: ParentElement;

    /// Give this element the surface, edge, shadow and radius of a popover.
    ///
    /// This is the one surface every popup shares — Popover, PopupMenu, Select,
    /// Combobox, DatePicker and the editor's hover popovers — so they cannot
    /// drift apart. See [`popover_shadow`] for what the shadow is modelled on.
    fn popover_style(self, cx: &App) -> Self;

    /// 프로스티드 표면 — 반투명 배경 + 뒤 블러 + 밝은 유리 림 + 표면 반경.
    ///
    /// `base` 는 표면색 토큰(메뉴는 `popover`, 대화상자는 `background`). 그림자는 표면마다
    /// 높이가 달라 호출부가 얹는다 — 드롭 그림자는 요소 박스를 파내고 그려지므로(gpui)
    /// 반투명 배경이 자기 그림자에 물들지 않는다.
    fn frosted_surface_style(self, base: Hsla, cx: &App) -> Self {
        self.bg(base.opacity(FROSTED_BG_ALPHA))
            .backdrop_blur(px(FROSTED_BLUR))
            .border(px(FROSTED_RIM))
            .border_color(frosted_rim(cx))
            .rounded(px(FROSTED_RADIUS))
    }

    /// Round this element as far as its size allows — a circle if it is square,
    /// a pill if it is not — unless the theme squares its corners.
    ///
    /// Use this instead of [`gpui::Styled::rounded_full`] on anything the theme
    /// owns. A hardcoded `rounded_full` survives [`crate::Theme::radius`] being
    /// set to zero, which leaves avatars, badge dots and slider thumbs round in
    /// a UI that is square everywhere else. See [`crate::Theme::radius_full`].
    fn rounded_full_style(self, cx: &App) -> Self {
        self.rounded(cx.theme().radius_full())
    }
}

impl<T: Styled + Sized> ThemeStyled for T {
    /// Draw the focus ring the framework's own controls use.
    ///
    /// Calling this turns the ring on; gate it with `when` for the conditions
    /// that decide whether the control shows one at all — its focus state,
    /// [`crate::FocusableExt::focus_ring`], appearance, and so on.
    ///
    /// The ring sits outside the element's border, so an ancestor that clips
    /// its content will cut it off — leave it a few pixels of room, or don't
    /// clip.
    fn focus_ring_style_always(mut self, window: &Window, cx: &App) -> Self
    where
        Self: ParentElement,
    {
        let rem_size = window.rem_size();
        let style = self.style();
        let border_widths = Edges::<Pixels> {
            top: style
                .border_widths
                .top
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            bottom: style
                .border_widths
                .bottom
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            left: style
                .border_widths
                .left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            right: style
                .border_widths
                .right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
        };
        let radius = Corners::<Pixels> {
            top_left: style
                .corner_radii
                .top_left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            top_right: style
                .corner_radii
                .top_right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            bottom_left: style
                .corner_radii
                .bottom_left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
            bottom_right: style
                .corner_radii
                .bottom_right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or_default(),
        }
        .map(|value| *value + FOCUS_RING_WIDTH);
        let mut ring_style = StyleRefinement::default();
        ring_style.corner_radii.top_left = Some(radius.top_left.into());
        ring_style.corner_radii.top_right = Some(radius.top_right.into());
        ring_style.corner_radii.bottom_left = Some(radius.bottom_left.into());
        ring_style.corner_radii.bottom_right = Some(radius.bottom_right.into());
        let inset = FOCUS_RING_WIDTH;

        self.border_color(cx.theme().ring).child(
            div()
                .flex_none()
                .absolute()
                .top(-(inset + border_widths.top))
                .left(-(inset + border_widths.left))
                .right(-(inset + border_widths.right))
                .bottom(-(inset + border_widths.bottom))
                .border(FOCUS_RING_WIDTH)
                .border_color(cx.theme().ring.alpha(FOCUS_RING_OPACITY))
                .refine_style(&ring_style),
        )
    }

    fn popover_style(self, cx: &App) -> Self {
        let theme = cx.theme();
        // No border: the edge is the ring inside `popover_shadow`, which is how
        // shadcn draws it and the only way the shadow can show through it.
        self.bg(theme.popover)
            .text_color(theme.popover_foreground)
            .shadow(popover_shadow(
                popover_ring(cx),
                theme.popup_shadow_alpha,
                1.,
            ))
            .rounded(theme.radius)
    }
}
