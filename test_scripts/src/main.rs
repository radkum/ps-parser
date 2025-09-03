// Test runner for PS Parser
// This is a simple Rust program to test the parser with the PowerShell scripts

use std::fs;

fn main() {
    println!("PowerShell Parser Test Runner");
    println!("=============================");

    let test_scripts = [
        ("Comprehensive Test", "comprehensive_test.ps1"),
        ("Parser Focused Test", "parser_focused_test.ps1"),
        ("Stress Test", "stress_test.ps1"),
    ];

    for (name, script_path) in &test_scripts {
        println!("\n🧪 Running: {}", name);
        println!("📁 File: {}", script_path);

        if let Ok(content) = fs::read_to_string(script_path) {
            println!("✅ File loaded successfully ({} bytes)", content.len());

            match ps_parser::PowerShellSession::new().parse_input(&content) {
                Ok(result) => println!("✅ Parsing successful: {:?}", result.result()),
                Err(error) => println!("❌ Parsing failed: {:?}", error),
            }

            println!("📊 Lines: {}", content.lines().count());
            println!("📊 Characters: {}", content.chars().count());
        } else {
            println!("❌ Failed to load file: {}", script_path);
        }

        println!("{}", "-".repeat(50));
    }

    println!("\n🎯 Test Summary:");
    println!("   - Total test scripts: {}", test_scripts.len());
    println!("   - To integrate with your parser, uncomment the parsing code above");
    println!("   - Each script tests different aspects of PowerShell syntax");
}
