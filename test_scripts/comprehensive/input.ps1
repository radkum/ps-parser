# ========================================
# Comprehensive PowerShell Test Script for PS Parser
# ========================================

# Test 1: Basic Variables and Assignment
Write-Host "=== Test 1: Variables and Assignment ===" -ForegroundColor Green
$simpleVar = "Hello World"
$numberVar = 42
$floatVar = 3.14159
$boolVar = $true
$nullVar = $null

# Test variable access
Write-Output "Simple: $simpleVar"
Write-Output "Number: $numberVar"
Write-Output "Float: $floatVar"
Write-Output "Boolean: $boolVar"
Write-Output "Null: $nullVar"

# Test mixed type arithmetic (should trigger error handling)
Write-Output "Mixed types: $(3 + "invalid")"

# Test 4: String Operations
Write-Host "=== Test 4: String Operations ===" -ForegroundColor Green
$str1 = "Hello"
$str2 = "World"
Write-Output "Concatenation: $($str1 + " " + $str2)"
Write-Output "String multiplication: $($str1 * 3)"
Write-Output "String interpolation: $str1, $str2!"

# Test 5: Comparison Operators
Write-Host "=== Test 5: Comparison Operators ===" -ForegroundColor Green
$x = 10
$y = 20
Write-Output "Equal: $($x -eq $y)"
Write-Output "Not Equal: $($x -ne $y)"
Write-Output "Greater Than: $($x -gt $y)"
Write-Output "Less Than: $($x -lt $y)"
Write-Output "Greater or Equal: $($x -ge $y)"
Write-Output "Less or Equal: $($x -le $y)"

# Test 6: Logical Operators
Write-Host "=== Test 6: Logical Operators ===" -ForegroundColor Green
$true1 = $true
$false1 = $false
Write-Output "AND: $($true1 -and $false1)"
Write-Output "OR: $($true1 -or $false1)"
Write-Output "NOT: $(-not $true1)"
Write-Output "XOR: $($true1 -xor $false1)"

# Test 7: Arrays
Write-Host "=== Test 7: Arrays ===" -ForegroundColor Green
$array1 = @(1, 2, 3, 4, 5)
$array2 = @("apple", "banana", "cherry")
$mixedArray = @(1, "two", 3.0, $true, $null)

Write-Output "Number array: $array1"
Write-Output "String array: $array2"
Write-Output "Mixed array: $mixedArray"
Write-Output "Array length: $($array1.Length)"
Write-Output "First element: $($array1[0])"

# Test 8: Ranges
Write-Host "=== Test 8: Ranges ===" -ForegroundColor Green
$range1 = 1..5
$range2 = 10..1
Write-Output "Ascending range: $range1"
Write-Output "Descending range: $range2"

# Test 9: Hash Tables / Dictionaries
Write-Host "=== Test 9: Hash Tables ===" -ForegroundColor Green
$hash = @{
    Name = "John"
    Age = 30
    City = "New York"
}
Write-Output "Hash table: $hash"
Write-Output "Name: $($hash.Name)"
Write-Output "Age: $($hash['Age'])"

# Test 10: Conditional Statements
Write-Host "=== Test 10: Conditional Statements ===" -ForegroundColor Green
$score = 85

if ($score -ge 90) {
    Write-Output "Grade: A"
} elseif ($score -ge 80) {
    Write-Output "Grade: B"
} elseif ($score -ge 70) {
    Write-Output "Grade: C"
} else {
    Write-Output "Grade: F"
}

# Test 11: Switch Statements
Write-Host "=== Test 11: Switch Statements ===" -ForegroundColor Green
$day = "Monday"
switch ($day) {
    "Monday" { Write-Output "Start of work week" }
    "Friday" { Write-Output "TGIF!" }
    "Saturday" { Write-Output "Weekend!" }
    "Sunday" { Write-Output "Weekend!" }
    default { Write-Output "Regular day" }
}

# Test 12: Loops - For Loop
Write-Host "=== Test 12: For Loop ===" -ForegroundColor Green
for ($i = 1; $i -le 5; $i++) {
    Write-Output "For loop iteration: $i"
}

# Test 13: Loops - While Loop
Write-Host "=== Test 13: While Loop ===" -ForegroundColor Green
$counter = 1
while ($counter -le 3) {
    Write-Output "While loop iteration: $counter"
    $counter++
}

# Test 14: Loops - ForEach Loop
Write-Host "=== Test 14: ForEach Loop ===" -ForegroundColor Green
$fruits = @("apple", "banana", "orange")
foreach ($fruit in $fruits) {
    Write-Output "Fruit: $fruit"
}

