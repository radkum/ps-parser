# ========================================
# Focused PowerShell Test Script for PS Parser
# Testing only supported features based on parser capabilities
# ========================================

# Test 1: Basic Variables and Types
$string = "Hello World"
$number = 42
$float = 3.14
$bool = $true
$null_val = $null

# Test 2: Environment Variables  
$path = $env:PATH
$temp = $env:TEMP
$user = $env:USERNAME

# Test 3: Arithmetic Operations
$add = 5 + 3
$sub = 10 - 4
$mul = 6 * 7
$div = 20 / 4
$mod = 15 % 4

# Test arithmetic with variables
$a = 10
$b = 5
$result = ($a + $b) * 2

# Test error case - this should set $? to false
$error_test = 3 + "invalid"
$error_status = $?

# Test 4: String Operations
$str1 = "Hello"
$str2 = "World"
$concat = $str1 + " " + $str2
$repeat = $str1 * 3

# Test 5: Comparison Operations
$eq = 5 -eq 5
$ne = 5 -ne 3
$gt = 10 -gt 5
$lt = 3 -lt 8
$ge = 5 -ge 5
$le = 3 -le 5

# Test 6: Logical Operations
$and_op = $true -and $false
$or_op = $true -or $false
$not_op = -not $true
$xor_op = $true -xor $false

# Test 7: Arrays
$array = @(1, 2, 3, 4, 5)
$string_array = @("apple", "banana", "cherry")
$mixed = @(1, "two", 3.0, $true)

# Test 8: Ranges
$range1 = 1..5
$range2 = 10..1

# Test 9: Type Casting
$cast_int = [int]"123"
$cast_float = [float]"3.14"
$cast_string = [string]42
$cast_bool = [bool]1

# Test 10: Conditional Statements
if ($true) {
    $if_result = "condition true"
}

if ($false) {
    $else_result = "false branch"
} else {
    $else_result = "true branch"
}

$score = 85
if ($score -ge 90) {
    $grade = "A"
} elseif ($score -ge 80) {
    $grade = "B"
} else {
    $grade = "C"
}

# Test 11: For Loop
for ($i = 1; $i -le 3; $i++) {
    $for_result = "iteration " + $i
}

# Test 12: While Loop
$counter = 1
while ($counter -le 3) {
    $while_result = "count " + $counter
    $counter++
}

# Test 13: ForEach Loop
$items = @("a", "b", "c")
foreach ($item in $items) {
    $foreach_result = "item: " + $item
}

# Test 14: Switch Statement
$day = "Monday"
switch ($day) {
    "Monday" { $switch_result = "Start of week" }
    "Friday" { $switch_result = "TGIF" }
    default { $switch_result = "Regular day" }
}

# Test 15: Functions
function Get-Double($x) {
    return $x * 2
}

function Get-Sum($a, $b) {
    return $a + $b
}

$double_result = Get-Double 5
$sum_result = Get-Sum 3 7

# Test 16: String Matching
$text = "PowerShell"
$like_test = $text -like "*Shell"
$match_test = $text -match "Power"

# Test 17: Contains Operations
$list = @("apple", "banana", "cherry")
$contains_test = $list -contains "banana"
$in_test = "apple" -in $list

# Test 18: Join and Split Operations
$join_test = $list -join ", "
$split_test = "a,b,c,d" -split ","

# Test 19: Replace Operations
$replace_test = "Hello World" -replace "World", "PowerShell"

# Test 20: Bitwise Operations
$band = 5 -band 3
$bor = 5 -bor 3
$bxor = 5 -bxor 3
$bnot = -bnot 5
$shl = 4 -shl 2
$shr = 16 -shr 2

# Test 21: Increment/Decrement
$inc_test = 5
$inc_test++
++$inc_test

$dec_test = 10
$dec_test--
--$dec_test

# Test 22: Parenthesized Expressions
$paren_result = (3 + 4) * (5 - 2)

# Test 23: Sub-expressions
$sub_expr = $(Get-Double 10)

# Test 24: Negation
$neg_number = -42
$neg_bool = !$true

# Test 25: Complex Nested Expressions
$complex = ((5 + 3) * 2) + (10 / 2) - 1

# Test 26: Variable Assignment with Operators
$assign_add = 10
$assign_add += 5

$assign_sub = 20
$assign_sub -= 3

$assign_mul = 4
$assign_mul *= 3

# Test 27: Format String Operations
$format_result = "Value: {0}, Name: {1}" -f 42, "Test"

# Test 28: Comments
# This is a single line comment
$comment_test = "after comment"

<# 
   This is a block comment
   spanning multiple lines
#>
$block_comment_test = "after block comment"

# Test 29: Multiple Statements
$stmt1 = 1; $stmt2 = 2; $stmt3 = $stmt1 + $stmt2

# Test 30: Error Recovery Test
$error1 = 1 + "bad"; $good = 5 + 5; $status = $?
