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
+ parse each powershell variant -  DONE
- extend parser to handle all defined syntax: classes, -and or -or operators, all statements, file_redirection_operator, merging_redirection_operator, format_operator, label, trap, try catch, finnally
- handle special variables ($$, $^, $?, $_) and environemtns
- instead of compilation feature flag, add option to Parser construction .with_culture("en-US)
- add option to use predefined variables or system variables
- deal with letter case agnostic powershell