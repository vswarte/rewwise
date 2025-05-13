use std::collections::VecDeque;
use std::ffi;
use std::io::Read;
use std::num::Wrapping;
use std::sync::mpsc::channel;

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjectId {
    String(String),
    Hash(u32),
}

impl ObjectId {
    pub fn as_hash(&self) -> u32 {
        match self {
            ObjectId::String(s) => create_hash(s),
            ObjectId::Hash(h) => h.clone(),
        }
    }

    fn write(output: &mut BitVec<u8, Msb0>, value: &Self) -> Result<(), DekuError> {
        let hash = value.as_hash();
        u32::write(&hash, output, ())?;
        Ok(())
    }

    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError> {
        let (r, v) = u32::read(rest, ())?;
        Ok((r, Self::Hash(v)))
    }
}

const FNV_BASE: Wrapping<u32> = Wrapping(2166136261);
const FNV_PRIME: Wrapping<u32> = Wrapping(16777619);

pub fn create_hash(input: &str) -> u32 {
    let input_lower = input.to_ascii_lowercase();
    let input_buffer = input_lower.as_bytes();

    let mut result = FNV_BASE;
    for byte in input_buffer {
        result *= FNV_PRIME;
        result ^= *byte as u32;
    }

    result.0
}

#[cfg(test)]
mod test {
    use crate::ObjectId;

