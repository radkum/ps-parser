# ps-parser

> A lightweight PowerShell parser written in Rust 🦀⚡

`ps-parser` is a Rust crate for parsing PowerShell scripts, designed for speed, correctness, and ease of integration into Rust projects.

---

## ✨ Features

- Parses PowerShell code into structured syntax trees
- Handles variables, pipelines, functions, and expressions
- Zero dependencies (or minimal)
- Built for extensibility and performance
- No-std environment

---

## 🚀 Getting Started

Crate it's not ready. It's prerelase version


## TODO
Version 0.1.0
- handle all special variables ($$, $^, $?, $_)
- add support for param in scripBlock
- eval all statemets, eg. preparse functions and later try to call it
- instead of compilation feature flag, add option to Parser construction .with_culture("en-US)
- make parser no_std