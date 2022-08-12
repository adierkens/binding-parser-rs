# binding-parser-rs

An experiment to see what potential performance impacts there are for writing [this](https://github.com/player-ui/player/blob/main/core/binding-grammar/src/custom/index.ts) binding parser in rust -> wasm

Benchmark in node (i9-12900k, 64GB ram)

```
┌─────────┬─────────────┬──────┬────────────────────┐
│ (index) │    name     │ time │     opsPerSec      │
├─────────┼─────────────┼──────┼────────────────────┤
│    0    │ 'parsimmon' │ 236  │ 131355.93220338985 │
│    1    │   'ebnf'    │ 1821 │ 17023.61339923119  │
│    2    │  'custom'   │  37  │ 837837.8378378379  │
│    3    │   'rust'    │ 145  │ 213793.10344827588 │
└─────────┴─────────────┴──────┴────────────────────┘
```

**TODO** bench it in a browser

To test it in node:

```
wasm-pack build --target nodejs
node --experimental-modules --experimental-wasm-modules  .\src\index.mjs
```
