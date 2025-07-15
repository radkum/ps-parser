mod ps_parser;
use ps_parser::PowerShellParser;

type DynError = Box<dyn std::error::Error>;
type DynResult<T> = core::result::Result<T, DynError>;
fn entry() -> DynResult<()> {
    let input = r#"
# This is a single line comment
$a = 1; $b = 2; Write-Output $a

Write-Output "Hello"  # Another comment

<#
    This is a
    multi-line block comment
#>

while ($true) {
    if ($someCondition) {
        break
    }
    # other code
}

switch ($var) {
    "a" { Write-Output "A" }
    1 { Write-Output "One" }
    default { Write-Output "Other" }
}

function Get-Square {
    param($x)
    return $x * $x
}

function Say-Hello {
    Write-Output "Hello"
}

$x="hello"
        Write-Host $x
        $y = 42
        Start-Process "notepad.exe"

        $x = 42
if ($x -eq 1) {
    Write-Output "One"
} elseif ($x -eq 2) {
    Write-Output "Two"
} else {
    Write-Output "Other"
}

Get-Process | Where-Object { $_.CPU -gt 100 }

$numbers = 1..5

foreach ($n in $numbers) {
    Write-Output $n
}

$x = 0
while ($x -lt 5) {
    Write-Output $x
    $x = $x + 1
}

    $pi = 3.1415
$half = .5
$hex = 0xFF
$bin = 0b1101

$name = "Alice"
$msg = "Hello, $name. Today is $day."
$escaped = "She said: `"Hi`""
$literal = 'Hello, $name'

$a = 1, 2, 3
$b = @("one", "two", "three")
$c = @(1, 2, @(3, 4))

#Matt Graebers second Reflection method 
$VMRviwsbtehQfPtxbt=$null;
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))"
"#;
    PowerShellParser::parse_input(input)?;

    Ok(())
}
