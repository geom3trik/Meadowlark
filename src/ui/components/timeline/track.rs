use std::collections::HashMap;
use tuix::*;


use crate::{backend::timeline::TimelineTrackSaveState, state::{BoundGuiState, ProjectSaveState}};

use super::Clip;

// Track (TODO)
pub struct Track {
    name: String,

    clips: HashMap<String, Entity>,

    // temp - probably
    index: usize,
}

impl Track {
    pub fn new(name: String, index: usize) -> Self {
        Self {
            name: name.clone(),
            clips: HashMap::new(),

            index,
        }
    }
}

impl Widget for Track {
    type Ret = Entity;
    type Data = TimelineTrackSaveState;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_background_color(state, Color::rgb(150, 100, 190))
            .set_height(state, Pixels(80.0))
            //.set_width(state, Pixels(1000.0))
            //.set_text(state, &self.name)
    }

    fn on_update(&mut self, state: &mut State, entity: Entity, data: &Self::Data) {
        for (index, clip) in data.audio_clips.iter().enumerate() {
            //println!("Clip Time: {:?}", clip.timeline_start());
            //println!("Clip Duration: {:?}", clip.duration());
            
            if !self.clips.contains_key(&clip.name) {
                self.clips.insert(clip.name.clone(), Clip::new()
                    .bind(BoundGuiState::save_state
                        .then(ProjectSaveState::timeline_tracks)
                        .index(self.index)
                        .then(TimelineTrackSaveState::audio_clips)
                        .index(index), |val| val.clone())
                    .build(state, entity, |builder|
                    builder
                        .set_background_color(Color::rgb(100, 80, 150))
                        .set_width(Pixels(50.0))
                ));
            }
            

        }
    }
}

// Track Controls (TODO)

pub struct TrackControls {
    name: String,
}

impl TrackControls {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

impl Widget for TrackControls {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        Element::new().build(state, entity, |builder| {
            builder
                .set_width(Pixels(10.0))
                .set_background_color(Color::rgb(254, 64, 64))
        });

        let col = Column::new().build(state, entity, |builder| {
            builder.set_space(Pixels(5.0)).set_row_between(Pixels(5.0))
        });

        Textbox::new(&self.name).build(state, col, |builder| {
            builder
                .set_background_color(Color::rgb(57, 52, 54))
                .set_child_space(Stretch(1.0))
        });

        Element::new().build(state, col, |builder| {
            builder
                .set_background_color(Color::rgb(0, 240, 77))
                .set_height(Pixels(10.0))
        });

        Element::new().build(state, col, |builder| {
            builder.set_background_color(Color::rgb(57, 52, 54))
        });

        let buttons = Element::new().build(state, entity, |builder| {
            builder
                .set_layout_type(LayoutType::Grid)
                .set_grid_rows(vec![Pixels(30.0), Pixels(30.0)])
                .set_grid_cols(vec![Pixels(30.0), Pixels(30.0), Pixels(30.0)])
                .set_row_between(Stretch(1.0))
                .set_col_between(Stretch(1.0))
                .set_space(Pixels(5.0))
        });

        Element::new().build(state, buttons, |builder| {
            builder
                .set_background_color(Color::rgb(57, 52, 54))
                .set_border_radius(Pixels(3.0))
                .set_child_space(Stretch(1.0))
                .set_row_index(0)
                .set_col_index(0)
                .set_text("R")
        });

        Element::new().build(state, buttons, |builder| {
            builder
                .set_background_color(Color::rgb(57, 52, 54))
                .set_border_radius(Pixels(3.0))
                .set_child_space(Stretch(1.0))
                .set_row_index(0)
                .set_col_index(1)
                .set_text("S")
        });

        Element::new().build(state, buttons, |builder| {
            builder
                .set_background_color(Color::rgb(57, 52, 54))
                .set_border_radius(Pixels(3.0))
                .set_child_space(Stretch(1.0))
                .set_row_index(0)
                .set_col_index(2)
                .set_text("M")
        });

        // Element::new().build(state, buttons, |builder|
        //     builder
        //         .set_background_color(Color::rgb(57, 52, 54))
        //         .set_border_radius(Pixels(3.0))
        //         .set_child_space(Stretch(1.0))
        //         .set_row_index(1)
        //         .set_col_index(0)
        //         .set_text("V")
        // );

        let map = DecibelMap::new(
            -12.0,
            12.0,
            ValueScaling::Linear,
            DisplayDecimals::One,
            true,
        );
        let normalized_default = map.db_to_normalized(0.0);

        Knob::new(map, normalized_default).build(state, buttons, |builder| {
            builder.set_row_index(1).set_col_index(0)
        });

        Element::new().build(state, buttons, |builder| {
            builder
                .set_background_color(Color::rgb(57, 52, 54))
                .set_border_radius(Pixels(3.0))
                .set_child_space(Stretch(1.0))
                .set_row_index(1)
                .set_col_index(1)
                .set_text("A")
        });

        Element::new().build(state, buttons, |builder| {
            builder
                .set_background_color(Color::rgb(57, 52, 54))
                .set_border_radius(Pixels(3.0))
                .set_child_space(Stretch(1.0))
                .set_row_index(1)
                .set_col_index(2)
                .set_text("B")
        });

        // Seems to be an issue with slider on the tuix end
        //Slider::new().build(state, col, |builder| builder);

        entity
            .set_layout_type(state, LayoutType::Row)
            .set_background_color(state, Color::rgb(136, 127, 130))
            .set_height(state, Pixels(80.0))
            .set_width(state, Stretch(1.0))
    }
}
