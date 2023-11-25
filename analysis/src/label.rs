use wwise_format::*;

use crate::dictionary::*;

pub fn get_label(
    a: &HIRCObject,
    dictionary: Option<&FNVDictionary>
) -> String {
    let name = dictionary.as_ref()
        .and_then(|d| d.get(&a.id));

    match name {
        Some(name) => format!("{}(\"{}\", {})", get_type_label(a), name, a.id),
        None => format!("{}({})", get_type_label(a), a.id),
    }
}

fn get_type_label(a: &HIRCObject) -> &'static str {
    match a.body {
        HIRCObjectBody::State(_) => "State",
        HIRCObjectBody::Sound(_) => "Sound",
        HIRCObjectBody::Action(_) => "Action",
        HIRCObjectBody::Event(_) => "Event",
        HIRCObjectBody::RandomSequenceContainer(_) => "RandomSequenceContainer",
        HIRCObjectBody::SwitchContainer(_) => "SwitchContainer",
        HIRCObjectBody::ActorMixer(_) => "ActorMixer",
        HIRCObjectBody::Bus(_) => "Bus",
        HIRCObjectBody::LayerContainer(_) => "LayerContainer",
        HIRCObjectBody::MusicSegment(_) => "MusicSegment",
        HIRCObjectBody::MusicTrack(_) => "MusicTrack",
        HIRCObjectBody::MusicSwitchContainer(_) => "MusicSwitchContainer",
        HIRCObjectBody::MusicRandomSequenceContainer(_) => "MusicRandomSequenceContainer",
        HIRCObjectBody::Attenuation(_) => "Attenuation",
        HIRCObjectBody::DialogueEvent(_) => "DialogueEvent",
        HIRCObjectBody::EffectShareSet(_) => "EffectShareSet",
        HIRCObjectBody::EffectCustom(_) => "EffectCustom",
        HIRCObjectBody::AuxiliaryBus(_) => "AuxiliaryBus",
        HIRCObjectBody::LFOModulator(_) => "LFOModulator",
        HIRCObjectBody::EnvelopeModulator(_) => "EnvelopeModulator",
        HIRCObjectBody::AudioDevice(_) => "AudioDevice",
        HIRCObjectBody::TimeModulator(_) => "TimeModulator",
    }
}
