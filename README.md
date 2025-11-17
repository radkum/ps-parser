# ps-parser

[![Crates.io](https://img.shields.io/crates/v/ps-parser.svg)](https://crates.io/crates/ps-parser)
[![Crates.io](https://img.shields.io/crates/d/ps-parser.svg)](https://crates.io/crates/ps-parser)
[![Docs.rs](https://docs.rs/ps-parser/badge.svg)](https://docs.rs/ps-parser)
[![License](https://img.shields.io/crates/l/ps-parser.svg)](LICENSE)

A PowerShell parser written in Rust.
Parse, evaluate and deobfuscate PowerShell scripts with idiomatic Rust types.

## Goal

Malicious scripts typically use "safe" operations to obfuscate "unsafe" ones. For example, arithmetic operations are used to obfuscate function arguments.

The goal of this parser is to combat obfuscation in PowerShell by evaluating everything that is "safe" but not anything that is "unsafe". Ps-parser deliver also possibility to get script "tokens"

## Features

- PowerShell script parsing using [pest](https://pest.rs/) grammar
- Value types for PowerShell objects (`String`, `Int`, `HashTable`, `ScriptBlock`, etc.)
- Arithmetic, logical, and string operations
- Script block evaluation and variable management
- HashTable and Array support
- Extensible for custom PowerShell types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ps-parser = "0.4.1"
```

## Usage

### Parse a PowerShell script and eval only "safe" operations

```rust
use ps_parser::PowerShellSession;

let mut ps = PowerShellSession::new(); 
let script = r#"
$y = 2/4
$arg = 20MB*$y

# Get-Process is not "safe" to evaluate, so Where-Object is also not evaluated
Get-Process | Where-Object WorkingSet -GT $arg  
$evenNumbers = 1..10 | Where-Object { $_ % 2 -eq 0 } # Where-Object is evaluated, because 1..10 is "safe" 
"#;

let result = ps.parse_input(script)?.deobfuscated();
println!("{}", result);
```

Output: 
```powershell
$y = 0.5
$arg = 10485760
Get-Process | Where-Object WorkingSet -GT 10485760

$evennumbers = @(2,4,6,8,10)
```

### Work with arrays, hashtables, scriptBlocks, functions and cmdlets
```rust
use ps_parser::PowerShellSession;

let mut ps = PowerShellSession::new(); 
let script = r#"
function Mul-By-Global($x) {return $x * $global:c}
$a = @('a', 'b', 'c');$b=$a[2]
$global:c = & {param($x, $y) return $x + $y} 1 2
$d = Mul-By-Global 5
$c + $d
"#;

let script_result = ps.parse_input(script)?;
println!("Deobfuscated:\n{}\n", script_result.deobfuscated());
println!("Output:\n{}\n", script_result.output());
```

Output: 
```powershell
Deobfuscated:
function Mul-By-Global($x) {return $x * $global:c}
$a = @('a','b','c')
$b = 'c'
$c = 3
$d = 15
40

Output:
40
```

### Deal with deobfuscation
```rust
use ps_parser::PowerShellSession;

let mut ps = PowerShellSession::new(); 
let script = r#"
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Ârmí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))";

$encoded = [syStem.texT.EncoDInG]::unIcoDe.geTstRiNg([SYSTem.cOnVERT]::froMbasE64striNg("ZABlAGMAbwBkAGUAZAA="));
"#;

let script_result = ps.parse_input(script)?;
println!("{}", script_result.deobfuscated());
```

Output: 
```powershell
$ilrynqstt = 'System.Management.Automation.ArmiUtils'
$encoded = 'decoded'
```

### Work environmental variables

```rust
use ps_parser::PowerShellSession;

let mut ps = PowerShellSession::new().with_variables(Variables::env()); 
let input = r#"$env:programfiles"#;
let script_result = ps.parse_input(input)?;
println!("{}", script_result.result());
```

Output: 
```powershell
C:\Program Files
```

### Get tokens and errors

```rust
use ps_parser::PowerShellSession;

let mut ps = PowerShellSession::new(); 
let input = r#"
$a = 5
$b = $a * 2
Write-Output "Addition: $($a + $b)"
$var = 1 + "Hello, World!" # Powershell cannot cast string to int
"#;
let script_result = ps.parse_input(input)?;
println!("{}", script_result.tokens().expandable_strings()[0]);
println!("{}", script_result.tokens().expression()[0]);
println!("errors: {:?}", script_result.errors());
```

Output: 
```rust
StringExpandable("\"Addition: $($a + $b)\"", "Addition: 15")
Expression("5", Int(5))
errors: [ValError(InvalidCast("String", "Int"))]
```

## Future plans
- parse script_param_block
- parse named blocks
- change Val::Array from Vec to struct
- deobfuscate -encodedArguments
- benchmarks
- "filter" functions
- eval "switch" statements
- parse "enum" and "class" statements
- more token kinds
- implement Get-ExecutionPolicy cmdlet using registry

## Documentation

- [API Reference (docs.rs)](https://docs.rs/ps-parser)
- [Crate on crates.io](https://crates.io/crates/ps-parser)

## License

Licensed under MIT or Apache-2.0, at your option.
See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.