    #[test]
    fn hashes_properly() {
        assert!(ObjectId::String("Play_c407001000".to_string()).as_hash() == 1834890111);
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Soundbank {
    #[deku(bits_read = "deku::rest.len()")]
    pub sections: Vec<Section>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct Section {
    #[deku(update = "self.body.deku_id().unwrap()")]
    pub magic: [u8; 4],
    pub size: u32,
    #[deku(ctx = "*magic, *size")]
    pub body: SectionBody,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "magic: [u8; 4], size: u32", id = "magic")]
pub enum SectionBody {
    #[deku(id = b"BKHD")]
    BKHD(#[deku(ctx = "size")] BKHDSection),
    #[deku(id = b"DIDX")]
    DIDX(#[deku(ctx = "size")] DIDXSection),
    #[deku(id = b"DATA")]
    DATA(#[deku(ctx = "size")] DATASection),
    #[deku(id = b"ENVS")]
    ENVS(ENVSSection),
    #[deku(id = b"FXPR")]
    FXPR(#[deku(ctx = "size")] TodoSection),
    #[deku(id = b"HIRC")]
    HIRC(HIRCSection),
    #[deku(id = b"STID")]
    STID(STIDSection),
    #[deku(id = b"STMG")]
    STMG(STMGSection),
    #[deku(id = b"INIT")]
    INIT(INITSection),
    #[deku(id = b"PLAT")]
    PLAT(PLATSection),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ENVSSection {
    pub conversion_table: ConversionTable,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct ConversionTable {
    pub curve_obs_vol: ObsOccCurve,
    pub curve_obs_lpf: ObsOccCurve,
    pub curve_obs_hpf: ObsOccCurve,
    pub curve_occ_vol: ObsOccCurve,
    pub curve_occ_lpf: ObsOccCurve,
    pub curve_occ_hpf: ObsOccCurve,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct ObsOccCurve {
    pub curve_enabled: u8,
    pub curve_scaling: u8,
    #[deku(update = "self.points.len()")]
    point_count: u16,
    #[deku(count = "point_count")]
    pub points: Vec<AkRTPCGraphPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkRTPCGraphPoint {
    pub from: f32,
    pub to: f32,
    pub interpolation: AkCurveInterpolation,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u32")]
pub enum AkCurveInterpolation {
    #[default]
    #[deku(id = "0x0")]
    Log3,
    #[deku(id = "0x1")]
    Sine,
    #[deku(id = "0x2")]
    Log1,
    #[deku(id = "0x3")]
    InvSCurve,
    #[deku(id = "0x4")]
    Linear,
    #[deku(id = "0x5")]
    SCurve,
    #[deku(id = "0x6")]
    Exp1,
    #[deku(id = "0x7")]
    SineRecip,
    #[deku(id = "0x8")]
    Exp3,
    #[deku(id = "0x9")]
    Constant,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
#[deku(ctx = "size: u32")]
pub struct BKHDSection {
    pub version: u32,
    pub bank_id: u32,
    pub language_fnv_hash: u32,
    pub wem_alignment: u32,
    pub project_id: u32,

    // This padding is here to align the DATA sections's
    // first WEM to a multiple of wem_alignment.
    #[deku(count = "size - (4 * 5)")]
    pub padding: Vec<u8>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct INITSection {
    #[deku(update = "self.plugins.len()")]
    plugin_count: u32,
    #[deku(count = "plugin_count")]
    pub plugins: Vec<IAkPlugin>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct IAkPlugin {
    pub plugin_id: PluginId,
    #[deku(update = "self.dll_name.as_bytes_with_nul().len()")]
    dll_name_length: u32,
    #[serde(with = "crate::serialization::cstring")]
    pub dll_name: ffi::CString,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct DIDXDescriptor {
    pub id: u32,
    pub offset: u32,
    pub size: u32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
#[deku(ctx = "size: u32")]
pub struct DIDXSection {
    #[deku(bytes_read = "size")]
    pub descriptors: Vec<DIDXDescriptor>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
#[deku(ctx = "size: u32")]
pub struct DATASection {
    #[serde(with = "crate::serialization::base64")]
    #[deku(bytes_read = "size")]
    pub data: Vec<u8>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AkStateTransition {
    from_state: u32,
    to_state: u32,
    transition_time: u32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct STMGSectionStateGroup {
    id: u32,
    default_transition_time: u32,
    #[deku(update = "self.state_transitions.len()")]
    state_transition_count: u32,
    #[deku(count = "state_transition_count")]
    state_transitions: Vec<AkStateTransition>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PLATSection {
    #[deku(update = "self.string.as_bytes_with_nul().len()")]
    string_length: u32,
    #[serde(with = "crate::serialization::cstring")]
    string: ffi::CString,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct HIRCSection {
    #[deku(update = "self.objects.len()")]
    object_count: u32,
    #[deku(count = "object_count")]
    pub objects: Vec<HIRCObject>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
#[deku(ctx = "size: u32")]
pub struct TodoSection {
    #[serde(with = "crate::serialization::base64")]
    #[deku(bytes_read = "size")]
    data: Vec<u8>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct STMGSection {
    pub volume_threshold: f32,
    pub max_voice_instances: u16,
    pub max_num_dangerous_virt_voices_limit_internal: u16,
    #[deku(update = "self.state_groups.len()")]
    state_group_count: u32,
    #[deku(count = "state_group_count")]
    pub state_groups: Vec<StateGroup>,
    #[deku(update = "self.switch_groups.len()")]
    switch_group_count: u32,
    #[deku(count = "switch_group_count")]
    pub switch_groups: Vec<SwitchGroup>,
    #[deku(update = "self.ramping_params.len()")]
    ramping_param_count: u32,
    #[deku(count = "ramping_param_count")]
    pub ramping_params: Vec<RTPCRamping>,
    #[deku(update = "self.textures.len()")]
    texture_count: u32,
    #[deku(count = "texture_count")]
    pub textures: Vec<AkAcousticTexture>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct StateGroup {
    pub id: u32,
    pub default_transition_time: u32,
    #[deku(update = "self.transitions.len()")]
    transition_count: u32,
    #[deku(count = "transition_count")]
    pub transitions: Vec<AkStateTransition>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchGroup {
    pub id: u32,
    pub rtpc_id: u32,
    pub rtpc_type: u8,
    #[deku(update = "self.graph_points.len()")]
    graph_point_count: u32,
    #[deku(count = "graph_point_count")]
    pub graph_points: Vec<AkSwitchGraphPoint>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AkSwitchGraphPoint {
    pub rtpc_value: f32,
    pub switch: u32,
    pub curve_shape: u32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct RTPCRamping {
    pub rtpc_id: u32,
    pub value: u32,
    pub ramp_type: u32,
    pub ramp_up: f32,
    pub ramp_down: f32,
    pub bind_to_built_in_param: i8,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AkAcousticTexture {
    pub id: u32,
    pub absorption_offset: f32,
    pub absorption_low: f32,
    pub absorption_mid_low: f32,
    pub absorption_mid_high: f32,
    pub absorption_high: f32,
    pub scattering: f32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct STIDSectionEntry {
    pub bnk_id: u32,
    #[deku(update = "self.name.len()")]
    name_length: u8,
    #[serde(with = "crate::serialization::bytestring")]
    #[deku(count = "name_length")]
    pub name: Vec<u8>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Serialize, Deserialize)]
pub struct STIDSection {
    pub string_encoding: u32,
    #[deku(update = "self.entries.len()")]
    entry_count: u32,
    #[deku(count = "entry_count")]
    pub entries: Vec<STIDSectionEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct HIRCObject {
    #[deku(update = "self.body.deku_id().unwrap()")]
    pub body_type: u8,
    pub size: u32,

    #[deku(
        reader = "ObjectId::read(deku::rest)",
        writer = "ObjectId::write(deku::output, &self.id)"
    )]
    pub id: ObjectId,

    #[deku(ctx = "*body_type, *size")]
    pub body: HIRCObjectBody,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "body_type: u8, size: u32", id = "body_type")]
pub enum HIRCObjectBody {
    #[deku(id = "01")]
    State(CAkState),
    #[deku(id = "02")]
    Sound(CAkSound),
    #[deku(id = "03")]
    Action(CAkAction),
    #[deku(id = "04")]
    Event(CAkEvent),
    #[deku(id = "05")]
    RandomSequenceContainer(CAkRanSeqCntr),
    #[deku(id = "06")]
    SwitchContainer(CAkSwitchCntr),
    #[deku(id = "07")]
    ActorMixer(CAkActorMixer),
    #[deku(id = "08")]
    Bus(CAkBus),
    #[deku(id = "09")]
    LayerContainer(CAkLayerCntr),
    #[deku(id = "10")]
    MusicSegment(CAkMusicSegment),
    #[deku(id = "11")]
    MusicTrack(CAkMusicTrack),
    #[deku(id = "12")]
    MusicSwitchContainer(CAkMusicSwitchCntr),
    #[deku(id = "13")]
    MusicRandomSequenceContainer(CAkMusicRanSeqCntr),
    #[deku(id = "14")]
    Attenuation(CAkAttentuation),
    #[deku(id = "15")]
    DialogueEvent(CAkDialogueEvent),
    #[deku(id = "16")]
    EffectShareSet(CAkFxShareSet),
    #[deku(id = "17")]
    EffectCustom(CAkFxCustom),
    #[deku(id = "18")]
    AuxiliaryBus(CAkAuxBus),
    #[deku(id = "19")]
    LFOModulator(#[deku(ctx = "size")] TodoObject),
    #[deku(id = "20")]
    EnvelopeModulator(#[deku(ctx = "size")] TodoObject),
    #[deku(id = "21")]
    AudioDevice(CAkAudioDevice),
    #[deku(id = "22")]
    TimeModulator(CAkTimeModulator),
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkPropID {
    #[deku(id = "0x00")]
    Volume,
    #[deku(id = "0x01")]
    LFE,
    #[deku(id = "0x02")]
    Pitch,
    #[deku(id = "0x03")]
    LPF,
    #[deku(id = "0x04")]
    HPF,
    #[deku(id = "0x05")]
    BusVolume,
    #[deku(id = "0x06")]
    MakeUpGain,
    #[deku(id = "0x07")]
    Priority,
    #[deku(id = "0x08")]
    PriorityDistanceOffset,
    #[deku(id = "0x0B")]
    MuteRatio,
    #[deku(id = "0x0C")]
    PanLR,
    #[deku(id = "0x0D")]
    PanFR,
    #[deku(id = "0x0E")]
    CenterPCT,
    #[deku(id = "0x0F")]
    DelayTime,
    #[deku(id = "0x10")]
    TransitionTime,
    #[deku(id = "0x11")]
    Probability,
    #[deku(id = "0x12")]
    DialogueMode,
    #[deku(id = "0x13")]
    UserAuxSendVolume0,
    #[deku(id = "0x14")]
    UserAuxSendVolume1,
    #[deku(id = "0x15")]
    UserAuxSendVolume2,
    #[deku(id = "0x16")]
    UserAuxSendVolume3,
    #[deku(id = "0x17")]
    GameAuxSendVolume,
    #[deku(id = "0x18")]
    OutputBusVolume,
    #[deku(id = "0x19")]
    OutputBusHPF,
    #[deku(id = "0x1A")]
    OutputBusLPF,
    #[deku(id = "0x1B")]
    HDRBusThreshold,
    #[deku(id = "0x1C")]
    HDRBusRatio,
    #[deku(id = "0x1D")]
    HDRBusReleaseTime,
    #[deku(id = "0x1E")]
    HDRBusGameParam,
    #[deku(id = "0x1F")]
    HDRBusGameParamMin,
    #[deku(id = "0x20")]
    HDRBusGameParamMax,
    #[deku(id = "0x21")]
    HDRActiveRange,
    #[deku(id = "0x22")]
    LoopStart,
    #[deku(id = "0x23")]
    LoopEnd,
    #[deku(id = "0x24")]
    TrimInTime,
    #[deku(id = "0x25")]
    TrimOutTime,
    #[deku(id = "0x26")]
    FadeInTime,
    #[deku(id = "0x27")]
    FadeOutTime,
    #[deku(id = "0x28")]
    FadeInCurve,
    #[deku(id = "0x29")]
    FadeOutCurve,
    #[deku(id = "0x2A")]
    LoopCrossfadeDuration,
    #[deku(id = "0x2B")]
    CrossfadeUpCurve,
    #[deku(id = "0x2C")]
    CrossfadeDownCurve,
    #[deku(id = "0x2D")]
    MidiTrackingRootNote,
    #[deku(id = "0x2E")]
    MidiPlayOnNoteType,
    #[deku(id = "0x2F")]
    MidiTransposition,
    #[deku(id = "0x30")]
    MidiVelocityOffset,
    #[deku(id = "0x31")]
    MidiKeyRangeMin,
    #[deku(id = "0x32")]
    MidiKeyRangeMax,
    #[deku(id = "0x33")]
    MidiVelocityRangeMin,
    #[deku(id = "0x34")]
    MidiVelocityRangeMax,
    #[deku(id = "0x35")]
    MidiChannelMask,
    #[deku(id = "0x36")]
    PlaybackSpeed,
    #[deku(id = "0x37")]
    MidiTempoSource,
    #[deku(id = "0x38")]
    MidiTargetNode,
    #[deku(id = "0x39")]
    AttachedPluginFXID,
    #[deku(id = "0x3A")]
    Loop,
    #[deku(id = "0x3B")]
    InitialDelay,
    #[deku(id = "0x3C")]
    UserAuxSendLPF0,
    #[deku(id = "0x3D")]
    UserAuxSendLPF1,
    #[deku(id = "0x3E")]
    UserAuxSendLPF2,
    #[deku(id = "0x3F")]
    UserAuxSendLPF3,
    #[deku(id = "0x40")]
    UserAuxSendHPF0,
    #[deku(id = "0x41")]
    UserAuxSendHPF1,
    #[deku(id = "0x42")]
    UserAuxSendHPF2,
    #[deku(id = "0x43")]
    UserAuxSendHPF3,
    #[deku(id = "0x44")]
    GameAuxSendLPF,
    #[deku(id = "0x45")]
    GameAuxSendHPF,
    #[deku(id = "0x46")]
    AttenuationID,
    #[deku(id = "0x47")]
    PositioningTypeBlend,
    #[deku(id = "0x48")]
    ReflectionBusVolume,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkParameterID {
    #[deku(id = "0x0")]
    Volume,
    #[deku(id = "0x1")]
    LFE,
    #[deku(id = "0x2")]
    Pitch,
    #[deku(id = "0x3")]
    LPF,
    #[deku(id = "0x4")]
    HPF,
    #[deku(id = "0x5")]
    BusVolume,
    #[deku(id = "0x6")]
    InitialDelay,
    #[deku(id = "0x7")]
    MakeUpGain,
    #[deku(id = "0x8")]
    DeprecatedFeedbackVolume,
    #[deku(id = "0x9")]
    DeprecatedFeedbackLowpass,
    #[deku(id = "0xA")]
    DeprecatedFeedbackPitch,
    #[deku(id = "0xB")]
    MidiTransposition,
    #[deku(id = "0xC")]
    MidiVelocityOffset,
    #[deku(id = "0xD")]
    PlaybackSpeed,
    #[deku(id = "0xE")]
    MuteRatio,
    #[deku(id = "0xF")]
    PlayMechanismSpecialTransitionsValue,
    #[deku(id = "0x10")]
    MaxNumInstances,
    #[deku(id = "0x11")]
    Priority,
    #[deku(id = "0x12")]
    PositionPANX2D,
    #[deku(id = "0x13")]
    PositionPANY2D,
    #[deku(id = "0x14")]
    PositionPANX3D,
    #[deku(id = "0x15")]
    PositionPANY3D,
    #[deku(id = "0x16")]
    PositionPANZ3D,
    #[deku(id = "0x17")]
    PositioningTypeBlend,
    #[deku(id = "0x18")]
    PositioningDivergenceCenterPCT,
    #[deku(id = "0x19")]
    PositioningConeAttenuationONOFF,
    #[deku(id = "0x1A")]
    PositioningConeAttenuation,
    #[deku(id = "0x1B")]
    PositioningConeLPF,
    #[deku(id = "0x1C")]
    PositioningConeHPF,
    #[deku(id = "0x1D")]
    BypassFX0,
    #[deku(id = "0x1E")]
    BypassFX1,
    #[deku(id = "0x1F")]
    BypassFX2,
    #[deku(id = "0x20")]
    BypassFX3,
    #[deku(id = "0x21")]
    BypassAllFX,
    #[deku(id = "0x22")]
    HDRBusThreshold,
    #[deku(id = "0x23")]
    HDRBusReleaseTime,
    #[deku(id = "0x24")]
    HDRBusRatio,
    #[deku(id = "0x25")]
    HDRActiveRange,
    #[deku(id = "0x26")]
    GameAuxSendVolume,
    #[deku(id = "0x27")]
    UserAuxSendVolume0,
    #[deku(id = "0x28")]
    UserAuxSendVolume1,
    #[deku(id = "0x29")]
    UserAuxSendVolume2,
    #[deku(id = "0x2A")]
    UserAuxSendVolume3,
    #[deku(id = "0x2B")]
    OutputBusVolume,
    #[deku(id = "0x2C")]
    OutputBusHPF,
    #[deku(id = "0x2D")]
    OutputBusLPF,
    #[deku(id = "0x2E")]
    PositioningEnableAttenuation,
    #[deku(id = "0x2F")]
    ReflectionsVolume,
    #[deku(id = "0x30")]
    UserAuxSendLPF0,
    #[deku(id = "0x31")]
    UserAuxSendLPF1,
    #[deku(id = "0x32")]
    UserAuxSendLPF2,
    #[deku(id = "0x33")]
    UserAuxSendLPF3,
    #[deku(id = "0x34")]
    UserAuxSendHPF0,
    #[deku(id = "0x35")]
    UserAuxSendHPF1,
    #[deku(id = "0x36")]
    UserAuxSendHPF2,
    #[deku(id = "0x37")]
    UserAuxSendHPF3,
    #[deku(id = "0x38")]
    GameAuxSendLPF,
    #[deku(id = "0x39")]
    GameAuxSendHPF,
    #[deku(id = "0x3A")]
    PositionPANZ2D,
    #[deku(id = "0x3B")]
    BypassAllMetadata,
    #[deku(id = "0x3D")]
    Custom1,
    #[deku(id = "0x3E")]
    Custom2,
    #[deku(id = "0x3F")]
    Custom3,
}

// Incomplete but I best enable them when I have examples to work off of
#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "action_type: u16", id = "action_type")]
pub enum CAkActionParams {
    // #[deku(id="0x0000")] None,
    #[deku(id = "0x1204")]
    SetState(CAkActionSetSwitch),
    // #[deku(id="0x1A02")] BypassFXM,
    // #[deku(id="0x1A03")] BypassFXO,
    // #[deku(id="0x1B02")] ResetBypassFXM,
    // #[deku(id="0x1B03")] ResetBypassFXO,
    // #[deku(id="0x1B04")] ResetBypassFXALL,
    // #[deku(id="0x1B05")] ResetBypassFXALLO,
    // #[deku(id="0x1B08")] ResetBypassFXAE,
    // #[deku(id="0x1B09")] ResetBypassFXAEO,
    #[deku(id = "0x1901")]
    SetSwitch(CAkActionSetSwitch),
    // #[deku(id="0x1002")] UseStateE,
    // #[deku(id="0x1102")] UnuseStateE,
    #[deku(id = "0x0403")]
    Play(CAkActionPlay),
    // #[deku(id="0x0503")] PlayAndContinue,
    #[deku(id = "0x0102")]
    StopE(CAkActionStop),
    #[deku(id = "0x0103")]
    StopEO(CAkActionStop),
    // #[deku(id="0x0104")] StopALL,
    // #[deku(id="0x0105")] StopALLO,
    // #[deku(id="0x0108")] StlopAE,
    // #[deku(id="0x0109")] StopAEO,
    #[deku(id = "0x0202")]
    PauseE(CAkActionPause),
    // #[deku(id="0x0203")] PauseEO,
    // #[deku(id="0x0204")] PauseALL,
    // #[deku(id="0x0205")] PauseALLO,
    // #[deku(id="0x0208")] PauseAE,
    // #[deku(id="0x0209")] PauseAEO,
    #[deku(id = "0x0302")]
    ResumeE(CAkActionResume),
    // #[deku(id="0x0303")] ResumeEO,
    // #[deku(id="0x0304")] ResumeALL,
    // #[deku(id="0x0305")] ResumeALLO,
    // #[deku(id="0x0308")] ResumeAE,
    // #[deku(id="0x0309")] ResumeAEO,
    // #[deku(id="0x1C02")] BreakE,
    // #[deku(id="0x1C03")] BreakEO,
    #[deku(id = "0x0602")]
    MuteM(CAkActionMute),
    #[deku(id = "0x0603")]
    MuteO(CAkActionMute),
    #[deku(id = "0x0702")]
    UnmuteM(CAkActionMute),
    #[deku(id = "0x0703")]
    UnmuteO(CAkActionMute),
    #[deku(id = "0x0704")]
    UnmuteALL(CAkActionMute),
    #[deku(id = "0x0705")]
    UnmuteALLO(CAkActionMute),
    #[deku(id = "0x0708")]
    UnmuteAE(CAkActionMute),
    #[deku(id = "0x0709")]
    UnmuteAEO(CAkActionMute),
    #[deku(id = "0x0A02")]
    SetVolumeM(CAkActionSetAkProp),
    #[deku(id = "0x0A03")]
    SetVolumeO(CAkActionSetAkProp),
    #[deku(id = "0x0B02")]
    ResetVolumeM(CAkActionSetAkProp),
    #[deku(id = "0x0B03")]
    ResetVolumeO(CAkActionSetAkProp),
    #[deku(id = "0x0B04")]
    ResetVolumeALL(CAkActionSetAkProp),
    // #[deku(id="0x0B05")] ResetVolumeALLO,
    // #[deku(id="0x0B08")] ResetVolumeAE,
    // #[deku(id="0x0B09")] ResetVolumeAEO,
    #[deku(id = "0x0802")]
    SetPitchM(CAkActionSetAkProp),
    #[deku(id = "0x0803")]
    SetPitchO(CAkActionSetAkProp),
    #[deku(id = "0x0902")]
    ResetPitchM(CAkActionSetAkProp),
    #[deku(id = "0x0903")]
    ResetPitchO(CAkActionSetAkProp),
    // #[deku(id="0x0904")] ResetPitchALL,
    // #[deku(id="0x0905")] ResetPitchALLO,
    // #[deku(id="0x0908")] ResetPitchAE,
    // #[deku(id="0x0909")] ResetPitchAEO,
    #[deku(id = "0x0E02")]
    SetLPFM(CAkActionSetAkProp),
    #[deku(id = "0x0E03")]
    SetLPFO(CAkActionSetAkProp),
    #[deku(id = "0x0F02")]
    ResetLPFM(CAkActionSetAkProp),
    #[deku(id = "0x0F03")]
    ResetLPFO(CAkActionSetAkProp),
    #[deku(id = "0x0F04")]
    ResetLPFALL(CAkActionSetAkProp),
    // #[deku(id="0x0F05")] ResetLPFALLO,
    // #[deku(id="0x0F08")] ResetLPFAE,
    // #[deku(id="0x0F09")] ResetLPFAEO,
    #[deku(id="0x2002")]
    SetHPFM(CAkActionSetAkProp),
    // #[deku(id="0x2003")] SetHPFO,
    #[deku(id="0x3002")]
    ResetHPFM(CAkActionSetAkProp),
    // #[deku(id="0x3003")] ResetHPFO,
    #[deku(id = "0x3004")]
    ResetHPFALL(CAkActionSetAkProp),
    // #[deku(id="0x3005")] ResetHPFALLO,
    // #[deku(id="0x3008")] ResetHPFAE,
    // #[deku(id="0x3009")] ResetHPFAEO,
    #[deku(id = "0x0C02")]
    SetBusVolumeM(CAkActionSetAkProp),
    // #[deku(id="0x0C03")] SetBusVolumeO,
    #[deku(id = "0x0D02")]
    ResetBusVolumeM(CAkActionSetAkProp),
    // #[deku(id="0x0D03")] ResetBusVolumeO,
    #[deku(id = "0x0D04")]
    ResetBusVolumeALL(CAkActionSetAkProp),
    // #[deku(id="0x0D08")] ResetBusVolumeAE,
    #[deku(id = "0x2103")]
    PlayEvent,
    // #[deku(id="0x1511")] StopEvent,
    // #[deku(id="0x1611")] PauseEvent,
    // #[deku(id="0x1711")] ResumeEvent,
    // #[deku(id="0x1820")] Duck,
    // #[deku(id="0x1D00")] Trigger,
    // #[deku(id="0x1D01")] TriggerO,
    // #[deku(id="0x1E02")] SeekE,
    // #[deku(id="0x1E03")] SeekEO,
    // #[deku(id="0x1E04")] SeekALL,
    // #[deku(id="0x1E05")] SeekALLO,
    // #[deku(id="0x1E08")] SeekAE,
    // #[deku(id="0x1E09")] SeekAEO,
    // #[deku(id="0x2202")] ResetPlaylistE,
    // #[deku(id="0x2203")] ResetPlaylistEO,
    // #[deku(id="0x1302")] SetGameParameter,
    // #[deku(id="0x1303")] SetGameParameterO,
    // #[deku(id="0x1402")] ResetGameParameter,
    // #[deku(id="0x1403")] ResetGameParameterO,
    // #[deku(id="0x1F02")] Release,
    // #[deku(id="0x1F03")] ReleaseO,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkPathMode {
    #[default]
    #[deku(id = "0x0")]
    StepSequence,
    #[deku(id = "0x1")]
    StepRandom,
    #[deku(id = "0x2")]
    ContinuousSequence,
    #[deku(id = "0x3")]
    ContinuousRandom,
    #[deku(id = "0x4")]
    StepSequencePickNewPath,
    #[deku(id = "0x5")]
    StepRandomPickNewPath,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum Ak3DSpatializationMode {
    #[default]
    #[deku(id = "0x0")]
    None,
    #[deku(id = "0x1")]
    PositionOnly,
    #[deku(id = "0x2")]
    PositionAndOrientation,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum AkSpeakerPanningType {
    #[deku(id = "0x0")]
    DirectSpeakerAssignment,
    #[deku(id = "0x1")]
    BalanceFadeHeight,
    #[deku(id = "0x2")]
    SteeringPanner,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "2")]
pub enum Ak3DPositionType {
    #[deku(id = "0x0")]
    Emitter,
    #[deku(id = "0x1")]
    EmitterWithAutomation,
    #[deku(id = "0x2")]
    ListenerWithAutomation,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkVirtualQueueBehavior {
    #[deku(id = "0x0")]
    PlayFromBeginning,
    #[deku(id = "0x1")]
    PlayFromElapsedTime,
    #[deku(id = "0x2")]
    Resume,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkBelowThresholdBehavior {
    #[deku(id = "0x0")]
    ContinueToPlay,
    #[deku(id = "0x1")]
    KillVoice,
    #[deku(id = "0x2")]
    SetAsVirtualVoice,
    #[deku(id = "0x3")]
    KillIfOneShotElseVirtual,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u32")]
pub enum AkSyncType {
    #[deku(id = "0x0")]
    Immediate,
    #[deku(id = "0x1")]
    NextGrid,
    #[deku(id = "0x2")]
    NextBar,
    #[deku(id = "0x3")]
    NextBeat,
    #[deku(id = "0x4")]
    NextMarket,
    #[deku(id = "0x5")]
    NextUserMarker,
    #[deku(id = "0x6")]
    EntryMarker,
    #[deku(id = "0x7")]
    ExitMarker,
    #[deku(id = "0x8")]
    ExitNever,
    #[deku(id = "0x9")]
    LastExitPosition,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkSyncTypeU8 {
    #[deku(id = "0x0")]
    Immediate,
    #[deku(id = "0x1")]
    NextGrid,
    #[deku(id = "0x2")]
    NextBar,
    #[deku(id = "0x3")]
    NehxtBeat,
    #[deku(id = "0x4")]
    NextMarket,
    #[deku(id = "0x5")]
    NextUserMarker,
    #[deku(id = "0x6")]
    EntryMarker,
    #[deku(id = "0x7")]
    ExitMarker,
    #[deku(id = "0x8")]
    ExitNever,
    #[deku(id = "0x9")]
    LastExitPosition,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkRtpcAccum {
    #[deku(id = "0x0")]
    None,
    #[deku(id = "0x1")]
    Exclusive,
    #[deku(id = "0x2")]
    Additive,
    #[deku(id = "0x3")]
    Multiply,
    #[deku(id = "0x4")]
    Boolean,
    #[deku(id = "0x5")]
    Maximum,
    #[deku(id = "0x6")]
    Filter,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkRtpcType {
    #[deku(id = "0x0")]
    GameParameter,
    #[deku(id = "0x1")]
    MIDIParameter,
    #[deku(id = "0x2")]
    Modulator,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkCurveScaling {
    #[deku(id = "0x0")]
    None,
    #[deku(id = "0x2")]
    DB,
    #[deku(id = "0x3")]
    Log,
    #[deku(id = "0x4")]
    DBToLin,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkCurveInterpolationU8 {
    #[deku(id = "0x0")]
    Log3,
    #[deku(id = "0x1")]
    Sine,
    #[deku(id = "0x2")]
    Log1,
    #[deku(id = "0x3")]
    InvSCurve,
    #[deku(id = "0x4")]
    Linear,
    #[deku(id = "0x5")]
    SCurve,
    #[deku(id = "0x6")]
    Exp1,
    #[deku(id = "0x7")]
    SineRecip,
    #[deku(id = "0x8")]
    Exp3,
    #[deku(id = "0x9")]
    Constant,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkGroupType {
    #[deku(id = "0x0")]
    Switch,
    #[deku(id = "0x1")]
    State,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum AkDecisionTreeMode {
    #[deku(id = "0x0")]
    BestMatch,
    #[deku(id = "0x1")]
    Weighted,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "size: u32")]
pub struct TodoObject {
    #[serde(with = "crate::serialization::base64")]
    #[deku(count = "size - 4")]
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkMusicSwitchCntr {
    pub music_trans_node_params: MusicTransNodeParams,
    pub continue_playback: u8,
    #[deku(update = "self.arguments.len()")]
    pub tree_depth: u32,
    #[deku(count = "tree_depth")]
    pub arguments: Vec<AkGameSync>,
    #[deku(count = "tree_depth")]
    pub group_types: Vec<AkGroupType>,

    pub tree_size: u32,
    pub tree_mode: AkDecisionTreeMode,
    #[deku(
        reader = "AkDecisionTreeNode::read(
            deku::rest,
            *tree_size,
            *tree_depth,
            0
        )",
        writer = "AkDecisionTreeNode::write(
            deku::output,
            &self.tree,
        )"
    )]
    pub tree: AkDecisionTreeNode,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkDialogueEvent {
    pub probability: u8,
    #[deku(update = "self.arguments.len()")]
    pub tree_depth: u32,
    #[deku(count = "tree_depth")]
    pub arguments: Vec<AkGameSync>,
    #[deku(count = "tree_depth")]
    pub group_types: Vec<AkGroupType>,

    pub tree_size: u32,
    pub tree_mode: AkDecisionTreeMode,
    #[deku(
        reader = "AkDecisionTreeNode::read(
            deku::rest,
            *tree_size,
            *tree_depth,
            0
        )",
        writer = "AkDecisionTreeNode::write(
            deku::output,
            &self.tree,
        )"
    )]
    pub tree: AkDecisionTreeNode,

    #[deku(
        reader = "PropBundle::read_list(
            deku::rest,
        )",
        writer = "PropBundle::write_list(
            deku::output,
            &self.prop_bundle.iter().collect::<Vec<_>>(),
        )"
    )]
    pub prop_bundle: Vec<PropBundle>,
    pub ranged_modifiers: PropRangedModifiers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AkDecisionTreeNode {
    pub key: u32,
    pub node_id: u32,
    pub first_child_index: u16,
    pub child_count: u16,
    pub weight: u16,
    pub probability: u16,
    pub children: Vec<AkDecisionTreeNode>,
}

impl AkDecisionTreeNode {
    fn read(
        rest: &BitSlice<u8, Msb0>,
        tree_size: u32,
        tree_depth: u32,
        current_depth: u32,
    ) -> Result<(&BitSlice<u8, Msb0>, AkDecisionTreeNode), DekuError> {
        fn parse_nodes(
            slice: &BitSlice<u8, Msb0>,
            tree_size: u32,
            tree_depth: u32,
            current_depth: u32,
            start_index: u16,
            count: u16,
        ) -> Result<Vec<AkDecisionTreeNode>, DekuError> {
            let mut items = Vec::new();

            let mut offset = start_index;
            for i in 0..count {
                items.push(parse_node(
                    slice,
                    tree_size,
                    tree_depth,
                    current_depth,
                    offset,
                )?);
                offset += 1;
            }

            Ok(items)
        }

        fn parse_node(
            slice: &BitSlice<u8, Msb0>,
            tree_size: u32,
            tree_depth: u32,
            current_depth: u32,
            offset: u16,
        ) -> Result<AkDecisionTreeNode, DekuError> {
            let (_, data) = slice.split_at(offset as usize * 8 * 0xC);

            let (data, key) = u32::read(data, ())?;
            let (data, node_id, first_child_index, child_count) = {
                let (data, node_id) = u32::read(data, ())?;

                let first_child_index = (node_id & 0xFFFF) as u16;
                let child_count = (node_id >> 16 & 0xFFFF) as u16;

                // If it's reliable enough for the wwiser people...
                if first_child_index > tree_size as u16
                    || child_count > tree_size as u16
                    || current_depth == tree_depth
                {
                    (data, node_id, 0, 0)
                } else {
                    (data, 0, first_child_index, child_count)
                }
            };

            let (data, weight) = u16::read(data, ())?;
            let (data, probability) = u16::read(data, ())?;

            let children = parse_nodes(
                slice,
                tree_size,
                tree_depth,
                current_depth + 1,
                first_child_index,
                child_count,
            )?;

            Ok(AkDecisionTreeNode {
                key,
                node_id,
                first_child_index,
                child_count,
                weight,
                probability,
                children,
            })
        }

        let (tree_data, rest) = rest.split_at(tree_size as usize * 8);
        let root = parse_node(tree_data, tree_size, tree_depth, 0, 0)?;

        Ok((rest, root))
    }

    pub fn write(
        output: &mut BitVec<u8, Msb0>,
        node: &AkDecisionTreeNode,
    ) -> Result<(), DekuError> {
        let mut next_child_index = 1u16;

        let mut pending = VecDeque::new();
        pending.push_front(node);

        while let Some(node) = pending.pop_front() {
            node.key.write(output, ())?;

            if node.child_count == 0 {
                node.node_id.write(output, ())?;
            } else {
                next_child_index.write(output, ())?;

                let child_count = node.children.len() as u16;
                child_count.write(output, ())?;
                next_child_index += child_count;
            }

            node.weight.write(output, ())?;
            node.probability.write(output, ())?;

            for node in node.children.iter() {
                pending.push_back(node);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkGameSync {
    pub group_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkFxShareSet {
    pub fx_base_initial_values: FxBaseInitialValues,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkTimeModulator {
    #[deku(
        reader = "PropBundle::read_list(
            deku::rest,
        )",
        writer = "PropBundle::write_list(
            deku::output,
            &self.prop_bundle.iter().collect::<Vec<_>>(),
        )"
    )]
    pub prop_bundle: Vec<PropBundle>,
    pub ranged_modifiers: PropRangedModifiers,
    pub initial_rtpc: InitialRTPC,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkMusicRanSeqCntr {
    pub music_trans_node_params: MusicTransNodeParams,
    #[deku(update = "self.playlist_items.len()")]
    playlist_item_count: u32,
    #[deku(count = "playlist_item_count")]
    pub playlist_items: Vec<AkMusicRanSeqPlaylistItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicRanSeqPlaylistItem {
    segment_id: u32,
    playlist_item_id: i32,
    child_count: u32,
    ers_type: u32,
    loop_base: i16,
    loop_min: i16,
    loop_max: i16,
    weight: u32,
    avoid_repeat_count: u16,
    use_weight: u8,
    shuffle: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct MusicTransNodeParams {
    pub music_node_params: MusicNodeParams,
    #[deku(update = "self.transition_rules.len()")]
    transition_rule_count: u32,
    #[deku(count = "transition_rule_count")]
    pub transition_rules: Vec<AkMusicTransitionRule>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicTransitionRule {
    #[deku(update = "self.source_ids.len()")]
    source_transition_rule_count: u32,
    #[deku(count = "source_transition_rule_count")]
    source_ids: Vec<i32>,
    #[deku(update = "self.destination_ids.len()")]
    destination_transition_rule_count: u32,
    #[deku(count = "destination_transition_rule_count")]
    destination_ids: Vec<i32>,
    source_transition_rule: AkMusicTransSrcRule,
    destination_transition_rule: AkMusicTransDstRule,
    alloc_trans_object_flag: u8,
    #[deku(skip, cond = "*alloc_trans_object_flag == 0")]
    transition_object: AkMusicTransitionObject,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicTransitionObject {
    segment_id: u32,
    fade_out: AkMusicFade,
    fade_in: AkMusicFade,
    play_pre_entry: u8,
    play_post_exit: u8,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicFade {
    transition_time: i32,
    curve: AkCurveInterpolation,
    offset: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicTransSrcRule {
    transition_time: i32,
    fade_curve: AkCurveInterpolation,
    fade_offet: i32,
    sync_type: AkSyncType,
    clue_filter_hash: u32,
    play_post_exit: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicTransDstRule {
    transition_time: i32,
    fade_curve: AkCurveInterpolation,
    fade_offet: i32,
    clue_filter_hash: u32,
    jump_to_id: i32,
    jump_to_type: u16,
    entry_type: u16,
    play_pre_entry: u8,
    destination_match_source_cue_name: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkMusicSegment {
    pub music_node_params: MusicNodeParams,
    pub duration: f64,
    #[deku(update = "self.markers.len()")]
    marker_count: u32,
    #[deku(count = "marker_count")]
    pub markers: Vec<AkMusicMarkerWwise>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct MusicNodeParams {
    pub flags: u8,
    pub node_base_params: NodeBaseParams,
    pub children: Children,
    pub meter_info: AkMeterInfo,
    #[deku(update = "self.stingers.len()")]
    stinger_count: u32,
    #[deku(count = "stinger_count")]
    pub stingers: Vec<CAkStinger>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMusicMarkerWwise {
    id: u32,
    position: f64,
    #[deku(update = "
            if self.string.is_empty() {
                0
            } else {
                self.string.as_bytes_with_nul().len()
            }
        ")]
    string_length: u32,
    #[serde(with = "crate::serialization::cstring")]
    #[deku(skip, cond = "*string_length == 0")]
    string: ffi::CString,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMeterInfo {
    pub grid_period: f64,
    pub grid_offset: f64,
    pub tempo: f32,
    pub time_signature_beat_count: u8,
    pub time_signature_beat_value: u8,
    pub meter_info_flag: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkStinger {
    trigger_id: u32,
    segment_id: u32,
    sync_play_at: AkSyncType,
    cue_filter_hash: u32,
    dont_repeat_time: i32,
    segment_look_head_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkMusicTrack {
    pub flags: u8,
    #[deku(update = "self.sources.len()")]
    source_count: u32,
    #[deku(count = "source_count")]
    pub sources: Vec<AkBankSourceData>,
    #[deku(update = "self.playlist.len()")]
    playlist_item_count: u32,
    #[deku(count = "playlist_item_count")]
    pub playlist: Vec<AkTrackSrcInfo>,
    #[deku(skip, cond = "*playlist_item_count == 0")]
    pub subtrack_count: u32,
    #[deku(update = "self.clip_items.len()")]
    clip_item_count: u32,
    #[deku(count = "clip_item_count")]
    pub clip_items: Vec<AkClipAutomation>,
    pub node_base_params: NodeBaseParams,
    pub track_type: u8,
    pub look_ahead_time: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u32")]
pub enum AkClipAutomationType {
    #[deku(id = "0x00")]
    Volume,
    #[deku(id = "0x01")]
    LPF,
    #[deku(id = "0x02")]
    HPF,
    #[deku(id = "0x03")]
    FadeIn,
    #[deku(id = "0x04")]
    FadeOut,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkClipAutomation {
    pub clip_index: u32,
    pub auto_type: AkClipAutomationType,
    #[deku(update = "self.graph_points.len()")]
    graph_point_count: u32,
    #[deku(count = "graph_point_count")]
    pub graph_points: Vec<AkRTPCGraphPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkTrackSrcInfo {
    pub track_id: u32,
    pub source_id: u32,
    pub event_id: u32,
    pub play_at: f64,
    pub begin_trim_offset: f64,
    pub end_trim_offset: f64,
    pub source_duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkFxCustom {
    pub fx_base_initial_values: FxBaseInitialValues,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkAuxBus {
    pub initial_values: BusInitialValues,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkAudioDevice {
    pub fx_base_initial_values: FxBaseInitialValues,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct FxBaseInitialValues {
    pub fx_id: u32,
    #[deku(update = "self.params.len()")]
    params_size: u32,
    #[serde(with = "crate::serialization::base64")]
    #[deku(count = "params_size")]
    pub params: Vec<u8>,
    #[deku(update = "self.media.len()")]
    media_count: u8,
    #[deku(count = "media_count")]
    pub media: Vec<AkMediaMap>,
    pub initial_rtpc: InitialRTPC,
    pub state_chunk: StateChunk,
    #[deku(update = "self.property_values.len()")]
    property_value_count: i16,
    #[deku(count = "property_value_count")]
    pub property_values: Vec<PluginPropertyValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct PluginPropertyValue {
    pub property: AkParameterID,
    pub rtpc_accum: AkRtpcAccum,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMediaMap {
    pub index: u8,
    pub source_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkBus {
    pub initial_values: BusInitialValues,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct BusInitialValues {
    pub override_bus_id: u32,
    #[deku(skip, cond = "*override_bus_id != 0")]
    pub device_share_set_id: u32,
    pub bus_initial_params: BusInitialParams,
    pub recovery_time: i32,
    pub max_duck_volume: f32,
    #[deku(update = "self.ducks.len()")]
    duck_count: u32,
    #[deku(count = "duck_count")]
    pub ducks: Vec<AkDuckInfo>,
    pub bus_initial_fx_params: BusInitialFxParams,
    pub override_attachment_params: u8,
    pub initial_rtpc: InitialRTPC,
    pub state_chunk: StateChunk,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkDuckInfo {
    pub bus_id: u32,
    pub duck_volume: f32,
    pub fade_out_time: i32,
    pub fade_in_time: i32,
    pub fade_curve: AkCurveInterpolationU8,
    pub target_prop: AkPropID,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct BusInitialParams {
    #[deku(
        reader = "PropBundle::read_list(
            deku::rest,
        )",
        writer = "PropBundle::write_list(
            deku::output,
            &self.prop_bundle.iter().collect::<Vec<_>>(),
        )"
    )]
    pub prop_bundle: Vec<PropBundle>,
    pub positioning_params: PositioningParams,
    pub aux_params: AuxParams,
    pub flags: u8,
    pub max_instance_count: u16,
    pub channel_config: u32,
    pub hdr_flags: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct BusInitialFxParams {
    #[deku(update = "self.fx.len()")]
    fx_count: u8,
    #[deku(skip, cond = "*fx_count == 0")]
    pub fx_bypass: u8,
    #[deku(count = "fx_count")]
    pub fx: Vec<FXChunk>,
    pub fx_id_0: u32,
    pub is_share_set_0: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct FXChunk {
    pub fx_index: u8,
    pub fx_id: u32,
    pub is_share_set: u8,
    pub is_rendered: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkAction {
    pub action_type: u16,
    pub external_id: u32,
    pub is_bus: u8,
    #[deku(
        reader = "PropBundle::read_list(
            deku::rest,
        )",
        writer = "PropBundle::write_list(
            deku::output,
            &self.prop_bundle.iter().collect::<Vec<_>>(),
        )"
    )]
    pub prop_bundle: Vec<PropBundle>,
    pub ranged_modifiers: PropRangedModifiers,
    #[deku(ctx = "*action_type")]
    pub params: CAkActionParams,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionSetState {
    pub state_group_id: u32,
    pub target_state_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionSetSwitch {
    pub switch_group_id: u32,
    pub switch_state_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionMute {
    pub fade_curve: u8,
    pub except: CAkActionParamsExcept,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionResume {
    pub fade_curve: u8,
    pub resume: u8,
    pub except: CAkActionParamsExcept,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionSetAkProp {
    pub fade_curve: u8,
    pub set_ak_prop: CAkActionParamsSetAkProp,
    pub except: CAkActionParamsExcept,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionParamsSetAkProp {
    pub value_meaning: u8,
    pub randomizer_modifier: RandomizerModifier,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct RandomizerModifier {
    pub base: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionPlay {
    pub fade_curve: u8,
    pub bank_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionPause {
    pub fade_curve: u8,
    pub pause: CAkActionParamsPause,
    pub except: CAkActionParamsExcept,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionParamsPause {
    flags: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionStop {
    pub stop: CAkActionParamsStop,
    pub except: CAkActionParamsExcept,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionParamsStop {
    flags1: u8,
    flags2: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionParamsExcept {
    #[deku(update = "self.exceptions.len()")]
    count: u8,
    #[deku(count = "count")]
    pub exceptions: Vec<CAkActionParamsExceptEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActionParamsExceptEntry {
    pub object_id: u32,
    pub is_bus: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkPropBundleByte {
    #[deku(update = "self.types.len()")]
    count: u8,
    #[deku(count = "count")]
    pub types: Vec<AkPropID>,
    #[deku(count = "count")]
    pub values: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkSwitchCntr {
    pub node_base_params: NodeBaseParams,
    pub group_type: u8,
    pub group_id: u32,
    pub default_switch: u32,
    pub continuous_validation: u8,
    pub children: Children,
    #[deku(update = "self.switch_groups.len()")]
    switch_group_count: u32,
    #[deku(count = "switch_group_count")]
    pub switch_groups: Vec<CAkSwitchPackage>,
    #[deku(update = "self.switch_params.len()")]
    switch_param_count: u32,
    #[deku(count = "switch_param_count")]
    pub switch_params: Vec<AkSwitchNodeParams>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkSwitchPackage {
    pub switch_id: u32,
    #[deku(update = "self.nodes.len()")]
    node_count: u32,
    #[deku(count = "node_count")]
    pub nodes: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkSwitchNodeParams {
    pub node_id: u32,
    #[deku(bits = "1")]
    unk1: bool,
    #[deku(bits = "1")]
    unk2: bool,
    #[deku(bits = "1")]
    unk3: bool,
    #[deku(bits = "1")]
    unk4: bool,
    #[deku(bits = "1")]
    unk5: bool,
    #[deku(bits = "1")]
    unk6: bool,
    #[deku(bits = "1")]
    pub continue_playback: bool,
    #[deku(bits = "1")]
    pub is_first_only: bool,
    #[deku(bits = "1")]
    unk9: bool,
    #[deku(bits = "1")]
    unk10: bool,
    #[deku(bits = "1")]
    unk11: bool,
    #[deku(bits = "1")]
    unk12: bool,
    #[deku(bits = "1")]
    unk13: bool,
    #[deku(bits = "1")]
    unk14: bool,
    #[deku(bits = "1")]
    unk15: bool,
    #[deku(bits = "1")]
    unk16: bool,
    pub fade_out_time: i32,
    pub fade_in_time: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkActorMixer {
    pub node_base_params: NodeBaseParams,
    pub children: Children,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkLayerCntr {
    pub node_base_params: NodeBaseParams,
    pub children: Children,
    #[deku(update = "self.layers.len()")]
    layer_count: u32,
    #[deku(count = "layer_count")]
    pub layers: Vec<CAkLayer>,
    pub is_continuous_validation: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkLayer {
    pub layer_id: u32,
    pub initial_rtpc: InitialRTPC,
    pub rtpc_id: u32,
    pub rtpc_type: AkRtpcType,
    #[deku(update = "self.associated_children.len()")]
    associated_childen_count: u32,
    #[deku(count = "associated_childen_count")]
    pub associated_children: Vec<CAssociatedChildData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAssociatedChildData {
    pub associated_child_id: u32,
    #[deku(update = "self.graph_points.len()")]
    graph_point_count: u32,
    #[deku(count = "graph_point_count")]
    pub graph_points: Vec<AkRTPCGraphPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkRanSeqCntr {
    pub node_base_params: NodeBaseParams,
    loop_count: u16,
    loop_mod_min: u16,
    loop_mod_max: u16,
    transition_time: f32,
    transition_time_mod_min: f32,
    transition_time_mod_max: f32,
    avoid_repeat_count: u16,
    transition_mode: u8,
    random_mode: u8,
    mode: u8,
    flags: u8,
    pub children: Children,
    pub playlist: CAkPlaylist,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct Children {
    #[deku(update = "self.items.len()")]
    count: u32,
    #[deku(count = "count")]
    pub items: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkPlaylist {
    #[deku(update = "self.items.len()")]
    count: u16,
    #[deku(count = "count")]
    items: Vec<CAkPlaylistItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkPlaylistItem {
    play_id: u32,
    weight: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkState {
    #[deku(update = "self.parameters.len()")]
    entry_count: u16,
    #[deku(count = "entry_count")]
    parameters: Vec<u16>,
    #[deku(count = "entry_count")]
    values: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkAttentuation {
    pub is_cone_enabled: u8,
    #[deku(skip, cond = "*is_cone_enabled == 0x0")]
    pub cone_params: ConeParams,
    pub curves_to_use: [i8; 7],
    #[deku(update = "self.curves.len()")]
    curve_count: u8,
    #[deku(count = "curve_count")]
    pub curves: Vec<CAkConversionTable>,
    pub initial_rtpc: InitialRTPC,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct ConeParams {
    pub inside_degrees: f32,
    pub outside_degrees: f32,
    pub outside_volume: f32,
    pub low_pass: f32,
    pub high_pass: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkConversionTable {
    pub curve_scaling: AkCurveScaling,
    #[deku(update = "self.points.len()")]
    point_count: u16,
    #[deku(count = "point_count")]
    pub points: Vec<AkRTPCGraphPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkEvent {
    #[deku(update = "self.actions.len()")]
    action_count: u8,
    #[deku(count = "action_count")]
    pub actions: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct CAkSound {
    pub bank_source_data: AkBankSourceData,
    pub node_base_params: NodeBaseParams,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkBankSourceData {
    pub plugin: PluginId,
    pub source_type: SourceType,
    pub media_information: AkMediaInformation,
    #[deku(update = "self.params.len()", skip, cond = "plugin.has_params()?")]
    params_size: u32,
    #[serde(with = "crate::serialization::base64")]
    #[deku(count = "params_size")]
    pub params: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum SourceType {
    #[deku(id = "0x0")]
    Embedded,
    #[deku(id = "0x1")]
    PrefetchStreaming,
    #[deku(id = "0x2")]
    Streaming,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(type = "u32")]
pub enum PluginId {
    #[deku(id = "0x00000000")]
    None,
    #[deku(id = "0x00000001")]
    BANK,
    #[deku(id = "0x00010001")]
    PCM,
    #[deku(id = "0x00020001")]
    ADPCM,
    #[deku(id = "0x00030001")]
    XMA,
    #[deku(id = "0x00040001")]
    VORBIS,
    #[deku(id = "0x00050001")]
    WIIADPCM,
    #[deku(id = "0x00070001")]
    PCMEX,
    #[deku(id = "0x00080001")]
    EXTERNALSOURCE,
    #[deku(id = "0x00090001")]
    XWMA,
    #[deku(id = "0x000A0001")]
    AAC,
    #[deku(id = "0x000B0001")]
    FILEPACKAGE,
    #[deku(id = "0x000C0001")]
    ATRAC9,
    #[deku(id = "0x000D0001")]
    VAGHEVAG,
    #[deku(id = "0x000E0001")]
    PROFILERCAPTURE,
    #[deku(id = "0x000F0001")]
    ANALYSISFILE,
    #[deku(id = "0x00100001")]
    MIDI,
    #[deku(id = "0x00110001")]
    OPUSNX,
    #[deku(id = "0x00120001")]
    CAF,
    #[deku(id = "0x00130001")]
    OPUS,
    #[deku(id = "0x00140001")]
    OPUSWEM1,
    #[deku(id = "0x00150001")]
    OPUSWEM2,
    #[deku(id = "0x00160001")]
    SONY360,
    #[deku(id = "0x00640002")]
    WwiseSine,
    #[deku(id = "0x00650002")]
    WwiseSilence,
    #[deku(id = "0x00660002")]
    WwiseToneGenerator,
    #[deku(id = "0x00670003")]
    WwiseUnk1,
    #[deku(id = "0x00680003")]
    WwiseUnk2,
    #[deku(id = "0x00690003")]
    WwiseParametricEQ,
    #[deku(id = "0x006A0003")]
    WwiseDelay,
    #[deku(id = "0x006C0003")]
    WwiseCompressor,
    #[deku(id = "0x006D0003")]
    WwiseExpander,
    #[deku(id = "0x006E0003")]
    WwisePeakLimiter,
    #[deku(id = "0x006F0003")]
    WwiseUnk3,
    #[deku(id = "0x00700003")]
    WwiseUnk4,
    #[deku(id = "0x00730003")]
    WwiseMatrixReverb,
    #[deku(id = "0x00740003")]
    SoundSeedImpact,
    #[deku(id = "0x00760003")]
    WwiseRoomVerb,
    #[deku(id = "0x00770002")]
    SoundSeedAirWind,
    #[deku(id = "0x00780002")]
    SoundSeedAirWoosh,
    #[deku(id = "0x007D0003")]
    WwiseFlanger,
    #[deku(id = "0x007E0003")]
    WwiseGuitarDistortion,
    #[deku(id = "0x007F0003")]
    WwiseConvolutionReverb,
    #[deku(id = "0x00810003")]
    WwiseMeter,
    #[deku(id = "0x00820003")]
    WwiseTimeStretch,
    #[deku(id = "0x00830003")]
    WwiseTremolo,
    #[deku(id = "0x00840003")]
    WwiseRecorder,
    #[deku(id = "0x00870003")]
    WwiseStereoDelay,
    #[deku(id = "0x00880003")]
    WwisePitchShifter,
    #[deku(id = "0x008A0003")]
    WwiseHarmonizer,
    #[deku(id = "0x008B0003")]
    WwiseGain,
    #[deku(id = "0x00940002")]
    WwiseSynthOne,
    #[deku(id = "0x00AB0003")]
    WwiseReflect,
    #[deku(id = "0x00AE0007")]
    System,
    #[deku(id = "0x00B00007")]
    Communication,
    #[deku(id = "0x00B10007")]
    ControllerHeadphones,
    #[deku(id = "0x00B30007")]
    ControllerSpeaker,
    #[deku(id = "0x00B50007")]
    NoOutput,
    #[deku(id = "0x03840009")]
    WwiseSystemOutputSettings,
    #[deku(id = "0x00B70002")]
    SoundSeedGrain,
    #[deku(id = "0x00BA0003")]
    MasteringSuite,
    #[deku(id = "0x00C80002")]
    WwiseAudioInput,
    #[deku(id = "0x01950002")]
    WwiseMotionGenerator1,
    #[deku(id = "0x01950005")]
    WwiseMotionGenerator2,
    #[deku(id = "0x01990002")]
    WwiseMotionSource1,
    #[deku(id = "0x01990005")]
    WwiseMotionSource2,
    #[deku(id = "0x01FB0007")]
    WwiseMotion,
    #[deku(id = "0x044C1073")]
    AuroHeadphone,
    #[deku(id = "0x00671003")]
    McDSPML1,
    #[deku(id = "0x006E1003")]
    McDSPFutzBox,
    #[deku(id = "0x00021033")]
    IZotopeHybridReverb,
    #[deku(id = "0x00031033")]
    IZotopeTrashDistortion,
    #[deku(id = "0x00041033")]
    IZotopeTrashDelay,
    #[deku(id = "0x00051033")]
    IZotopeTrashDynamicsMono,
    #[deku(id = "0x00061033")]
    IZotopeTrashFilters,
    #[deku(id = "0x00071033")]
    IZotopeTrashBoxModeler,
    #[deku(id = "0x00091033")]
    IZotopeTrashMultibandDistortion,
    #[deku(id = "0x006E0403")]
    PlatinumMatrixSurroundMk2,
    #[deku(id = "0x006F0403")]
    PlatinumLoudnessMeter,
    #[deku(id = "0x00710403")]
    PlatinumSpectrumViewer,
    #[deku(id = "0x00720403")]
    PlatinumEffectCollection,
    #[deku(id = "0x00730403")]
    PlatinumMeterWithFilter,
    #[deku(id = "0x00740403")]
    PlatinumSimple3D,
    #[deku(id = "0x00750403")]
    PlatinumUpmixer,
    #[deku(id = "0x00760403")]
    PlatinumReflection,
    #[deku(id = "0x00770403")]
    PlatinumDownmixer,
    #[deku(id = "0x00780403")]
    PlatinumFlex,
    #[deku(id = "0x00020403")]
    CodemastersEffect,
    #[deku(id = "0x00640332")]
    Ubisoft,
    #[deku(id = "0x04F70803")]
    UbisoftEffect1,
    #[deku(id = "0x04F80806")]
    UbisoftMixer,
    #[deku(id = "0x04F90803")]
    UbisoftEffect2,
    #[deku(id = "0x00AA1137")]
    MicrosoftSpatialSound,
    #[deku(id = "0x000129A3")]
    CPRimpleDelay,
    #[deku(id = "0x000229A2")]
    CPRVoiceBroadcastReceive1,
    #[deku(id = "0x000329A3")]
    CPRVoiceBroadcastSend1,
    #[deku(id = "0x000429A2")]
    CPRVoiceBroadcastReceive2,
    #[deku(id = "0x000529A3")]
    CPRVoiceBroadcastSend2,
    #[deku(id = "0x01A01052")]
    CrankcaseREVModelPlayer,
}

impl PluginId {
    fn has_params(&self) -> Result<bool, DekuError> {
        Ok(self.deku_id()? & 0x0F != 0x2)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkMediaInformation {
    pub source_id: u32,
    pub in_memory_media_size: u32,
    pub source_flags: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct NodeBaseParams {
    pub node_initial_fx_parameters: NodeInitialFxParams,
    pub override_attachment_params: u8,
    pub override_bus_id: u32,
    pub direct_parent_id: u32,
    pub unknown_flags: u8,
    pub node_initial_params: NodeInitialParams,
    pub positioning_params: PositioningParams,
    pub aux_params: AuxParams,
    pub adv_settings_params: AdvSettingsParams,
    pub state_chunk: StateChunk,
    pub initial_rtpc: InitialRTPC,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct NodeInitialFxParams {
    pub is_override_parent_fx: u8,
    #[deku(update = "self.fx_chunks.len()")]
    fx_chunk_count: u8,
    #[deku(skip, cond = "*fx_chunk_count == 0")]
    pub fx_bypass_bits: u8,
    #[deku(count = "fx_chunk_count")]
    pub fx_chunks: Vec<FXChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct NodeInitialParams {
    #[deku(
        reader = "PropBundle::read_list(
            deku::rest,
        )",
        writer = "PropBundle::write_list(
            deku::output,
            &self.prop_initial_values.iter().collect::<Vec<_>>(),
        )"
    )]
    pub prop_initial_values: Vec<PropBundle>,
    pub prop_ranged_modifiers: PropRangedModifiers,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "prop_id: u8", id = "prop_id")]
pub enum PropBundle {
    #[deku(id = "0x00")]
    Volume(f32),
    #[deku(id = "0x01")]
    LFE(f32),
    #[deku(id = "0x02")]
    Pitch(f32),
    #[deku(id = "0x03")]
    LPF(f32),
    #[deku(id = "0x04")]
    HPF(f32),
    #[deku(id = "0x05")]
    BusVolume(f32),
    #[deku(id = "0x06")]
    MakeUpGain(f32),
    #[deku(id = "0x07")]
    Priority(f32),
    #[deku(id = "0x08")]
    PriorityDistanceOffset(f32),
    #[deku(id = "0x09")]
    FeedbackVolume(f32),
    #[deku(id = "0x0A")]
    FeedbackLPF(f32),
    #[deku(id = "0x0B")]
    MuteRatio(f32),
    #[deku(id = "0x0C")]
    PanLR(f32),
    #[deku(id = "0x0D")]
    PanFR(f32),
    #[deku(id = "0x0E")]
    CenterPCT(f32),
    #[deku(id = "0x0F")]
    DelayTime(i32),
    #[deku(id = "0x10")]
    TransitionTime(u32),
    #[deku(id = "0x11")]
    Probability(f32),
    #[deku(id = "0x12")]
    DialogueMode(f32),
    #[deku(id = "0x13")]
    UserAuxSendVolume0(f32),
    #[deku(id = "0x14")]
    UserAuxSendVolume1(f32),
    #[deku(id = "0x15")]
    UserAuxSendVolume2(f32),
    #[deku(id = "0x16")]
    UserAuxSendVolume3(f32),
    #[deku(id = "0x17")]
    GameAuxSendVolume(f32),
    #[deku(id = "0x18")]
    OutputBusVolume(f32),
    #[deku(id = "0x19")]
    OutputBusHPF(f32),
    #[deku(id = "0x1A")]
    OutputBusLPF(f32),
    #[deku(id = "0x1B")]
    HDRBusThreshold(f32),
    #[deku(id = "0x1C")]
    HDRBusRatio(f32),
    #[deku(id = "0x1D")]
    HDRBusReleaseTime(f32),
    #[deku(id = "0x1E")]
    HDRBusGameParam(f32),
    #[deku(id = "0x1F")]
    HDRBusGameParamMin(f32),
    #[deku(id = "0x20")]
    HDRBusGameParamMax(f32),
    #[deku(id = "0x21")]
    HDRActiveRange(f32),
    #[deku(id = "0x22")]
    LoopStart(f32),
    #[deku(id = "0x23")]
    LoopEnd(f32),
    #[deku(id = "0x24")]
    TrimInTime(f32),
    #[deku(id = "0x25")]
    TrimOutTime(f32),
    #[deku(id = "0x26")]
    FadeInTime(f32),
    #[deku(id = "0x27")]
    FadeOutTime(f32),
    #[deku(id = "0x28")]
    FadeInCurve(f32),
    #[deku(id = "0x29")]
    FadeOutCurve(f32),
    #[deku(id = "0x2A")]
    LoopCrossfadeDuration(f32),
    #[deku(id = "0x2B")]
    CrossfadeUpCurve(f32),
    #[deku(id = "0x2C")]
    CrossfadeDownCurve(f32),
    #[deku(id = "0x2D")]
    MidiTrackingRootNote(f32),
    #[deku(id = "0x2E")]
    MidiPlayOnNoteType(f32),
    #[deku(id = "0x2F")]
    MidiTransposition(f32),
    #[deku(id = "0x30")]
    MidiVelocityOffset(f32),
    #[deku(id = "0x31")]
    MidiKeyRangeMin(f32),
    #[deku(id = "0x32")]
    MidiKeyRangeMax(f32),
    #[deku(id = "0x33")]
    MidiVelocityRangeMin(f32),
    #[deku(id = "0x34")]
    MidiVelocityRangeMax(f32),
    #[deku(id = "0x35")]
    MidiChannelMask(f32),
    #[deku(id = "0x36")]
    PlaybackSpeed(f32),
    #[deku(id = "0x37")]
    MidiTempoSource(f32),
    #[deku(id = "0x38")]
    MidiTargetNode(f32),
    #[deku(id = "0x39")]
    AttachedPluginFXID(u32),
    #[deku(id = "0x3A")]
    Loop(f32),
    #[deku(id = "0x3B")]
    InitialDelay(f32),
    #[deku(id = "0x3C")]
    UserAuxSendLPF0(f32),
    #[deku(id = "0x3D")]
    UserAuxSendLPF1(f32),
    #[deku(id = "0x3E")]
    UserAuxSendLPF2(f32),
    #[deku(id = "0x3F")]
    UserAuxSendLPF3(f32),
    #[deku(id = "0x40")]
    UserAuxSendHPF0(f32),
    #[deku(id = "0x41")]
    UserAuxSendHPF1(f32),
    #[deku(id = "0x42")]
    UserAuxSendHPF2(f32),
    #[deku(id = "0x43")]
    UserAuxSendHPF3(f32),
    #[deku(id = "0x44")]
    GameAuxSendLPF(f32),
    #[deku(id = "0x45")]
    GameAuxSendHPF(f32),
    #[deku(id = "0x46")]
    AttenuationID(u32),
    #[deku(id = "0x47")]
    PositioningTypeBlend(f32),
    #[deku(id = "0x48")]
    ReflectionBusVolume(f32),
}

impl PropBundle {
    fn read_list(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, Vec<Self>), DekuError> {
        let (mut rest, count) = u8::read(rest, ())?;

        let mut prop_ids = vec![];
        for _ in 0..count {
            let current_type: u8;
            (rest, current_type) = u8::read(rest, ())?;
            prop_ids.push(current_type);
        }

        let mut results = vec![];
        for prop_id in prop_ids.iter() {
            let current_value: Self;
            (rest, current_value) = Self::read_by_id(*prop_id, rest)?;
            results.push(current_value);
        }

        Ok((rest, results))
    }

    fn write_list(output: &mut BitVec<u8, Msb0>, values: &[&Self]) -> Result<(), DekuError> {
        u8::write(&(values.len() as u8), output, ())?;

        for value in values {
            u8::write(&(value.deku_id()?), output, ())?;
        }

        for value in values {
            value.write_internal(output)?;
        }

        Ok(())
    }

    fn read_by_id(
        prop_id: u8,
        rest: &BitSlice<u8, Msb0>,
    ) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError> {
        match prop_id {
            0x00 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::Volume(v)))
            }
            0x01 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::LFE(v)))
            }
            0x02 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::Pitch(v)))
            }
            0x03 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::LPF(v)))
            }
            0x04 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HPF(v)))
            }
            0x05 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::BusVolume(v)))
            }
            0x06 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MakeUpGain(v)))
            }
            0x07 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::Priority(v)))
            }
            0x08 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::PriorityDistanceOffset(v)))
            }
            0x09 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::FeedbackVolume(v)))
            }
            0x0A => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::FeedbackLPF(v)))
            }
            0x0B => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MuteRatio(v)))
            }
            0x0C => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::PanLR(v)))
            }
            0x0D => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::PanFR(v)))
            }
            0x0E => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::CenterPCT(v)))
            }
            0x0F => {
                let (r, v) = i32::read(rest, ())?;
                Ok((r, Self::DelayTime(v)))
            }
            0x10 => {
                let (r, v) = u32::read(rest, ())?;
                Ok((r, Self::TransitionTime(v)))
            }
            0x11 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::Probability(v)))
            }
            0x12 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::DialogueMode(v)))
            }
            0x13 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendVolume0(v)))
            }
            0x14 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendVolume1(v)))
            }
            0x15 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendVolume2(v)))
            }
            0x16 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendVolume3(v)))
            }
            0x17 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::GameAuxSendVolume(v)))
            }
            0x18 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::OutputBusVolume(v)))
            }
            0x19 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::OutputBusHPF(v)))
            }
            0x1A => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::OutputBusLPF(v)))
            }
            0x1B => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRBusThreshold(v)))
            }
            0x1C => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRBusRatio(v)))
            }
            0x1D => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRBusReleaseTime(v)))
            }
            0x1E => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRBusGameParam(v)))
            }
            0x1F => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRBusGameParamMin(v)))
            }
            0x20 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRBusGameParamMax(v)))
            }
            0x21 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::HDRActiveRange(v)))
            }
            0x22 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::LoopStart(v)))
            }
            0x23 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::LoopEnd(v)))
            }
            0x24 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::TrimInTime(v)))
            }
            0x25 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::TrimOutTime(v)))
            }
            0x26 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::FadeInTime(v)))
            }
            0x27 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::FadeOutTime(v)))
            }
            0x28 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::FadeInCurve(v)))
            }
            0x29 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::FadeOutCurve(v)))
            }
            0x2A => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::LoopCrossfadeDuration(v)))
            }
            0x2B => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::CrossfadeUpCurve(v)))
            }
            0x2C => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::CrossfadeDownCurve(v)))
            }
            0x2D => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiTrackingRootNote(v)))
            }
            0x2E => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiPlayOnNoteType(v)))
            }
            0x2F => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiTransposition(v)))
            }
            0x30 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiVelocityOffset(v)))
            }
            0x31 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiKeyRangeMin(v)))
            }
            0x32 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiKeyRangeMax(v)))
            }
            0x33 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiVelocityRangeMin(v)))
            }
            0x34 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiVelocityRangeMax(v)))
            }
            0x35 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiChannelMask(v)))
            }
            0x36 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::PlaybackSpeed(v)))
            }
            0x37 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiTempoSource(v)))
            }
            0x38 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::MidiTargetNode(v)))
            }
            0x39 => {
                let (r, v) = u32::read(rest, ())?;
                Ok((r, Self::AttachedPluginFXID(v)))
            }
            0x3A => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::Loop(v)))
            }
            0x3B => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::InitialDelay(v)))
            }
            0x3C => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendLPF0(v)))
            }
            0x3D => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendLPF1(v)))
            }
            0x3E => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendLPF2(v)))
            }
            0x3F => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendLPF3(v)))
            }
            0x40 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendHPF0(v)))
            }
            0x41 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendHPF1(v)))
            }
            0x42 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendHPF2(v)))
            }
            0x43 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::UserAuxSendHPF3(v)))
            }
            0x44 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::GameAuxSendLPF(v)))
            }
            0x45 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::GameAuxSendHPF(v)))
            }
            0x46 => {
                let (r, v) = u32::read(rest, ())?;
                Ok((r, Self::AttenuationID(v)))
            }
            0x47 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::PositioningTypeBlend(v)))
            }
            0x48 => {
                let (r, v) = f32::read(rest, ())?;
                Ok((r, Self::ReflectionBusVolume(v)))
            }
            _ => panic!("Unknown prop ID: {}", prop_id),
        }
    }

    fn write_internal(&self, output: &mut BitVec<u8, Msb0>) -> Result<(), DekuError> {
        match self {
            Self::Volume(v) => v.write(output, ())?,
            Self::LFE(v) => v.write(output, ())?,
            Self::Pitch(v) => v.write(output, ())?,
            Self::LPF(v) => v.write(output, ())?,
            Self::HPF(v) => v.write(output, ())?,
            Self::BusVolume(v) => v.write(output, ())?,
            Self::MakeUpGain(v) => v.write(output, ())?,
            Self::Priority(v) => v.write(output, ())?,
            Self::PriorityDistanceOffset(v) => v.write(output, ())?,
            Self::FeedbackVolume(v) => v.write(output, ())?,
            Self::FeedbackLPF(v) => v.write(output, ())?,
            Self::MuteRatio(v) => v.write(output, ())?,
            Self::PanLR(v) => v.write(output, ())?,
            Self::PanFR(v) => v.write(output, ())?,
            Self::CenterPCT(v) => v.write(output, ())?,
            Self::DelayTime(v) => v.write(output, ())?,
            Self::TransitionTime(v) => v.write(output, ())?,
            Self::Probability(v) => v.write(output, ())?,
            Self::DialogueMode(v) => v.write(output, ())?,
            Self::UserAuxSendVolume0(v) => v.write(output, ())?,
            Self::UserAuxSendVolume1(v) => v.write(output, ())?,
            Self::UserAuxSendVolume2(v) => v.write(output, ())?,
            Self::UserAuxSendVolume3(v) => v.write(output, ())?,
            Self::GameAuxSendVolume(v) => v.write(output, ())?,
            Self::OutputBusVolume(v) => v.write(output, ())?,
            Self::OutputBusHPF(v) => v.write(output, ())?,
            Self::OutputBusLPF(v) => v.write(output, ())?,
            Self::HDRBusThreshold(v) => v.write(output, ())?,
            Self::HDRBusRatio(v) => v.write(output, ())?,
            Self::HDRBusReleaseTime(v) => v.write(output, ())?,
            Self::HDRBusGameParam(v) => v.write(output, ())?,
            Self::HDRBusGameParamMin(v) => v.write(output, ())?,
            Self::HDRBusGameParamMax(v) => v.write(output, ())?,
            Self::HDRActiveRange(v) => v.write(output, ())?,
            Self::LoopStart(v) => v.write(output, ())?,
            Self::LoopEnd(v) => v.write(output, ())?,
            Self::TrimInTime(v) => v.write(output, ())?,
            Self::TrimOutTime(v) => v.write(output, ())?,
            Self::FadeInTime(v) => v.write(output, ())?,
            Self::FadeOutTime(v) => v.write(output, ())?,
            Self::FadeInCurve(v) => v.write(output, ())?,
            Self::FadeOutCurve(v) => v.write(output, ())?,
            Self::LoopCrossfadeDuration(v) => v.write(output, ())?,
            Self::CrossfadeUpCurve(v) => v.write(output, ())?,
            Self::CrossfadeDownCurve(v) => v.write(output, ())?,
            Self::MidiTrackingRootNote(v) => v.write(output, ())?,
            Self::MidiPlayOnNoteType(v) => v.write(output, ())?,
            Self::MidiTransposition(v) => v.write(output, ())?,
            Self::MidiVelocityOffset(v) => v.write(output, ())?,
            Self::MidiKeyRangeMin(v) => v.write(output, ())?,
            Self::MidiKeyRangeMax(v) => v.write(output, ())?,
            Self::MidiVelocityRangeMin(v) => v.write(output, ())?,
            Self::MidiVelocityRangeMax(v) => v.write(output, ())?,
            Self::MidiChannelMask(v) => v.write(output, ())?,
            Self::PlaybackSpeed(v) => v.write(output, ())?,
            Self::MidiTempoSource(v) => v.write(output, ())?,
            Self::MidiTargetNode(v) => v.write(output, ())?,
            Self::AttachedPluginFXID(v) => v.write(output, ())?,
            Self::Loop(v) => v.write(output, ())?,
            Self::InitialDelay(v) => v.write(output, ())?,
            Self::UserAuxSendLPF0(v) => v.write(output, ())?,
            Self::UserAuxSendLPF1(v) => v.write(output, ())?,
            Self::UserAuxSendLPF2(v) => v.write(output, ())?,
            Self::UserAuxSendLPF3(v) => v.write(output, ())?,
            Self::UserAuxSendHPF0(v) => v.write(output, ())?,
            Self::UserAuxSendHPF1(v) => v.write(output, ())?,
            Self::UserAuxSendHPF2(v) => v.write(output, ())?,
            Self::UserAuxSendHPF3(v) => v.write(output, ())?,
            Self::GameAuxSendLPF(v) => v.write(output, ())?,
            Self::GameAuxSendHPF(v) => v.write(output, ())?,
            Self::AttenuationID(v) => v.write(output, ())?,
            Self::PositioningTypeBlend(v) => v.write(output, ())?,
            Self::ReflectionBusVolume(v) => v.write(output, ())?,
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct PropRangedModifiers {
    #[deku(update = "self.entries.len()")]
    count: u8,
    #[deku(count = "count")]
    pub entries: Vec<PropRangedModifier>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct PropRangedModifier {
    pub prop_type: u8,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct PositioningParams {
    #[deku(bits = "1")]
    unk1: bool,
    pub three_dimensional_position_type: Ak3DPositionType,
    pub speaker_panning_type: AkSpeakerPanningType,
    #[deku(bits = "1")]
    pub listener_relative_routing: bool,
    #[deku(bits = "1")]
    pub override_parent: bool,
    #[deku(bits = "1", skip, cond = "!*listener_relative_routing")]
    unk2: bool,
    #[deku(bits = "1", skip, cond = "!*listener_relative_routing")]
    pub enable_diffraction: bool,
    #[deku(bits = "1", skip, cond = "!*listener_relative_routing")]
    pub hold_listener_orientation: bool,
    #[deku(bits = "1", skip, cond = "!*listener_relative_routing")]
    pub hold_emitter_position_and_orientation: bool,
    #[deku(bits = "1", skip, cond = "!*listener_relative_routing")]
    pub enable_attenuation: bool,
    #[deku(skip, cond = "!*listener_relative_routing")]
    pub three_dimensional_spatialization_mode: Ak3DSpatializationMode,
    #[deku(
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    pub path_mode: AkPathMode,
    #[deku(
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    pub transition_time: i32,
    #[deku(
        update = "self.vertices.len()",
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    vertex_count: u32,
    #[deku(
        count = "vertex_count",
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    pub vertices: Vec<AkPathVertex>,

    #[deku(
        update = "self.path_list_item_offsets.len()",
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    path_list_item_count: u32,
    #[deku(
        count = "path_list_item_count",
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    pub path_list_item_offsets: Vec<AkPathListItemOffset>,
    #[deku(
        count = "path_list_item_count",
        skip,
        cond = "*three_dimensional_position_type == Ak3DPositionType::Emitter"
    )]
    pub three_dimensional_automation_params: Vec<Ak3DAutomationParams>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkPathVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub duration: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkPathListItemOffset {
    pub vertices_offset: u32,
    pub vertices_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct Ak3DAutomationParams {
    pub range_x: f32,
    pub range_y: f32,
    pub range_z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AuxParams {
    #[deku(bits = 1)]
    unk1: bool,
    #[deku(bits = 1)]
    unk2: bool,
    #[deku(bits = 1)]
    unk3: bool,
    #[deku(bits = 1)]
    pub override_reflections_aux_bus: bool,
    #[deku(bits = 1)]
    pub has_aux: bool,
    #[deku(bits = 1)]
    pub override_user_aux_sends: bool,
    #[deku(bits = 2)]
    unk4: u8,
    #[deku(skip, cond = "!*has_aux")]
    pub aux1: u32,
    #[deku(skip, cond = "!*has_aux")]
    pub aux2: u32,
    #[deku(skip, cond = "!*has_aux")]
    pub aux3: u32,
    #[deku(skip, cond = "!*has_aux")]
    pub aux4: u32,
    pub reflections_aux_bus: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AdvSettingsParams {
    #[deku(bits = "1")]
    pub unk1: bool,
    #[deku(bits = "1")]
    pub unk2: bool,
    #[deku(bits = "1")]
    pub unk3: bool,
    #[deku(bits = "1")]
    pub is_virtual_voices_opt_override_parent: bool,
    #[deku(bits = "1")]
    pub ignore_parent_maximum_instances: bool,
    #[deku(bits = "1")]
    pub unk4: bool,
    #[deku(bits = "1")]
    pub use_virtual_behavior: bool,
    #[deku(bits = "1")]
    pub kill_newest: bool,
    pub virtual_queue_behavior: AkVirtualQueueBehavior,
    pub max_instance_count: u16,
    pub below_threshold_behavior: AkBelowThresholdBehavior,
    #[deku(bits = "1")]
    pub unk5: bool,
    #[deku(bits = "1")]
    pub unk6: bool,
    #[deku(bits = "1")]
    pub unk7: bool,
    #[deku(bits = "1")]
    pub unk8: bool,
    #[deku(bits = 1)]
    pub enable_envelope: bool,
    #[deku(bits = 1)]
    pub normalize_loudness: bool,
    #[deku(bits = 1)]
    pub override_analysis: bool,
    #[deku(bits = 1)]
    pub override_hdr_envelope: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct StateChunk {
    #[deku(update = "self.state_property_info.len()")]
    state_property_count: u8,
    #[deku(count = "state_property_count")]
    pub state_property_info: Vec<AkStatePropertyInfo>,
    #[deku(update = "self.state_group_chunks.len()")]
    state_group_count: u8,
    #[deku(count = "state_group_count")]
    pub state_group_chunks: Vec<AkStateGroupChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkStatePropertyInfo {
    pub property: AkPropID,
    pub accum_type: AkRtpcAccum,
    pub in_db: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkStateGroupChunk {
    pub state_group_id: u32,
    pub sync_type: AkSyncTypeU8,
    #[deku(update = "self.states.len()")]
    state_count: u8,
    #[deku(count = "state_count")]
    pub states: Vec<AkState>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct AkState {
    pub state_id: u32,
    pub state_instance_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct InitialRTPC {
    #[deku(update = "self.rtpcs.len()")]
    count: u16,
    #[deku(count = "count")]
    pub rtpcs: Vec<RTPC>,
}

#[derive(Debug, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
pub struct RTPC {
    pub id: u32,
    pub rtpc_type: AkRtpcType,
    pub rtpc_accum: AkRtpcAccum,
    pub param_id: u8,
    pub curve_id: u32,
    pub curve_scaling: AkCurveScaling,
    #[deku(update = "self.graph_points.len()")]
    graph_point_count: u16,
    #[deku(count = "graph_point_count")]
    pub graph_points: Vec<AkRTPCGraphPoint>,
}
