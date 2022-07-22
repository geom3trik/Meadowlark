use basedrop::{Owned, Shared};
use dropseed::plugin::HostRequestChannelSender;
use dropseed::plugin::{
    buffer::EventBuffer, ext, HostInfo, PluginActivatedInfo, PluginAudioThread, PluginDescriptor,
    PluginFactory, PluginInstanceID, PluginMainThread, ProcBuffers, ProcInfo, ProcessStatus,
};
use dropseed::transport::TempoMap;
use fnv::FnvHashMap;
use meadowlark_core_types::time::{Frames, MusicalTime, SampleRate, Seconds, SuperFrames};
use pcm_loader::PcmRAM;
use rtrb::{Consumer, Producer, RingBuffer};
use std::hash::Hash;

use crate::ui::AudioClipState;

pub static TIMELINE_TRACK_PLUG_RDN: &str = "app.meadowlark.timeline-track";

static MSG_BUFFER_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelineTrackAudioClipID(u64);

enum HandleToProcMsg {
    UpdateAudioClipList(Owned<FnvHashMap<TimelineTrackAudioClipID, AudioClipProcState>>),
    UpdateAudioClip(AudioClipProcState),
}

#[derive(Clone)]
struct AudioClipProcState {
    id: TimelineTrackAudioClipID,

    timeline_start: MusicalTime,
    length: Seconds,
    fade_in_secs: Seconds,
    fade_out_secs: Seconds,
    clip_start_offset: SuperFrames,

    timeline_start_frame: Frames,
    timeline_end_frame: Frames,

    fade_in_end_frame: Frames,
    fade_out_start_frame: Frames,

    clip_start_offset_frames: Frames,
    clip_start_offset_sub_frame: f64,

    pcm: Shared<PcmRAM>,
}

impl AudioClipProcState {
    fn new(
        id: TimelineTrackAudioClipID,
        timeline_start: MusicalTime,
        timeline_start_frame: Frames,
        length: Seconds,
        fade_in_secs: Seconds,
        fade_out_secs: Seconds,
        clip_start_offset: SuperFrames,
        pcm: Shared<PcmRAM>,
        tempo_map: &TempoMap,
    ) -> Self {
        let timeline_start_secs = tempo_map.musical_to_seconds(timeline_start);
        let timeline_end_frame =
            tempo_map.seconds_to_nearest_frame_round(timeline_start_secs + length);

        let fade_in_end_frame = if fade_in_secs.0 == 0.0 {
            timeline_start_frame
        } else {
            tempo_map.seconds_to_nearest_frame_round(timeline_start_secs + fade_in_secs)
        };
        let fade_out_start_frame = if fade_out_secs.0 == 0.0 {
            timeline_end_frame
        } else {
            tempo_map.seconds_to_nearest_frame_round(timeline_start_secs + length - fade_in_secs)
        };

        let clip_sample_rate = SampleRate(pcm.sample_rate() as f64);
        let (clip_start_offset_frames, clip_start_offset_sub_frame) =
            clip_start_offset.to_seconds().to_sub_frame(clip_sample_rate);

        Self {
            id,

            timeline_start,
            length,
            fade_in_secs,
            fade_out_secs,
            clip_start_offset,

            timeline_start_frame,
            timeline_end_frame,
            fade_in_end_frame,
            fade_out_start_frame,
            clip_start_offset_frames,
            clip_start_offset_sub_frame,
            pcm,
        }
    }
}

pub struct TimelineTrackPlugFactory;

impl PluginFactory for TimelineTrackPlugFactory {
    fn description(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: TIMELINE_TRACK_PLUG_RDN.into(),
            version: "0.1".into(),
            name: "Sample Browser".into(),
            vendor: "Meadowlark".into(),
            description: String::new(),
            url: String::new(),
            manual_url: String::new(),
            support_url: String::new(),
            features: String::new(),
        }
    }

    fn instantiate(
        &mut self,
        host_request_channel: HostRequestChannelSender,
        _host_info: Shared<HostInfo>,
        _plugin_id: PluginInstanceID,
        _coll_handle: &basedrop::Handle,
    ) -> Result<Box<dyn PluginMainThread>, String> {
        Ok(Box::new(TimelineTrackPlugMainThread::new(host_request_channel)))
    }
}

pub struct TimelineTrackPlugHandle {
    proc_tx: Producer<HandleToProcMsg>,

    audio_clips: FnvHashMap<TimelineTrackAudioClipID, AudioClipProcState>,

    next_id: u64,

    coll_handle: basedrop::Handle,
}

impl TimelineTrackPlugHandle {
    fn new(proc_tx: Producer<HandleToProcMsg>, coll_handle: basedrop::Handle) -> Self {
        Self { proc_tx, audio_clips: FnvHashMap::default(), next_id: 0, coll_handle }
    }

    fn send(&mut self, msg: HandleToProcMsg) {
        if let Err(e) = self.proc_tx.push(msg) {
            panic!("Failed to send message to TimelineTrackPluginAudioThread: {}", e);
        }
    }

