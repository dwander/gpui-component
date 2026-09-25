use crate::{Icon, IconName, Sizable, Size};
use gpui::{
    Animation, AnimationExt as _, App, Global, Hsla, IntoElement, ParentElement, RenderOnce,
    Styled as _, Transformation, Window, div, ease_in_out, percentage, prelude::FluentBuilder as _,
};
use instant::Duration;

/// 앱 전체의 기본 스피너 아이콘 — [`set_default_icon`] 으로 정한다.
///
/// 아이콘을 따로 주지 않은 **모든** [`Spinner`] 가 이것을 쓴다. 컴포넌트 안에서 만드는 스피너
/// (버튼·입력칸 로딩 표시 등)도 `Spinner::new()` 라 함께 따라오므로, 앱은 한 곳에서 한 번만
/// 정하면 된다. 정하지 않으면 [`IconName::Loader`].
struct DefaultSpinnerIcon(Icon);

impl Global for DefaultSpinnerIcon {}

/// 기본 스피너 아이콘을 바꾼다 — 앱 시작 시 한 번 부른다.
pub fn set_default_icon(icon: impl Into<Icon>, cx: &mut App) {
    cx.set_global(DefaultSpinnerIcon(icon.into()));
}

/// A cycling loading spinner.
#[derive(IntoElement)]
pub struct Spinner {
    size: Size,
    /// 따로 준 아이콘 — 없으면 그릴 때 [`set_default_icon`] 의 값(없으면 [`IconName::Loader`]).
    icon: Option<Icon>,
    speed: Duration,
    easing: Box<dyn Fn(f32) -> f32>,
    color: Option<Hsla>,
}

impl Spinner {
    /// Create a new loading spinner.
    pub fn new() -> Self {
        Self {
            size: Size::Medium,
            speed: Duration::from_secs_f64(0.8),
            easing: Box::new(ease_in_out),
            icon: None,
            color: None,
        }
    }

    /// Set specified icon for the spinner.
    ///
    /// Default is the app-wide icon from [`set_default_icon`], or [`IconName::Loader`].
    ///
    /// Please ensure the icon used is suitable for a loading spinner.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the icon color.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the easing function.
    pub fn ease(mut self, easing: impl Fn(f32) -> f32 + 'static) -> Self {
        self.easing = Box::new(easing);
        self
    }
}

impl Sizable for Spinner {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let icon = self.icon.unwrap_or_else(|| {
            cx.try_global::<DefaultSpinnerIcon>()
                .map(|d| d.0.clone())
                .unwrap_or_else(|| Icon::new(IconName::Loader))
        });
        div()
            .child(
                icon
                    .with_size(self.size)
                    .when_some(self.color, |this, color| this.text_color(color))
                    .with_animation(
                        "circle",
                        Animation::new(self.speed).repeat().with_easing(self.easing),
                        |this, delta| this.transform(Transformation::rotate(percentage(delta))),
                    ),
            )
            .into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Render, TestAppContext, px, size};

    struct SpinnerHost;

    impl Render for SpinnerHost {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            Spinner::new()
        }
    }

    #[gpui::test]
    fn reduced_motion_spinner_is_static_and_requests_no_frame(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let window = cx.open_window(size(px(100.), px(100.)), |_, _| SpinnerHost);
        cx.run_until_parked();

        assert_eq!(
            window
                .update(cx, |_, window, cx| window.simulate_next_frame(cx))
                .unwrap(),
            0
        );
    }
}
