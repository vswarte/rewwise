use crate::*;
use deku::bitvec::BitVec;

#[derive(Debug)]
pub enum PrepareExportError {
    Deku(deku::DekuError)
}

/// Trait that applies some additional logic to the soundbank to prepare it for
/// export/encoding. This includes things like:
/// - Determining the required BKHD padding
/// - Updating any size or length fields
pub trait PrepareExport {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError>;
}

impl PrepareExport for Soundbank {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        // Prepare BKHD padding if there is a DATA section
        let descriptor_count_result = self.sections.iter()
            .find_map(|s| match &s.body {
                SectionBody::DIDX(d) => Some(d.descriptors.len()),
                _ => None,
            });

        if let Some(descriptor_count) = descriptor_count_result {
            let bkhd = self.sections.iter_mut()
                .find_map(|s| match &mut s.body {
                    SectionBody::BKHD(b) => Some(b),
                    _ => None,
                })
                .expect("Can not create a soundbank without BKHD section");

            // Calculate the offset in the file to the first WEM
            // This consists of the BKHD header, the DIDX header, the DATA
            // header, the BKHD content and the DIDX contents.
            // Example build-up:
            // +-------------------+
            // |  BKHD section     |
            // |                   |
            // +-------------------+
            // |   PADDING         | <-- This is used to align
            // +-------------------+
            // |   DIDX section    |
            // |                   |
            // |  +-------------+  |
            // |  |  WEM desc   |  |
            // |  +-------------+  |
            // |  |  WEM desc   |  |
            // |  +-------------+  |
            // |  |  WEM desc   |  |
            // |  +-------------+  |
            // +-------------------+
            // |   DATA section    |
            // |                   |
            // |  +-------------+  | <-- This needs to be aligned
            // |  |  WEM file   |  |
            // |  +-------------+  |
            // |  |  WEM file   |  |
            // |  +-------------+  |
            // |  |  WEM file   |  |
            // |  +-------------+  |
            // +-------------------+
            let first_wem_offset = (
                // Account for the three section headers
                8 * 3 + 
                // Account for the unpadded BKHD contents
                0x14 +
                // Account for the DIDX WEM descriptors
                (descriptor_count * 0xC)
            ) as u32;

            let padding_size = {
                if first_wem_offset % bkhd.wem_alignment == 0x0 {
                    // Do nothing if first WEM already aligns
                    0x0
                } else {
                    // Calculate remaining bytes
                    (
                        first_wem_offset + (
                            bkhd.wem_alignment - first_wem_offset % bkhd.wem_alignment
                        )
                    ) - first_wem_offset
                }
            };

            bkhd.padding = vec![0u8; padding_size as usize];
        }

        for section in self.sections.iter_mut() {
            section.prepare_export()?;
        }

        Ok(())
    }
}

impl PrepareExport for Section {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        match &mut self.body {
            SectionBody::BKHD(s) => de(s.update()),
            SectionBody::DIDX(s) => de(s.update()),
            SectionBody::DATA(s) => de(s.update()),
            SectionBody::ENVS(s) => s.prepare_export(),
            SectionBody::FXPR(s) => de(s.update()),
            SectionBody::HIRC(s) => s.prepare_export(),
            SectionBody::STID(s) => s.prepare_export(),
            SectionBody::STMG(s) => s.prepare_export(),
            SectionBody::INIT(s) => s.prepare_export(),
            SectionBody::PLAT(s) => de(s.update()),
        }?;

        self.size = sample_section_body_size(self)
            .map_err(PrepareExportError::Deku)?;

        self.update().map_err(PrepareExportError::Deku)?;

        Ok(())
    }
}