# Test 15: Functions
Write-Host "=== Test 15: Functions ===" -ForegroundColor Green
function Get-Square($number) {
    return $number * $number
}

function Get-Greeting($name = "World") {
    return "Hello, $name!"
}

Write-Output "Square of 5: $(Get-Square 5)"
Write-Output "Greeting: $(Get-Greeting)"
Write-Output "Greeting with name: $(Get-Greeting "Alice")"

# Test 16: Advanced Function with Parameters
Write-Host "=== Test 16: Advanced Function ===" -ForegroundColor Green
function Test-Parameters {
    param(
        [string]$Name,
        [int]$Age = 25,
        [switch]$Verbose
    )
    
    $result = "Name: $Name, Age: $Age"
    if ($Verbose) {
        $result += " (Verbose mode)"
    }
    return $result
}

Write-Output "$(Test-Parameters -Name "Bob" -Age 30 -Verbose)"

# Test 17: String Matching and Regex
Write-Host "=== Test 17: String Matching ===" -ForegroundColor Green
$text = "PowerShell is awesome"
Write-Output "Contains 'Shell': $($text -like "*Shell*")"
Write-Output "Starts with 'Power': $($text -like "Power*")"
Write-Output "Matches regex: $($text -match "P\w+Shell")"

# Test 18: Type Casting
Write-Host "=== Test 18: Type Casting ===" -ForegroundColor Green
$stringNumber = "123"
$intNumber = [int]$stringNumber
$floatNumber = [double]$stringNumber
Write-Output "String: $stringNumber (Type: $($stringNumber.GetType().Name))"
Write-Output "String: $stringNumber (Type: $($stringNumber.GetType()['Name']))"
Write-Output "Int: $intNumber (Type: $($intNumber.GetType().Name))"
Write-Output "Float: $floatNumber (Type: $($floatNumber.GetType().Name))"

# Test 19: Error Handling and $? Variable
Write-Host "=== Test 19: Error Handling ===" -ForegroundColor Green
$successful = 1 + 1
Write-Output "Successful operation result: $successful"
Write-Output "Last operation success: $?"

# Attempt an operation that should fail
$failed = 3 + "invalid string"
Write-Output "Failed operation result: $failed"
Write-Output "Last operation success after error: $?"

# Test 20: Complex Expressions
Write-Host "=== Test 20: Complex Expressions ===" -ForegroundColor Green
$result = ((10 + 5) * 2) / 3
Write-Output "Complex arithmetic: $result"

$complexCondition = ($true -and ($false -or $true)) -xor $false
Write-Output "Complex logical: $complexCondition"

# Test 21: Pipeline Operations (basic)
Write-Host "=== Test 21: Pipeline Operations ===" -ForegroundColor Green
$numbers = 1..10
$evenNumbers = $numbers | Where-Object { $_ % 2 -eq 0 }
Write-Output "Even numbers: $evenNumbers"

# Test 22: Special Variables
Write-Host "=== Test 22: Special Variables ===" -ForegroundColor Green
Write-Output "PowerShell Version: $($PSVersionTable.PSVersion)"
Write-Output "Execution Policy: $(Get-ExecutionPolicy)"
Write-Output "Current Location: $(Get-Location)"

# Test 23: Nested Structures
Write-Host "=== Test 23: Nested Structures ===" -ForegroundColor Green
$nestedData = @{
    Users = @(
        @{ Name = "Alice"; Age = 30; Skills = @("PowerShell", "Python") }
        @{ Name = "Bob"; Age = 25; Skills = @("Java", "C#") }
    )
    Settings = @{
        Theme = "Dark"
        Language = "en-US"
    }
}

Write-Output "First user: $($nestedData.Users[0].Name)"
Write-Output "First user skills: $($nestedData.Users[0].Skills -join ', ')"
Write-Output "First user skills: $($nestedData.Users[0].SkillsUnknonw -join ', ')"
Write-Output "Theme setting: $($nestedData.Settings.Theme)"

# Test 24: Comments and Documentation
Write-Host "=== Test 24: Comments ===" -ForegroundColor Green
# This is a single line comment
Write-Output "Single line comment test"

<#
    This is a multi-line comment
    that spans multiple lines
    and can contain documentation
#>
Write-Output "Multi-line comment test"

# Test 25: Script Blocks
Write-Host "=== Test 25: Script Blocks ===" -ForegroundColor Green
$scriptBlock = {
    param($x, $y) return $x + $y
}

$result = & $scriptBlock 10 20
Write-Output "Script block result: $result"

Write-Host "=== All Tests Completed ===" -ForegroundColor Cyan
Write-Output "Test script execution finished. Check results above for any parsing issues."
