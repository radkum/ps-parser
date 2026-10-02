use ps_parser::PowerShellSession;

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: cargo run --example enc_oneliner -- <script.ps1>");
        std::process::exit(1);
    };
    let script = std::fs::read_to_string(path).unwrap();

    println!("=== parse_command ===");
    match PowerShellSession::new().parse_command(script.trim()) {
        Ok(res) => println!("deob:\n{}\nout:\n{}", res.deobfuscated(), res.output()),
        Err(e) => println!("error: {e:?}"),
    }

    println!("=== parse_script ===");
    match PowerShellSession::new().parse_script(script.trim()) {
        Ok(res) => println!("deob:\n{}\nout:\n{}", res.deobfuscated(), res.output()),
        Err(e) => println!("error: {e:?}"),
    }
}
