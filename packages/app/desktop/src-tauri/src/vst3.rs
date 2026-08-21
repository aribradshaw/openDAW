use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Mutex};
use tauri::State;
use vst3_host::{
    AudioBusBuffer, AudioBusLayout, MidiChannel, MidiEvent, Parameter, Plugin, PluginInfo, Vst3Host,
};

type CommandResult<T> = Result<T, String>;

pub struct Vst3Service {
    host: Vst3Host,
    instances: HashMap<u64, Plugin>,
    next_instance_id: u64,
}

impl Default for Vst3Service {
    fn default() -> Self {
        Self {
            host: Vst3Host::builder()
                .sample_rate(48_000.0)
                .block_size(128)
                .build()
                .expect("VST3 host configuration must be valid"),
            instances: HashMap::new(),
            next_instance_id: 1,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vst3InstanceInfo {
    instance_id: u64,
    plugin: PluginInfo,
    parameters: Vec<Parameter>,
    audio_buses: AudioBusLayout,
    latency_samples: u32,
    tail_samples: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterChange {
    instance_id: u64,
    parameter_id: u32,
    value: f64,
    sample_offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MidiMessage {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    PitchBend {
        channel: u8,
        value: u16,
    },
    ChannelAftertouch {
        channel: u8,
        pressure: u8,
    },
    PolyAftertouch {
        channel: u8,
        note: u8,
        pressure: u8,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledMidiMessage {
    instance_id: u64,
    sample_offset: i32,
    message: MidiMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportUpdate {
    instance_id: u64,
    tempo: f64,
    numerator: i32,
    denominator: i32,
    playing: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioBusInput {
    active: bool,
    channels: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessBlockRequest {
    instance_id: u64,
    block_size: usize,
    inputs: Vec<AudioBusInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessBlockResponse {
    outputs: Vec<Vec<Vec<f32>>>,
    output_midi: Vec<MidiEvent>,
    latency_samples: u32,
    tail_samples: u32,
}

fn with_service<T>(
    state: &State<'_, Mutex<Vst3Service>>,
    operation: impl FnOnce(&mut Vst3Service) -> CommandResult<T>,
) -> CommandResult<T> {
    let mut service = state
        .lock()
        .map_err(|_| "VST3 host state is unavailable".to_string())?;
    operation(&mut service)
}

fn instance(service: &mut Vst3Service, id: u64) -> CommandResult<&mut Plugin> {
    service
        .instances
        .get_mut(&id)
        .ok_or_else(|| format!("VST3 instance {id} does not exist"))
}

fn midi_channel(channel: u8) -> CommandResult<MidiChannel> {
    if !(1..=16).contains(&channel) {
        return Err(format!("MIDI channel {channel} is outside 1..=16"));
    }
    MidiChannel::from_index(channel - 1).ok_or_else(|| format!("invalid MIDI channel {channel}"))
}

fn midi_event(message: MidiMessage) -> CommandResult<MidiEvent> {
    Ok(match message {
        MidiMessage::NoteOn {
            channel,
            note,
            velocity,
        } => MidiEvent::NoteOn {
            channel: midi_channel(channel)?,
            note,
            velocity,
        },
        MidiMessage::NoteOff {
            channel,
            note,
            velocity,
        } => MidiEvent::NoteOff {
            channel: midi_channel(channel)?,
            note,
            velocity,
        },
        MidiMessage::ControlChange {
            channel,
            controller,
            value,
        } => MidiEvent::ControlChange {
            channel: midi_channel(channel)?,
            controller,
            value,
        },
        MidiMessage::PitchBend { channel, value } => MidiEvent::PitchBend {
            channel: midi_channel(channel)?,
            value,
        },
        MidiMessage::ChannelAftertouch { channel, pressure } => MidiEvent::ChannelAftertouch {
            channel: midi_channel(channel)?,
            pressure,
        },
        MidiMessage::PolyAftertouch {
            channel,
            note,
            pressure,
        } => MidiEvent::PolyAftertouch {
            channel: midi_channel(channel)?,
            note,
            pressure,
        },
    })
}

#[tauri::command]
pub fn vst3_scan(state: State<'_, Mutex<Vst3Service>>) -> CommandResult<Vec<PluginInfo>> {
    with_service(&state, |service| {
        service
            .host
            .discover_plugins()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_load(
    path: String,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<Vst3InstanceInfo> {
    with_service(&state, |service| {
        let path = Path::new(&path)
            .canonicalize()
            .map_err(|error| format!("cannot resolve VST3 path: {error}"))?;
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("vst3"))
        {
            return Err("only VST3 plugins can be loaded".to_string());
        }

        let mut plugin = service
            .host
            .load_plugin(&path)
            .map_err(|error| error.to_string())?;
        let info = Vst3InstanceInfo {
            instance_id: service.next_instance_id,
            plugin: plugin.info().clone(),
            parameters: plugin.get_parameters().map_err(|error| error.to_string())?,
            audio_buses: plugin
                .audio_bus_layout()
                .map_err(|error| error.to_string())?,
            latency_samples: plugin.latency_samples(),
            tail_samples: plugin.tail_samples(),
        };
        plugin
            .start_processing()
            .map_err(|error| error.to_string())?;
        service.instances.insert(info.instance_id, plugin);
        service.next_instance_id = service.next_instance_id.saturating_add(1);
        Ok(info)
    })
}

#[tauri::command]
pub fn vst3_unload(instance_id: u64, state: State<'_, Mutex<Vst3Service>>) -> CommandResult<()> {
    with_service(&state, |service| {
        let mut plugin = service
            .instances
            .remove(&instance_id)
            .ok_or_else(|| format!("VST3 instance {instance_id} does not exist"))?;
        plugin.stop_processing().map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_set_parameter(
    change: ParameterChange,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<String> {
    with_service(&state, |service| {
        let plugin = instance(service, change.instance_id)?;
        if let Some(offset) = change.sample_offset {
            plugin
                .set_parameter_at(change.parameter_id, change.value, offset)
                .map_err(|error| error.to_string())?;
        } else {
            plugin
                .set_parameter(change.parameter_id, change.value)
                .map_err(|error| error.to_string())?;
        }
        plugin
            .format_parameter(change.parameter_id, change.value)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_save_state(
    instance_id: u64,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<Vec<u8>> {
    with_service(&state, |service| {
        instance(service, instance_id)?
            .save_state()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_load_state(
    instance_id: u64,
    plugin_state: Vec<u8>,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<()> {
    with_service(&state, |service| {
        instance(service, instance_id)?
            .load_state(&plugin_state)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_send_midi(
    event: ScheduledMidiMessage,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<()> {
    with_service(&state, |service| {
        let message = midi_event(event.message)?;
        instance(service, event.instance_id)?
            .send_midi_event_at(message, event.sample_offset)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_set_transport(
    update: TransportUpdate,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<()> {
    with_service(&state, |service| {
        let plugin = instance(service, update.instance_id)?;
        plugin
            .set_tempo(update.tempo)
            .map_err(|error| error.to_string())?;
        plugin
            .set_time_signature(update.numerator, update.denominator)
            .map_err(|error| error.to_string())?;
        plugin
            .set_playing(update.playing)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command]
pub fn vst3_process_block(
    request: ProcessBlockRequest,
    state: State<'_, Mutex<Vst3Service>>,
) -> CommandResult<ProcessBlockResponse> {
    with_service(&state, |service| {
        if request.block_size == 0 || request.block_size > 16_384 {
            return Err("audio block size must be in 1..=16384".to_string());
        }
        let plugin = instance(service, request.instance_id)?;
        let mut buffers = plugin
            .create_bus_audio_buffers(request.block_size)
            .map_err(|error| error.to_string())?;
        if request.inputs.len() != buffers.inputs.len() {
            return Err(format!(
                "expected {} input buses, received {}",
                buffers.inputs.len(),
                request.inputs.len()
            ));
        }
        for (source, target) in request.inputs.into_iter().zip(&mut buffers.inputs) {
            copy_bus(source, target, request.block_size)?;
        }
        plugin
            .process_bus_audio(&mut buffers)
            .map_err(|error| error.to_string())?;
        Ok(ProcessBlockResponse {
            outputs: buffers
                .outputs
                .into_iter()
                .map(|bus| bus.channels)
                .collect(),
            output_midi: plugin.take_output_midi(),
            latency_samples: plugin.latency_samples(),
            tail_samples: plugin.tail_samples(),
        })
    })
}

fn copy_bus(
    source: AudioBusInput,
    target: &mut AudioBusBuffer,
    block_size: usize,
) -> CommandResult<()> {
    if source.active != target.active {
        return Err("input bus activation does not match the plugin layout".to_string());
    }
    if source.channels.len() != target.channels.len() {
        return Err(format!(
            "expected {} channels on input bus, received {}",
            target.channels.len(),
            source.channels.len()
        ));
    }
    for (source, target) in source.channels.into_iter().zip(&mut target.channels) {
        if source.len() != block_size {
            return Err(format!(
                "expected {block_size} samples per channel, received {}",
                source.len()
            ));
        }
        target.copy_from_slice(&source);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_midi_channels() {
        assert!(midi_channel(0).is_err());
        assert!(midi_channel(17).is_err());
        assert!(midi_channel(1).is_ok());
        assert!(midi_channel(16).is_ok());
    }

    #[test]
    fn validates_audio_bus_shapes() {
        let mut target = AudioBusBuffer::new(2, 4, true);
        let source = AudioBusInput {
            active: true,
            channels: vec![vec![1.0; 4], vec![2.0; 4]],
        };
        copy_bus(source, &mut target, 4).unwrap();
        assert_eq!(target.channels[0], vec![1.0; 4]);
        assert_eq!(target.channels[1], vec![2.0; 4]);
    }

    #[test]
    #[ignore = "requires OPENDAW_VST3_TEST_PLUGIN to name an installed VST3"]
    fn loads_and_processes_an_installed_plugin() {
        let path = std::env::var("OPENDAW_VST3_TEST_PLUGIN")
            .expect("OPENDAW_VST3_TEST_PLUGIN must be set");
        let mut service = Vst3Service::default();
        let mut plugin = service.host.load_plugin(path).expect("load installed VST3");
        let state = plugin.save_state().expect("save plugin state");
        let layout = plugin.audio_bus_layout().expect("query audio buses");
        let mut buffers = vst3_host::BusAudioBuffers::new(&layout, 128, 48_000.0);
        plugin.start_processing().expect("start processing");
        plugin
            .process_bus_audio(&mut buffers)
            .expect("process silent block");
        plugin.stop_processing().expect("stop processing");
        plugin.load_state(&state).expect("restore plugin state");
        assert!(!plugin.info().uid.is_empty());
    }
}
