import {mkdir, readdir, rm} from "node:fs/promises"
import {resolve} from "node:path"

const output = resolve(import.meta.dirname, "../test/gen")
await mkdir(output, {recursive: true})
const entries = await readdir(output)
await Promise.all(entries.map(entry => rm(resolve(output, entry), {recursive: true, force: true})))
