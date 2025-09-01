# PowerShell Test Script Runner for PS Parser
# Run this script to test your parser with the generated test files

Write-Host "PS Parser Test Suite" -ForegroundColor Cyan
Write-Host "===================" -ForegroundColor Cyan

$testScripts = @(
    @{ Name = "Parser Focused Test"; Path = "test_scripts/parser_focused_test.ps1"; Description = "Tests core parser features" },
    @{ Name = "Stress Test"; Path = "test_scripts/stress_test.ps1"; Description = "Tests edge cases and error handling" },
    @{ Name = "Comprehensive Test"; Path = "test_scripts/comprehensive_test.ps1"; Description = "Full PowerShell feature test" }
)

foreach ($test in $testScripts) {
    Write-Host "`n🧪 $($test.Name)" -ForegroundColor Green
    Write-Host "   Description: $($test.Description)" -ForegroundColor Gray
    Write-Host "   File: $($test.Path)" -ForegroundColor Gray
    
    if (Test-Path $test.Path) {
        $content = Get-Content $test.Path -Raw
        $lines = ($content -split "`n").Count
        $chars = $content.Length
        
        Write-Host "   ✅ File exists ($lines lines, $chars characters)" -ForegroundColor Yellow
        Write-Host "   💡 To test with your parser, run:" -ForegroundColor Cyan
        Write-Host "      cargo run -- '$($test.Path)'" -ForegroundColor White
        
        # You can add actual parser testing here:
        # Write-Host "   🔄 Testing with parser..." -ForegroundColor Blue
        # $result = & cargo run -- $test.Path
        # if ($LASTEXITCODE -eq 0) {
        #     Write-Host "   ✅ Parser test passed" -ForegroundColor Green
        # } else {
        #     Write-Host "   ❌ Parser test failed" -ForegroundColor Red
        # }
    } else {
        Write-Host "   ❌ File not found: $($test.Path)" -ForegroundColor Red
    }
}

Write-Host "`n📋 Test Script Features:" -ForegroundColor Cyan
Write-Host "   • Variables and assignments" -ForegroundColor White
Write-Host "   • Environment variables" -ForegroundColor White  
Write-Host "   • Arithmetic operations" -ForegroundColor White
Write-Host "   • String operations" -ForegroundColor White
Write-Host "   • Comparison operators" -ForegroundColor White
Write-Host "   • Logical operators" -ForegroundColor White
Write-Host "   • Arrays and ranges" -ForegroundColor White
Write-Host "   • Conditional statements (if/else)" -ForegroundColor White
Write-Host "   • Loops (for, while, foreach)" -ForegroundColor White
Write-Host "   • Switch statements" -ForegroundColor White
Write-Host "   • Functions" -ForegroundColor White
Write-Host "   • Type casting" -ForegroundColor White
Write-Host "   • Error handling (" -NoNewline -ForegroundColor White
Write-Host "$?" -NoNewline -ForegroundColor Yellow
Write-Host " variable)" -ForegroundColor White
Write-Host "   • Bitwise operations" -ForegroundColor White
Write-Host "   • String matching and regex" -ForegroundColor White
Write-Host "   • Complex nested expressions" -ForegroundColor White

Write-Host "`n🎯 Usage Instructions:" -ForegroundColor Cyan
Write-Host "   1. Review the generated test scripts" -ForegroundColor White
Write-Host "   2. Integrate them with your parser testing framework" -ForegroundColor White
Write-Host "   3. Run: " -NoNewline -ForegroundColor White
Write-Host "cargo test" -ForegroundColor Yellow
Write-Host "   4. For individual files: " -NoNewline -ForegroundColor White
Write-Host "cargo run -- <script_path>" -ForegroundColor Yellow

Write-Host "`n🔧 Customization:" -ForegroundColor Cyan
Write-Host "   • Modify test scripts to focus on specific features" -ForegroundColor White
Write-Host "   • Add more edge cases as needed" -ForegroundColor White
Write-Host "   • Uncomment parser integration code above" -ForegroundColor White

Write-Host "`n✨ Happy Testing!" -ForegroundColor Magenta
