// @label DC Blocker (2 Hz)

class Processor {
    cutoff = 2.0
    initialized = false

    x1L = 0.0; x2L = 0.0; y1L = 0.0; y2L = 0.0
    x1R = 0.0; x2R = 0.0; y1R = 0.0; y2R = 0.0

    b0 = 0.0; b1 = 0.0; b2 = 0.0; a1 = 0.0; a2 = 0.0

    recalcCoefficients() {
        const resonance = Math.SQRT1_2
        const w0 = 2.0 * Math.PI * this.cutoff / sampleRate
        const alpha = Math.sin(w0) / (2.0 * resonance)
        const cosw0 = Math.cos(w0)
        const a0 = 1.0 + alpha

        this.b0 = ((1.0 + cosw0) / 2.0) / a0
        this.b1 = (-(1.0 + cosw0)) / a0
        this.b2 = this.b0
        this.a1 = (-2.0 * cosw0) / a0
        this.a2 = (1.0 - alpha) / a0
        this.initialized = true
    }

    resetHistory() {
        this.x1L = 0.0; this.x2L = 0.0; this.y1L = 0.0; this.y2L = 0.0
        this.x1R = 0.0; this.x2R = 0.0; this.y1R = 0.0; this.y2R = 0.0
    }

    process({src, out}, {s0, s1, flags}) {
        if (!this.initialized) this.recalcCoefficients()
        if ((flags & 2) !== 0) this.resetHistory()

        const [srcL, srcR] = src
        const [outL, outR] = out
        for (let i = s0; i < s1; i++) {
            const nextL = this.b0 * srcL[i] + this.b1 * this.x1L + this.b2 * this.x2L
                - this.a1 * this.y1L - this.a2 * this.y2L
            this.x2L = this.x1L; this.x1L = srcL[i]
            this.y2L = this.y1L; this.y1L = nextL
            outL[i] = nextL

            const nextR = this.b0 * srcR[i] + this.b1 * this.x1R + this.b2 * this.x2R
                - this.a1 * this.y1R - this.a2 * this.y2R
            this.x2R = this.x1R; this.x1R = srcR[i]
            this.y2R = this.y1R; this.y1R = nextR
            outR[i] = nextR
        }
    }
}
