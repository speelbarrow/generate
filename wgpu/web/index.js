import init from "./gen/{{project-name}}.js"
import wasm from "./gen/{{project-name}}_bg.wasm"
await init({ module_or_path: wasm })
