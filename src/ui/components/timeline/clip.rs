

use basedrop::Shared;
use tuix::*;

use crate::backend::timeline::AudioClipSaveState;

pub struct Clip {
    start_time_label: Entity,
    duration_label: Entity,
}

impl Clip {
    pub fn new() -> Self {
        Self {
            start_time_label: Entity::null(),
            duration_label: Entity::null(),
        }
    }
}

impl Widget for Clip {
    type Ret = Entity;
    type Data = AudioClipSaveState;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {

        self.start_time_label = Label::new("")
        .build(state, entity, |builder| 
            builder
                .set_child_space(Stretch(1.0))
        );
        self.duration_label = Label::new("")
        .bind(AudioClipSaveState::duration, |duration| format!("Duration: {}s", duration.as_f32()))
        .build(state, entity, |builder| 
            builder
                .set_child_space(Stretch(1.0))
        );

        entity
    }

    fn on_update(&mut self, state: &mut State, entity: Entity, data: &Self::Data) {
        //println!("Tempo Map {:?}", data.1);
        //let start_x = 30.0 * data.0.get_timeline_start().to_seconds(&data.1).as_f32();
        //let width = 30.0 * data.0.get_duration().as_f32();
        //entity.set_left(state, Pixels(start_x));
        //entity.set_width(state, Pixels(width));

        //self.start_time_label.set_text(state, &format!("Start Time: {}s", data.0.get_timeline_start().to_seconds(&data.1).as_f32()));
        //self.duration_label.set_text(state, &format!("Duration: {}s", data.0.duration().as_f32()));
    }
}