impl PrepareExport for STIDSection {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for entry in self.entries.iter_mut() {
            entry.update().map_err(PrepareExportError::Deku)?
        }

        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for STMGSection {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for state_group in self.state_groups.iter_mut() {
            state_group.update().map_err(PrepareExportError::Deku)?
        }

        for switch_group in self.switch_groups.iter_mut() {
            switch_group.update().map_err(PrepareExportError::Deku)?
        }

        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for INITSection {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for plugin in self.plugins.iter_mut() {
            plugin.update().map_err(PrepareExportError::Deku)?
        }

        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

fn sample_section_body_size(s: &Section) -> Result<u32, deku::DekuError> {
    // Encode the body once
    let mut buffer = BitVec::default();
    s.body.write(&mut buffer, (s.magic, 0x100))?;

    // Get the encoded body length and add the header size
    Ok(buffer.as_raw_slice().len() as u32)
}

impl PrepareExport for HIRCSection {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for object in self.objects.iter_mut() {
            object.prepare_export()?;
        }
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for HIRCObject {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        match &mut self.body {
            HIRCObjectBody::State(o) => de(o.update()),
            HIRCObjectBody::Sound(o) => o.prepare_export(),
            HIRCObjectBody::Action(o) => o.prepare_export(),
            HIRCObjectBody::Event(o) => de(o.update()),
            HIRCObjectBody::RandomSequenceContainer(o) => o.prepare_export(),
            HIRCObjectBody::SwitchContainer(o) => o.prepare_export(),
            HIRCObjectBody::ActorMixer(o) => o.prepare_export(),
            HIRCObjectBody::Bus(o) => o.prepare_export(),
            HIRCObjectBody::LayerContainer(o) => o.prepare_export(),
            HIRCObjectBody::MusicSegment(o) => o.prepare_export(),
            HIRCObjectBody::MusicTrack(o) => o.prepare_export(),
            HIRCObjectBody::MusicSwitchContainer(o) => o.prepare_export(),
            HIRCObjectBody::MusicRandomSequenceContainer(o) => o.prepare_export(),
            HIRCObjectBody::Attenuation(o) => o.prepare_export(),
            HIRCObjectBody::DialogueEvent(o) => o.prepare_export(),
            HIRCObjectBody::EffectShareSet(o) => o.prepare_export(),
            HIRCObjectBody::EffectCustom(o) => o.prepare_export(),
            HIRCObjectBody::AuxiliaryBus(o) => o.prepare_export(),
            HIRCObjectBody::LFOModulator(o) => de(o.update()),
            HIRCObjectBody::EnvelopeModulator(o) => de(o.update()),
            HIRCObjectBody::AudioDevice(o) => o.prepare_export(),
            HIRCObjectBody::TimeModulator(o) => o.prepare_export(),
        }?;

        self.size = sample_hirc_body_size(self)
            .map_err(PrepareExportError::Deku)?;

        self.update().map_err(PrepareExportError::Deku)?;

        Ok(())
    }
}

fn sample_hirc_body_size(s: &mut HIRCObject) -> Result<u32, deku::DekuError> {
    // Encode the body once
    let mut buffer = BitVec::default();
    s.body.write(&mut buffer, (s.body_type, 0x100))?;

    // Get the encoded body length and add the header size
    Ok(buffer.as_raw_slice().len() as u32 + 4)
}

impl PrepareExport for CAkSound {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_base_params.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)?;

        Ok(())
    }
}

impl PrepareExport for NodeBaseParams {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_initial_fx_parameters.update()
            .map_err(PrepareExportError::Deku)?;
        self.node_initial_params.prepare_export()?;
        self.positioning_params.update()
            .map_err(PrepareExportError::Deku)?;
        self.aux_params.update()
            .map_err(PrepareExportError::Deku)?;
        self.adv_settings_params.update()
            .map_err(PrepareExportError::Deku)?;
        self.state_chunk.prepare_export()?;
        self.initial_rtpc.prepare_export()?;

        // TODO: rest

        self.update().map_err(PrepareExportError::Deku)?;

        Ok(())
    }
}

