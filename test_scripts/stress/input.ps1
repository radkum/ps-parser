# ========================================
# PowerShell Parser Stress Test
# Testing edge cases, error conditions, and complex scenarios
# ========================================

# Test 1: Edge Cases with Numbers
$zero = 0
$negative = -123
$large_int = 999999999
$small_float = 0.000001
$scientific = 1.23e-4

# Test 2: Edge Cases with Strings
$empty_string = ""
$single_char = "a"
$with_quotes = "He said ""Hello"""
$with_backslash = "Path\to\file"
$unicode = "Café"

# Test 3: Division by Zero (should trigger error)
$div_by_zero = 10 / 0
$div_zero_status = $?

# Test 4: Type Conversion Errors
$invalid_int = [int]"not_a_number"
$invalid_conversion_status = $?

$valid_after_error = [int]"123"
$valid_status = $?

# Test 5: Deeply Nested Expressions
$nested = ((((1 + 2) * 3) + 4) * 5) + 6

# Test 6: Mixed Type Operations
$mixed1 = 5 + "10"    # Should convert string to number
$mixed2 = "Hello" + 123  # Should convert number to string
$mixed3 = $true + 1      # Should work
$mixed4 = $null + 5      # Should work

# Test 7: Array Access Edge Cases
$arr = @(1, 2, 3)
$valid_index = $arr[0]
$last_index = $arr[2]
# $invalid_index = $arr[10]  # Would be out of bounds

# Test 8: Complex Boolean Logic
$complex_bool = (($true -and $false) -or ($true -xor $false)) -and (-not $false)

# Test 9: String with Special Characters
$special_chars = "!@#$%^&*()_+-={}[]|;:,.<>?"
$newlines = "Line1`nLine2`nLine3"
$tabs = "Col1`tCol2`tCol3"

# Test 10: Large Arrays
$large_array = 1..1000
$array_operation = $large_array[999]

# Test 11: Chained Operations
$chained = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10

# Test 12: Precedence Testing
$precedence1 = 2 + 3 * 4      # Should be 14, not 20
$precedence2 = (2 + 3) * 4    # Should be 20
$precedence3 = 2 * 3 + 4      # Should be 10
$precedence4 = 2 * (3 + 4)    # Should be 14

# Test 13: Error Recovery in Sequences
$seq1 = 1 + 1; $seq_error = 2 + "bad"; $seq2 = 3 + 3; $recovery_status = $?

# Test 14: Comparison Edge Cases
$str_num_eq = "5" -eq 5       # String vs Number comparison
$case_sensitive = "Hello" -ceq "hello"
$case_insensitive = "Hello" -ieq "hello"

# Test 15: Null Comparisons
$null_eq_null = $null -eq $null
$null_eq_zero = $null -eq 0
$null_eq_empty = $null -eq ""
$null_eq_false = $null -eq $false

# Test 16: Boolean Conversions
$bool_from_int = [bool]0      # Should be false
$bool_from_int2 = [bool]1     # Should be true
$bool_from_int3 = [bool]-1    # Should be true
$bool_from_string = [bool]""  # Should be false
$bool_from_string2 = [bool]"hello"  # Should be true

# Test 17: Arithmetic with Different Types
$int_float = 5 + 3.14
$float_int = 3.14 + 5
$bool_arithmetic = $true + $false + $true  # Should be 2

# Test 18: String Repetition Edge Cases
$repeat_zero = "test" * 0     # Should be empty
$repeat_one = "test" * 1      # Should be "test"
$repeat_negative = "test" * -1  # Should handle gracefully

# Test 19: Modulo with Different Types
$mod_float = 7.5 % 2.5
$mod_negative = -7 % 3
$mod_by_negative = 7 % -3

# Test 20: Bitwise with Edge Cases
$bitwise_zero = 0 -band 5
$bitwise_negative = -1 -band 5
$shift_large = 1 -shl 31

# Test 21: Multiple Assignment Operators
$multi = 10
$multi += 5    # Should be 15
$multi -= 3    # Should be 12  
$multi *= 2    # Should be 24
$multi /= 4    # Should be 6
$multi %= 4    # Should be 2

# Test 22: Complex String Operations
$str_complex = ("Hello" + " " + "World") * 2
$str_compare = ("Hello" -eq "Hello") -and ("World" -ne "world")

# Test 23: Environment Variable Edge Cases
$env_exists = $env:programfiles
$env_maybe_not_exists = $env:NONEXISTENT_VAR_12345
$env_empty = $env:EMPTY_VAR

# Test 24: Range Edge Cases
$range_same = 5..5           # Single element range
$range_reverse = 10..1       # Reverse range
$range_negative = -5..-1     # Negative range

# Test 25: Function Edge Cases
function Test-Empty() {
    # Empty function
}

function Test-OneParam($p) {
    return $p * 2
}

function Test-MultiParam($a, $b, $c) {
    return $a + $b + $c
}

$empty_result = Test-Empty
$one_param_result = Test-OneParam 5
$multi_param_result = Test-MultiParam 1 2 3

# Test 26: Nested Function Calls
function Inner($x) { return $x + 1 }
function Outer($y) { return Inner($y * 2) }
$nested_func_result = Outer 5
$nested_func_result

# Test 27: Variable Scoping in Blocks
$global_var = "global"
if ($true) {
    $block_var = "block"
    $scope_test = $global_var + $block_var
}
$scope_test
# Test 28: Switch with Edge Cases
$switch_var = $null
switch ($switch_var) {
    $null { $switch_null_result = "matched null" }
    "" { $switch_empty_result = "matched empty" }
    0 { $switch_zero_result = "matched zero" }
    default { $switch_default_result = "default case" }
}
$switch_null_result

# Test 29: Loop Edge Cases
for ($i = 10; $i -gt 10; $i--) {
    $never_executed = "should not run"
}

$countdown = 3
while ($countdown -le 0) {
    $while_never = "should not run"
}

foreach ($item in @()) {
    $foreach_empty = "should not run"
}

# Test 30: Error Cascade Testing
$cascade1 = 1 + 1           # Good
$cascade_error = 2 + "bad"  # Error
$cascade2 = 3 + 3           # Good after error
$cascade3 = $?              # Should reflect last operation status

# Test 31: Memory/Performance Test
$perf_array = 1..10000
$perf_sum = 0
for ($i = 0; $i -lt $perf_array.Length; $i++) {
    $perf_sum += $perf_array[$i]
}

# Test 32: Complex Conditional Logic
$complex_if = if (($true -and $false) -or ($true -xor $false)) { "complex true" } else { "complex false" }
$complex_if
# Test 33: String Escape Sequences
$escaped = "Quote: `" Newline: `n Tab: `t Backslash: ``"

# Test 34: Final Status Check
$final_good_operation = 100 + 200
$final_status = $?
