use crowntest::{TestApp, role};
use semantics::Role;
use view::{Listeners, Styled, bind, component};
use widgets::{button, text, zstack};

/// A 1000x500 stage drawn at half size from the top left, holding a card drawn at 1.5x from
/// its own top left, holding a button: the layout of a lock screen fitted to a scaled output.
fn scaled_stage() -> TestApp {
    TestApp::with_size(
        component(|cx| {
            let clicks = cx.signal(0_u32);
            let hovered = cx.signal(false);
            let card = zstack((button("Play")
                .w(40.0)
                .h(20.0)
                .absolute()
                .left(10.0)
                .top(10.0)
                .on_click(move |cx, _| clicks.update(cx, |count| *count += 1))
                .on_pointer_enter(move |cx, _| hovered.set(cx, true)),))
            .w(100.0)
            .h(50.0)
            .absolute()
            .left(600.0)
            .top(300.0)
            .origin(0.0, 0.0)
            .scale(1.5);
            zstack((
                zstack((card,))
                    .w(1000.0)
                    .h(500.0)
                    .absolute()
                    .left(0.0)
                    .top(0.0)
                    .origin(0.0, 0.0)
                    .scale(0.5),
                text(bind(move |runtime| {
                    let hover = if hovered.get(runtime) { " hovered" } else { "" };
                    format!("clicks {}{hover}", clicks.get(runtime))
                }))
                .absolute()
                .left(0.0)
                .top(480.0),
            ))
        }),
        1000.0,
        500.0,
    )
}

/// The stage maps x to x * 0.5, the card maps its local x to 600 + x * 1.5: the button's
/// layout rect (610, 310, 40, 20) is drawn at (307.5, 157.5, 30, 15).
const DRAWN_CENTER: (f32, f32) = (322.5, 165.0);
const LAYOUT_CENTER: (f32, f32) = (630.0, 320.0);

#[test]
fn a_click_lands_where_a_scaled_button_is_drawn() {
    let mut app = scaled_stage();
    let (x, y) = DRAWN_CENTER;
    app.pointer_move(x, y).pointer_down(x, y).pointer_up(x, y);
    assert_eq!(app.text_of(role(Role::Text)), "clicks 1 hovered");
}

#[test]
fn a_click_where_layout_put_the_button_misses_it() {
    let mut app = scaled_stage();
    let (x, y) = LAYOUT_CENTER;
    app.pointer_move(x, y).pointer_down(x, y).pointer_up(x, y);
    assert_eq!(app.text_of(role(Role::Text)), "clicks 0");
}

#[test]
fn semantic_bounds_report_the_drawn_rect() {
    let mut app = scaled_stage();
    let bounds = app.find(role(Role::Button).name("Play")).bounds;
    for (actual, expected) in [
        (bounds.origin.x, 307.5),
        (bounds.origin.y, 157.5),
        (bounds.size.width, 30.0),
        (bounds.size.height, 15.0),
    ] {
        assert!((actual - expected).abs() < 0.01, "{bounds:?}");
    }
}

#[test]
fn hovering_a_scaled_button_by_query_reaches_it() {
    let mut app = scaled_stage();
    app.hover(role(Role::Button).name("Play"));
    assert_eq!(app.text_of(role(Role::Text)), "clicks 0 hovered");
}