impl PrepareExport for NodeInitialParams {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.prop_initial_values.update()
            .map_err(PrepareExportError::Deku)?;
        self.prop_ranged_modifiers.update()
            .map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for StateChunk {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for state_group_chunk in self.state_group_chunks.iter_mut() {
            state_group_chunk.update()
                .map_err(PrepareExportError::Deku)?;
        }

        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for InitialRTPC {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for rtpc in self.rtpcs.iter_mut() {
            rtpc.update().map_err(PrepareExportError::Deku)?;
        }

        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkAction {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.prop_bundle.update().map_err(PrepareExportError::Deku)?;
        self.ranged_modifiers.update().map_err(PrepareExportError::Deku)?;
        self.params.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkActionParams {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        match self {
            CAkActionParams::SetState(p) => de(p.update()),
            CAkActionParams::SetSwitch(p) => de(p.update()),
            CAkActionParams::Play(p) => de(p.update()),
            CAkActionParams::StopE(p) => p.prepare_export(),
            CAkActionParams::StopEO(p) => p.prepare_export(),
            CAkActionParams::MuteM(p) => p.prepare_export(),
            CAkActionParams::MuteO(p) => p.prepare_export(),
            CAkActionParams::UnmuteM(p) => p.prepare_export(),
            CAkActionParams::UnmuteO(p) => p.prepare_export(),
            CAkActionParams::UnmuteALL(p) => p.prepare_export(),
            CAkActionParams::UnmuteALLO(p) => p.prepare_export(),
            CAkActionParams::UnmuteAE(p) => p.prepare_export(),
            CAkActionParams::UnmuteAEO(p) => p.prepare_export(),
            CAkActionParams::SetVolumeM(p) => p.prepare_export(),
            CAkActionParams::SetVolumeO(p) => p.prepare_export(),
            CAkActionParams::ResetVolumeM(p) => p.prepare_export(),
            CAkActionParams::ResetVolumeO(p) => p.prepare_export(),
            CAkActionParams::PlayEvent => { Ok(()) },
        }?;

        Ok(())
    }
}

impl PrepareExport for CAkActionStop {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.stop.update().map_err(PrepareExportError::Deku)?;
        self.except.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkActionMute {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.except.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkActionSetAkProp {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.set_ak_prop.update().map_err(PrepareExportError::Deku)?;
        self.except.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkRanSeqCntr {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_base_params.prepare_export()?;
        self.children.update().map_err(PrepareExportError::Deku)?;
        self.playlist.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkSwitchCntr {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_base_params.prepare_export()?;
        self.children.update().map_err(PrepareExportError::Deku)?;
        for switch_group in self.switch_groups.iter_mut() {
            switch_group.update().map_err(PrepareExportError::Deku)?;
        }
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkActorMixer {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_base_params.prepare_export()?;
        self.children.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkBus {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.initial_values.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for BusInitialValues {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.bus_initial_params.prepare_export()?;
        self.bus_initial_fx_params.update().map_err(PrepareExportError::Deku)?;
        self.initial_rtpc.prepare_export()?;
        self.state_chunk.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for BusInitialParams {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.prop_bundle.update().map_err(PrepareExportError::Deku)?;
        self.positioning_params.update().map_err(PrepareExportError::Deku)?;
        self.aux_params.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkLayerCntr {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_base_params.prepare_export()?;
        self.children.update().map_err(PrepareExportError::Deku)?;
        for layer in self.layers.iter_mut() {
            layer.prepare_export()?;
        }
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkLayer {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.initial_rtpc.prepare_export()?;
        for child in self.associated_children.iter_mut() {
            child.update().map_err(PrepareExportError::Deku)?;
        }
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkMusicSegment {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.music_node_params.prepare_export()?;
        for marker in self.markers.iter_mut() {
            marker.update().map_err(PrepareExportError::Deku)?;
        }
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for MusicNodeParams {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.node_base_params.prepare_export()?;
        self.children.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkMusicTrack {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for source in self.sources.iter_mut() {
            source.update().map_err(PrepareExportError::Deku)?;
        }
        for clip in self.clip_items.iter_mut() {
            clip.update().map_err(PrepareExportError::Deku)?;
        }
        self.node_base_params.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

impl PrepareExport for CAkMusicSwitchCntr {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.music_trans_node_params.prepare_export()?;
        for node in self.tree.iter_mut() {
            node.prepare_export()?;
        }
        self.tree_size = sample_tree_size(&self.tree)
            .map_err(PrepareExportError::Deku)? as u32;

        self.update().map_err(PrepareExportError::Deku)
    }
}

fn sample_tree_size(s: &[AkDecisionTreeNode]) -> Result<usize, deku::DekuError> {
    let mut buffer = BitVec::default();
    AkDecisionTreeNode::write(&mut buffer, &s.iter().collect::<Vec<_>>())?;

    // Get the encoded body length and add the header size
    Ok(buffer.as_raw_slice().len())
}

impl PrepareExport for MusicTransNodeParams {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.music_node_params.prepare_export()?;
        for rule in self.transition_rules.iter_mut() {
            rule.update().map_err(PrepareExportError::Deku)?;
        }
        self.update().map_err(PrepareExportError::Deku)
    }
}

impl PrepareExport for AkDecisionTreeNode {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.child_count = self.children.len() as u16; 
        for child in self.children.iter_mut() {
            child.prepare_export()?;
        }
        Ok(())
    }
}

impl PrepareExport for CAkMusicRanSeqCntr {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.music_trans_node_params.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)
    }
}

impl PrepareExport for CAkAttentuation {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for curve in self.curves.iter_mut() {
            curve.update().map_err(PrepareExportError::Deku)?;
        }
        self.initial_rtpc.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)
    }
}

impl PrepareExport for CAkDialogueEvent {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        for node in self.tree.iter_mut() {
            node.prepare_export()?;
        }
        self.prop_bundle.update().map_err(PrepareExportError::Deku)?;
        self.ranged_modifiers.update().map_err(PrepareExportError::Deku)?;
        self.update().map_err(PrepareExportError::Deku)
    }
}

impl PrepareExport for CAkFxShareSet {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.fx_base_initial_values.prepare_export()
    }
}

impl PrepareExport for FxBaseInitialValues {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.initial_rtpc.prepare_export()?;
        self.state_chunk.prepare_export()?;
        self.update().map_err(PrepareExportError::Deku)
    }
}

impl PrepareExport for CAkFxCustom {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.fx_base_initial_values.prepare_export()
    }
}

impl PrepareExport for CAkAuxBus {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.initial_values.prepare_export()
    }
}

impl PrepareExport for CAkAudioDevice {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.fx_base_initial_values.prepare_export()
    }
}

impl PrepareExport for CAkTimeModulator {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.prop_bundle.update().map_err(PrepareExportError::Deku)?;
        self.ranged_modifiers.update().map_err(PrepareExportError::Deku)?;
        self.initial_rtpc.prepare_export()?;
        Ok(())
    }
}

impl PrepareExport for ENVSSection {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.conversion_table.prepare_export()
    }
}

impl PrepareExport for ConversionTable {
    fn prepare_export(&mut self) -> Result<(), PrepareExportError> {
        self.curve_obs_vol.update().map_err(PrepareExportError::Deku)?;
        self.curve_obs_lpf.update().map_err(PrepareExportError::Deku)?;
        self.curve_obs_hpf.update().map_err(PrepareExportError::Deku)?;
        self.curve_occ_vol.update().map_err(PrepareExportError::Deku)?;
        self.curve_occ_lpf.update().map_err(PrepareExportError::Deku)?;
        self.curve_occ_hpf.update().map_err(PrepareExportError::Deku)?;
        Ok(())
    }
}

fn de(input: Result<(), deku::DekuError>) -> Result<(), PrepareExportError> {
    input.map_err(PrepareExportError::Deku)
}
