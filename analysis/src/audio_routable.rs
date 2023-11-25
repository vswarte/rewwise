use wwise_format::*;

trait AudioRoutable {
    fn outputs_to(&self) -> Vec<u32>;
}

pub fn get_output_nodes(a: &HIRCObject) -> Option<Vec<u32>> {
    Some(match &a.body {
        HIRCObjectBody::Sound(b)
            => b.outputs_to(),
        HIRCObjectBody::RandomSequenceContainer(b)
            => b.outputs_to(),
        HIRCObjectBody::SwitchContainer(b)
            => b.outputs_to(),
        HIRCObjectBody::ActorMixer(b)
            => b.outputs_to(),
        HIRCObjectBody::Bus(b)
            => b.outputs_to(),
        HIRCObjectBody::LayerContainer(b)
            => b.outputs_to(),
        HIRCObjectBody::MusicSegment(b)
            => b.outputs_to(),
        HIRCObjectBody::MusicTrack(b)
            => b.outputs_to(),
        HIRCObjectBody::MusicSwitchContainer(b)
            => b.outputs_to(),
        HIRCObjectBody::MusicRandomSequenceContainer(b)
            => b.outputs_to(),
        HIRCObjectBody::AuxiliaryBus(b)
            => b.outputs_to(),
        _ => return None,
    })
}

impl AudioRoutable for CAkSound {
    fn outputs_to(&self) -> Vec<u32> {
        if self.node_base_params.override_bus_id != 0 {
            vec![self.node_base_params.override_bus_id]
        } else {
            vec![self.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkRanSeqCntr {
    fn outputs_to(&self) -> Vec<u32> {
        if self.node_base_params.override_bus_id != 0 {
            vec![self.node_base_params.override_bus_id]
        } else {
            vec![self.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkSwitchCntr {
    fn outputs_to(&self) -> Vec<u32> {
        if self.node_base_params.override_bus_id != 0 {
            vec![self.node_base_params.override_bus_id]
        } else {
            vec![self.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkActorMixer {
    fn outputs_to(&self) -> Vec<u32> {
        if self.node_base_params.override_bus_id != 0 {
            vec![self.node_base_params.override_bus_id]
        } else {
            vec![self.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkBus {
    fn outputs_to(&self) -> Vec<u32> {
        if self.initial_values.override_bus_id != 0 {
            vec![self.initial_values.override_bus_id]
        } else {
            vec![]
        }
    }
}

impl AudioRoutable for CAkLayerCntr {
    fn outputs_to(&self) -> Vec<u32> {
        if self.node_base_params.override_bus_id != 0 {
            vec![self.node_base_params.override_bus_id]
        } else {
            vec![self.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkMusicSegment {
    fn outputs_to(&self) -> Vec<u32> {
        if self.music_node_params.node_base_params.override_bus_id != 0 {
            vec![self.music_node_params.node_base_params.override_bus_id]
        } else {
            vec![self.music_node_params.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkMusicTrack {
    fn outputs_to(&self) -> Vec<u32> {
        if self.node_base_params.override_bus_id != 0 {
            vec![self.node_base_params.override_bus_id]
        } else {
            vec![self.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkMusicSwitchCntr {
    fn outputs_to(&self) -> Vec<u32> {
        if self.music_trans_node_params.music_node_params.node_base_params.override_bus_id != 0 {
            vec![self.music_trans_node_params.music_node_params.node_base_params.override_bus_id]
        } else {
            vec![self.music_trans_node_params.music_node_params.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkMusicRanSeqCntr {
    fn outputs_to(&self) -> Vec<u32> {
        if self.music_trans_node_params.music_node_params.node_base_params.override_bus_id != 0 {
            vec![self.music_trans_node_params.music_node_params.node_base_params.override_bus_id]
        } else {
            vec![self.music_trans_node_params.music_node_params.node_base_params.direct_parent_id]
        }
    }
}

impl AudioRoutable for CAkAuxBus {
    fn outputs_to(&self) -> Vec<u32> {
        if self.initial_values.override_bus_id != 0 {
            vec![self.initial_values.override_bus_id]
        } else {
            vec![]
        }
    }
}
