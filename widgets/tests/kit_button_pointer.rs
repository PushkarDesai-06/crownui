//! Kit buttons clicked through the real pointer path (press and release), not the
//! accessibility action `TestApp::click` performs.

use crowntest::{TestApp, text as text_query};
use view::{Listeners, bind, component};
use widgets::{
    kit::{button, toggle},
    text, vstack,
};

#[test]
fn a_pointer_press_and_release_on_a_button_clicks_it() {
    let mut app = TestApp::with_size(
        component(|cx| {
            let presses = cx.signal(0);
            vstack((
                button("Save").on_click(move |cx, _| presses.update(cx, |count| *count += 1)),
                text(bind(move |runtime| {
                    format!("Pressed {}", presses.get(runtime))
                })),
            ))
        }),
        300.0,
        200.0,
    );
    app.settle();
    app.pointer_move(20.0, 12.0)
        .pointer_down(20.0, 12.0)
        .pointer_up(20.0, 12.0)
        .settle();
    assert_eq!(
        app.text_of(text_query("Pressed")),
        "Pressed 1",
        "{}",
        app.layout_snapshot()
    );
}

#[test]
fn a_pointer_press_and_release_on_a_toggle_flips_it() {
    let mut app = TestApp::with_size(
        component(|cx| {
            let on = cx.signal(false);
            vstack((
                toggle(on),
                text(bind(move |runtime| format!("On {}", on.get(runtime)))),
            ))
        }),
        300.0,
        200.0,
    );
    app.settle();
    app.pointer_move(12.0, 10.0)
        .pointer_down(12.0, 10.0)
        .pointer_up(12.0, 10.0)
        .settle();
    assert_eq!(
        app.text_of(text_query("On")),
        "On true",
        "{}",
        app.layout_snapshot()
    );
}

#[test]
fn a_button_in_a_themed_scrolling_settings_panel_clicks_by_pointer() {
    use view::Styled;
    use widgets::{
        hstack,
        kit::{Theme, provide_theme, section, setting_row},
        scroll_view, spacer, zstack,
    };
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            let presses = cx.signal(0);
            let header = hstack((
                text("Lock screen"),
                spacer(),
                button("Save").on_click(move |cx, _| presses.update(cx, |count| *count += 1)),
            ))
            .gap(8.0)
            .items_center();
            let rows = section(
                "Preview",
                (setting_row(
                    "Play",
                    button("Play").on_click(move |cx, _| presses.update(cx, |count| *count += 10)),
                ),),
            );
            let panel = scroll_view(
                vstack((
                    header,
                    rows,
                    text(bind(move |runtime| {
                        format!("Pressed {}", presses.get(runtime))
                    })),
                ))
                .p(16.0)
                .gap(16.0),
            )
            .absolute()
            .top(0.0)
            .right(0.0)
            .bottom(0.0)
            .w(300.0);
            zstack((panel,))
                .w_full()
                .h_full()
                .on_pointer_down(|_cx, _event| {})
                .on_pointer_move(|_cx, _event| {})
                .on_pointer_up(|_cx, _event| {})
        }),
        600.0,
        400.0,
    );
    app.settle();
    let layout = app.layout_snapshot();
    // The Save button: right end of the header row, 16 px in from the panel's top-right.
    app.pointer_move(560.0, 32.0)
        .pointer_down(560.0, 32.0)
        .pointer_up(560.0, 32.0)
        .settle();
    assert_eq!(app.text_of(text_query("Pressed")), "Pressed 1", "{layout}");
}

#[test]
fn a_button_still_clicks_when_frames_pass_between_press_and_release() {
    use widgets::kit::{Theme, provide_theme};
    let mut app = TestApp::with_size(
        component(|cx| {
            provide_theme(cx, Theme::dark());
            let presses = cx.signal(0);
            vstack((
                button("Save").on_click(move |cx, _| presses.update(cx, |count| *count += 1)),
                text(bind(move |runtime| {
                    format!("Pressed {}", presses.get(runtime))
                })),
            ))
        }),
        300.0,
        200.0,
    );
    app.settle();
    app.pointer_move(20.0, 12.0).advance_ms(120);
    app.pointer_down(20.0, 12.0).advance_ms(40);
    app.pointer_up(20.0, 12.0).settle();
    assert_eq!(app.text_of(text_query("Pressed")), "Pressed 1");
}