    pub fn add_new_audio_clip(
        &mut self,
        timeline_start: MusicalTime,
        clip: &AudioClipState,
        tempo_map: &TempoMap,
    ) -> TimelineTrackAudioClipID {
        let timeline_start_frame = tempo_map.musical_to_nearest_frame_round(timeline_start);

        let id = TimelineTrackAudioClipID(self.next_id);
        self.next_id += 1;

        let audio_clip = AudioClipProcState::new(
            id,
            timeline_start.into(),
            timeline_start_frame,
            clip.length.into(),
            clip.fade_in_secs.into(),
            clip.fade_out_secs.into(),
            clip.clip_start_offset.into(),
            Shared::clone(&clip.pcm.pcm),
            tempo_map,
        );

        let _ = self.audio_clips.insert(id, audio_clip.clone());

        // TODO: Use a persistant data structure to make this more efficient?
        let msg = HandleToProcMsg::UpdateAudioClipList(Owned::new(
            &self.coll_handle,
            self.audio_clips.clone(),
        ));
        self.send(msg);

        id
    }

    pub fn remove_audio_clip(&mut self, id: &TimelineTrackAudioClipID) -> Result<(), ()> {
        if self.audio_clips.remove(id).is_some() {
            // TODO: Use a persistant data structure to make this more efficient?
            let msg = HandleToProcMsg::UpdateAudioClipList(Owned::new(
                &self.coll_handle,
                self.audio_clips.clone(),
            ));

            self.send(msg);

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn audio_clip_modified(
        &mut self,
        id: &TimelineTrackAudioClipID,
        timeline_start: MusicalTime,
        clip: &AudioClipState,
        tempo_map: &TempoMap,
    ) -> Result<(), ()> {
        let timeline_start_frame = tempo_map.musical_to_nearest_frame_round(timeline_start);

        if let Some(c) = self.audio_clips.get_mut(id) {
            *c = AudioClipProcState::new(
                *id,
                timeline_start.into(),
                timeline_start_frame,
                c.length,
                c.fade_in_secs,
                c.fade_out_secs,
                c.clip_start_offset,
                Shared::clone(&c.pcm),
                tempo_map,
            );

            let msg = HandleToProcMsg::UpdateAudioClip(c.clone());
            self.send(msg);

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn tempo_map_modified(&mut self, tempo_map: &TempoMap) {
        for (id, c) in self.audio_clips.iter_mut() {
            *c = AudioClipProcState::new(
                *id,
                c.timeline_start,
                tempo_map.musical_to_nearest_frame_round(c.timeline_start),
                c.length,
                c.fade_in_secs,
                c.fade_out_secs,
                c.clip_start_offset,
                Shared::clone(&c.pcm),
                tempo_map,
            );
        }

        // TODO: Use a persistant data structure to make this more efficient?
        let msg = HandleToProcMsg::UpdateAudioClipList(Owned::new(
            &self.coll_handle,
            self.audio_clips.clone(),
        ));
        self.send(msg);
    }
}

pub struct TimelineTrackPlugMainThread {
    host_request_channel: HostRequestChannelSender,
}

impl TimelineTrackPlugMainThread {
    pub fn new(host_request_channel: HostRequestChannelSender) -> Self {
        Self { host_request_channel }
    }
}

impl PluginMainThread for TimelineTrackPlugMainThread {
    fn activate(
        &mut self,
        sample_rate: SampleRate,
        _min_frames: u32,
        max_frames: u32,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        let (proc_tx, handle_rx) = RingBuffer::<HandleToProcMsg>::new(MSG_BUFFER_SIZE);

        Ok(PluginActivatedInfo {
            audio_thread: Box::new(TimelineTrackPlugAudioThread::new(handle_rx, coll_handle)),
            internal_handle: Some(Box::new(TimelineTrackPlugHandle::new(
                proc_tx,
                coll_handle.clone(),
            ))),
        })
    }

    fn audio_ports_ext(&mut self) -> Result<ext::audio_ports::PluginAudioPortsExt, String> {
        Ok(ext::audio_ports::PluginAudioPortsExt::stereo_out())
    }
}

pub struct TimelineTrackPlugAudioThread {
    audio_clips: Owned<FnvHashMap<TimelineTrackAudioClipID, AudioClipProcState>>,
    handle_rx: Consumer<HandleToProcMsg>,
}

impl TimelineTrackPlugAudioThread {
    fn new(handle_rx: Consumer<HandleToProcMsg>, coll_handle: &basedrop::Handle) -> Self {
        Self { audio_clips: Owned::new(coll_handle, FnvHashMap::default()), handle_rx }
    }
}

impl TimelineTrackPlugAudioThread {
    fn poll(&mut self) {
        while let Ok(msg) = self.handle_rx.pop() {
            match msg {
                HandleToProcMsg::UpdateAudioClipList(new_list) => {
                    self.audio_clips = new_list;
                }
                HandleToProcMsg::UpdateAudioClip(clip) => {
                    if let Some(c) = self.audio_clips.get_mut(&clip.id) {
                        *c = clip;
                    }
                }
            }
        }
    }
}

impl PluginAudioThread for TimelineTrackPlugAudioThread {
    fn start_processing(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn stop_processing(&mut self) {}

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
        in_events: &EventBuffer,
        _out_events: &mut EventBuffer,
    ) -> ProcessStatus {
        self.poll();

        if self.audio_clips.is_empty() {
            buffers.clear_all_outputs(proc_info);
            return ProcessStatus::Continue;
        }

        for audio_clip in self.audio_clips.values() {
            // TODO
        }

        ProcessStatus::Continue
    }

    fn param_flush(&mut self, in_events: &EventBuffer, _out_events: &mut EventBuffer) {}
}
