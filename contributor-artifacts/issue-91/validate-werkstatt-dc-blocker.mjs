import fs from "node:fs"
import path from "node:path"
import vm from "node:vm"
import {fileURLToPath} from "node:url"

const sampleRate = 48_000
const root = path.dirname(fileURLToPath(import.meta.url))
const source = fs.readFileSync(path.join(root, "werkstatt-dc-blocker.js"), "utf8")
const context = {sampleRate, Math, Float32Array}
vm.runInNewContext(`${source}\nglobalThis.ProcessorUnderTest = Processor`, context)

const render = (left, right) => {
    const processor = new context.ProcessorUnderTest()
    const outL = new Float32Array(left.length)
    const outR = new Float32Array(right.length)
    const blockSize = 128
    for (let s0 = 0, index = 0; s0 < left.length; s0 += blockSize, index++) {
        const s1 = Math.min(s0 + blockSize, left.length)
        processor.process({src: [left, right], out: [outL, outR]}, {
            s0,
            s1,
            index,
            bpm: 120,
            p0: 0,
            p1: 0,
            flags: index === 0 ? 2 : 0
        })
    }
    return [outL, outR]
}

const rms = (values, start = 0) => {
    let sum = 0.0
    for (let i = start; i < values.length; i++) sum += values[i] * values[i]
    return Math.sqrt(sum / (values.length - start))
}

const maxAbs = (values, start = 0) => {
    let max = 0.0
    for (let i = start; i < values.length; i++) max = Math.max(max, Math.abs(values[i]))
    return max
}

const assert = (condition, message) => {
    if (!condition) throw new Error(message)
}

const dcLength = sampleRate * 4
const dcL = new Float32Array(dcLength).fill(0.5)
const dcR = new Float32Array(dcLength).fill(-0.25)
const [dcOutL, dcOutR] = render(dcL, dcR)
const dcTailStart = sampleRate * 3
const dcTailMaxL = maxAbs(dcOutL, dcTailStart)
const dcTailMaxR = maxAbs(dcOutR, dcTailStart)

const toneLength = sampleRate * 2
const toneL = new Float32Array(toneLength)
const toneR = new Float32Array(toneLength)
for (let i = 0; i < toneLength; i++) {
    const sample = Math.sin(2.0 * Math.PI * 1_000 * i / sampleRate)
    toneL[i] = sample
    toneR[i] = sample * 0.75
}
const [toneOutL, toneOutR] = render(toneL, toneR)
const toneStart = sampleRate
const gainL = rms(toneOutL, toneStart) / rms(toneL, toneStart)
const gainR = rms(toneOutR, toneStart) / rms(toneR, toneStart)

assert(dcTailMaxL < 1e-5, `left DC tail too large: ${dcTailMaxL}`)
assert(dcTailMaxR < 1e-5, `right DC tail too large: ${dcTailMaxR}`)
assert(Math.abs(gainL - 1.0) < 1e-4, `left 1 kHz gain changed: ${gainL}`)
assert(Math.abs(gainR - 1.0) < 1e-4, `right 1 kHz gain changed: ${gainR}`)
assert([...dcOutL, ...dcOutR, ...toneOutL, ...toneOutR].every(Number.isFinite), "non-finite output")

console.log(JSON.stringify({
    sampleRate,
    cutoffHz: 2,
    dcTailMax: {left: dcTailMaxL, right: dcTailMaxR},
    gainAt1kHz: {left: gainL, right: gainR},
    result: "pass"
}, null, 2))
