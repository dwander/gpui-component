use gpui::{App, ClickEvent, InteractiveElement, Stateful, Task, Window};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub trait InteractiveElementExt: InteractiveElement {
    /// Set the listener for a double click event.
    fn on_double_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self
    where
        Self: Sized,
    {
        self.interactivity().on_click(move |event, window, cx| {
            if event.click_count() == 2 {
                listener(event, window, cx);
            }
        });
        self
    }

    /// Set click handlers that distinguish single and double clicks using a delay.
    ///
    /// - **Double click**: fires `on_double` immediately and cancels any pending single click.
    /// - **Single click**: fires `on_single` after `delay_ms` (cancelled if a double click arrives).
    /// - A 500ms cooldown after each double click prevents re-entry (e.g. mode-switch bounce).
    ///
    /// Use this when single and double clicks on the same element must trigger *different* actions
    /// and must not interfere with each other.
    fn on_click_with_double_click(
        mut self,
        delay_ms: u64,
        on_single: impl Fn(&mut Window, &mut App) + 'static,
        on_double: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self
    where
        Self: Sized,
    {
        let pending: Rc<Cell<Option<Task<()>>>> = Rc::new(Cell::new(None));
        let last_double: Rc<Cell<Option<Instant>>> = Rc::new(Cell::new(None));
        let on_single = Rc::new(on_single);

        self.interactivity().on_click(move |event, window, cx| {
            if event.click_count() == 2 {
                // Double click: cancel pending single + fire immediately
                pending.set(None); // Task drop = cancel
                last_double.set(Some(Instant::now()));
                on_double(event, window, cx);
            } else {
                // Single click: check cooldown, then schedule delayed execution
                let in_cooldown = last_double
                    .get()
                    .map(|t| t.elapsed() < Duration::from_millis(500))
                    .unwrap_or(false);
                if in_cooldown {
                    return;
                }

                let pending_clone = pending.clone();
                let on_single = on_single.clone();
                let task = window.spawn(cx, async move |async_cx| {
                    smol::Timer::after(Duration::from_millis(delay_ms)).await;
                    let _ = async_cx.update(|window, cx| {
                        on_single(window, cx);
                        pending_clone.set(None);
                    });
                });
                pending.set(Some(task));
            }
        });
        self
    }
}

impl<E: InteractiveElement> InteractiveElementExt for Stateful<E> {}
