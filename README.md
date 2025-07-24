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
VERSION 0.0.1
+ parse each powershell variant -  DONE
+ support for each expression -  DONE
- improve evaluation of pipelines, to handle 'redirections', more commands, etc
- extend parser to handle all statements
- add deobfuscation() function to return evaluated script
- add function() to filter out token from script

VERSION 0.1.0
- handle special variables ($$, $^, $?, $_), environment variables, global and local variables
- eval all statemets, eg. preparse functions and later try to call it
- instead of compilation feature flag, add option to Parser construction .with_culture("en-US)
- deal with letter case agnostic powershell
- make parser no_std