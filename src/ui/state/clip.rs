use super::core_types::{WMusicalTime, WSeconds, WSuperFrames};
use basedrop::Shared;
use pcm_loader::PcmRAM;
use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Data)]
pub struct ClipState {
    pub name: String,
    pub timeline_start: ClipStart,

    pub channel: usize,

    pub type_: ClipType,
}

#[derive(Debug, Lens, Clone, Data)]
pub enum ClipType {
    Audio(AudioClipState),
    PianoRoll(PianoRollClipState),
    Automation(AutomationClipState),
}

#[derive(Debug, Lens, Clone, Data)]
pub struct AudioClipState {
    pub length: WSeconds,

    pub fade_in_secs: WSeconds,
    pub fade_out_secs: WSeconds,

    /// The amount of time between the start of the raw waveform data
    /// and the start of the clip.
    ///
    /// TODO
    pub clip_start_offset: WSuperFrames,

    pub pcm: SharedPcmData,
    // TODO: pointer to waveform data
}

pub struct SharedPcmData {
    pub pcm: Shared<PcmRAM>,
}

impl vizia::prelude::Data for SharedPcmData {
    fn same(&self, other: &Self) -> bool {
        &*self.pcm as *const PcmRAM == &*other.pcm as *const PcmRAM
    }
}

impl Clone for SharedPcmData {
    fn clone(&self) -> Self {
        Self { pcm: Shared::clone(&self.pcm) }
    }
}

impl std::fmt::Debug for SharedPcmData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("SharedPcmData");
        f.field("channels", &self.pcm.channels());
        f.field("len_frames", &self.pcm.len_frames());
        f.finish()
    }
}

#[derive(Debug, Lens, Clone, Data)]
pub struct PianoRollClipState {
    // TODO
}

#[derive(Debug, Lens, Clone, Data)]
pub struct AutomationClipState {
    // TODO
}

#[derive(Debug, Lens, Clone, Data)]
pub enum ClipStart {
    OnLane(OnLane),
    /// This means that the clip is not currently on the timeline,
    /// and instead just lives in the clips panel.
    NotInTimeline,
}

#[derive(Debug, Lens, Clone, Data)]
pub struct OnLane {
    pub lane_index: u32,
    pub timeline_start: WMusicalTime,
}
