import {invoke} from "@tauri-apps/api/core"

export interface Vst3PluginInfo {
    path: string
    name: string
    vendor: string
    version: string
    category: string
    uid: string
    audio_inputs: number
    audio_outputs: number
    has_midi_input: boolean
    has_midi_output: boolean
    has_gui: boolean
}

export interface Vst3Parameter {
    id: number
    name: string
    value: number
    min: number
    max: number
    default: number
    unit: string
    step_count: number
    can_automate: boolean
    is_read_only: boolean
    is_bypass: boolean
    flags: number
}

export interface Vst3AudioBus {
    channel_count: number
    active: boolean
}

export interface Vst3InstanceInfo {
    instanceId: number
    plugin: Vst3PluginInfo
    parameters: ReadonlyArray<Vst3Parameter>
    audioBuses: {inputs: ReadonlyArray<Vst3AudioBus>, outputs: ReadonlyArray<Vst3AudioBus>}
    latencySamples: number
    tailSamples: number
}

export type Vst3MidiMessage =
    | {type: "noteOn", channel: number, note: number, velocity: number}
    | {type: "noteOff", channel: number, note: number, velocity: number}
    | {type: "controlChange", channel: number, controller: number, value: number}
    | {type: "pitchBend", channel: number, value: number}
    | {type: "channelAftertouch", channel: number, pressure: number}
    | {type: "polyAftertouch", channel: number, note: number, pressure: number}

export interface Vst3AudioBusInput {
    active: boolean
    channels: ReadonlyArray<ReadonlyArray<number>>
}

export interface Vst3ProcessResponse {
    outputs: Array<Array<Array<number>>>
    outputMidi: ReadonlyArray<unknown>
    latencySamples: number
    tailSamples: number
}

export namespace Vst3Client {
    export const isAvailable = (): boolean => "__TAURI_INTERNALS__" in window

    export const scan = (): Promise<ReadonlyArray<Vst3PluginInfo>> =>
        invoke("vst3_scan")

    export const load = (path: string): Promise<Vst3InstanceInfo> =>
        invoke("vst3_load", {path})

    export const unload = (instanceId: number): Promise<void> =>
        invoke("vst3_unload", {instanceId})

    export const setParameter = (instanceId: number, parameterId: number, value: number,
                                 sampleOffset?: number): Promise<string> =>
        invoke("vst3_set_parameter", {change: {instanceId, parameterId, value, sampleOffset}})

    export const saveState = (instanceId: number): Promise<Array<number>> =>
        invoke("vst3_save_state", {instanceId})

    export const loadState = (instanceId: number, pluginState: ReadonlyArray<number>): Promise<void> =>
        invoke("vst3_load_state", {instanceId, pluginState})

    export const sendMidi = (instanceId: number, message: Vst3MidiMessage,
                             sampleOffset: number = 0): Promise<void> =>
        invoke("vst3_send_midi", {event: {instanceId, sampleOffset, message}})

    export const setTransport = (instanceId: number, tempo: number, numerator: number,
                                 denominator: number, playing: boolean): Promise<void> =>
        invoke("vst3_set_transport", {update: {instanceId, tempo, numerator, denominator, playing}})

    export const processBlock = (instanceId: number, blockSize: number,
                                 inputs: ReadonlyArray<Vst3AudioBusInput>): Promise<Vst3ProcessResponse> =>
        invoke("vst3_process_block", {request: {instanceId, blockSize, inputs}})
}
