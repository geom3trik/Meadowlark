use tuix::*;

mod track;
use track::*;

use crate::backend::timeline::TimelineTrackSaveState;
use crate::backend::{BackendHandle, ProjectSaveState};

use crate::ui::app_data::AppData;

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollEvent {
    ScrollH(Scroll),
    ScrollV(Scroll),
}

#[derive(Debug, Default, Clone, Copy, Lens)]
pub struct ScrollState {
    horizontal: Scroll,
    vertical: Scroll,
}

impl Model for ScrollState {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(scroll_event) = event.message.downcast() {
            match scroll_event {
                ScrollEvent::ScrollV(scroll) => {
                    self.vertical = *scroll;
                    entity.emit(state, BindEvent::Update);
                    event.consume();
                }

                ScrollEvent::ScrollH(scroll) => {
                    self.horizontal = *scroll;
                    entity.emit(state, BindEvent::Update);
                    event.consume();
                }
            }
        }
    }
}

/// A general purpose timeline widget
pub struct Timeline {}

impl Timeline {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Timeline {
    type Ret = Entity;
    type Data = Vec<TimelineTrackSaveState>;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let scroll_data = ScrollState::default().build(state, entity);

        scroll_data
            .set_background_color(state, Color::rgb(64, 59, 59))
            .set_layout_type(state, LayoutType::Grid)
            .set_grid_cols(state, vec![Pixels(250.0), Stretch(1.0), Pixels(10.0)])
            .set_grid_rows(state, vec![Stretch(1.0), Pixels(10.0)]);

        Element::new().build(state, scroll_data, |builder| {
            builder
                .set_background_color(Color::rgb(43, 39, 40))
                .set_row_index(1)
                .set_col_index(2)
                .set_row_span(1)
                .set_col_span(1)
        });

        // Vertical scroll container for control
        let scroll = ScrollContainer::new()
            .on_scroll(|data, state, scroll_container| {
                scroll_container.emit(state, ScrollEvent::ScrollV(data.scroll));
            })
            .bind(ScrollState::vertical, |scroll| *scroll)
            .build(state, scroll_data, |builder| {
                builder
                    .set_col_index(0)
                    .set_row_index(0)
                    .set_col_span(1)
                    .set_row_span(1)
            });

        let controls = ListView::new(|item| TrackControls::new())
            .bind(
                AppData::backend_handle
                    .then(BackendHandle::save_state)
                    .then(ProjectSaveState::timeline_tracks),
                |tracks| tracks.clone(),
            )
            .build(state, scroll, |builder| {
                builder
                    .set_height(Auto)
                    .set_width(Stretch(1.0))
                    .set_row_between(Pixels(2.0))
            });

        // //
        // let controls = Element::new().build(state, scroll, |builder| {
        //     builder
        //         .set_background_color(Color::rgb(64, 59, 59))
        //         //.set_text("Controls")
        //         .set_height(Auto)
        //         .set_width(Stretch(1.0))
        //         .set_row_between(Pixels(2.0))
        // });

        // for _ in 0..10 {
        //     Element::new().build(state, controls, |builder| {
        //         builder
        //             .set_height(Pixels(50.0))
        //             .set_background_color(Color::rgb(114, 106, 109))
        //             .set_text("Track controls...")
        //     });
        // }

        // Vertical scroll container for tracks
        let scroll = ScrollContainer::new()
            .on_scroll(|data, state, scroll_container| {
                scroll_container.emit(state, ScrollEvent::ScrollV(data.scroll));
            })
            .bind(ScrollState::vertical, |scroll| *scroll)
            .build(state, scroll_data, |builder| {
                builder
                    //.set_background_color(Color::yellow())
                    .set_col_index(1)
                    .set_row_index(0)
                    .set_col_span(1)
                    .set_row_span(1)
            });

        let tracks = ScrollContainerH::new()
            .on_scroll(|data, state, scroll_container| {
                scroll_container.emit(state, ScrollEvent::ScrollH(data.scroll));
            })
            .bind(ScrollState::horizontal, |scroll| *scroll)
            .build(
                state,
                scroll,
                |builder| builder.set_height(Auto).set_width(Stretch(1.0)), //.set_background_color(Color::rgb(20,200,20))
                                                                            //.set_text("Tracks")
            );

        tracks
            .set_row_between(state, Pixels(2.0))
            .set_height(state, Auto);

        ListView::new(|item: &TimelineTrackSaveState| Track::new(item.name().clone()))
            .bind(
                AppData::backend_handle
                    .then(BackendHandle::save_state)
                    .then(ProjectSaveState::timeline_tracks),
                |tracks| tracks.clone(),
            )
            .build(state, tracks, |builder| {
                builder
                    .set_height(Auto)
                    .set_width(Auto)
                    .set_row_between(Pixels(2.0))
            });

        // println!("Tracks: {}", tracks);

        // for _ in 0..10 {
        //     Element::new().build(state, tracks, |builder|
        //         builder
        //             .set_height(Pixels(50.0))
        //             .set_width(Pixels(1000.0))
        //             .set_background_color(Color::rgba(114, 106, 109, 100))
        //             .set_text("Clips and stuff goes here... Clips and stuff goes here... Clips and stuff goes here...")
        //     );
        // }

        Scrollbar::new(ScrollDirection::Horizontal)
            .on_scroll(|data, state, scrollbar| {
                scrollbar.emit(state, ScrollEvent::ScrollH(data.scroll));
            })
            .bind(ScrollState::horizontal, |scroll| *scroll)
            .build(state, scroll_data, |builder| {
                builder.set_col_index(1).set_row_index(1)
            });

        Scrollbar::new(ScrollDirection::Vertical)
            .on_scroll(|data, state, scrollbar| {
                scrollbar.emit(state, ScrollEvent::ScrollV(data.scroll));
            })
            .bind(ScrollState::vertical, |scroll| *scroll)
            .build(state, scroll_data, |builder| {
                builder.set_col_index(2).set_row_index(0)
            });

        entity.set_element(state, "timeline")
    }

    fn on_update(&mut self, state: &mut State, entity: Entity, data: &Self::Data) {}
}
