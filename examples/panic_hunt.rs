use std::{cell::RefCell, collections::BTreeMap, panic};

use ps_parser::PowerShellSession;

thread_local! {
    static LAST: RefCell<String> = RefCell::new(String::new());
}

const TARGETED: &[&str] = &[
    "9223372036854775807 + 1",
    "-9223372036854775807 - 10",
    "9223372036854775807 * 2",
    "$a = 9223372036854775807; $a++",
    "[long]::MinValue / -1",
    "(-9223372036854775807 - 1) / -1",
    "(-9223372036854775807 - 1) % -1",
    "5 / '0.4'",
    "5 % '0.4'",
    "5 / 0.4",
    "5 % 0.4",
    "1 -shl 64",
    "1 -shl -1",
    "1 -shr 100",
    "'zażółć'.Substring(3)",
    "'zażółć'.Substring(2,2)",
    "'zażółć'.Remove(3)",
    "'abc'.Substring(-1)",
    "'abc'.Substring(1,-1)",
    "'abc'.Remove(-1)",
    "'abc'.Substring(9223372036854775807, 1)",
    "'zażółć'.PadLeft(10)",
    "'zażółć'.Trim('ż')",
    "'zażółć'.ToUpper()",
    "'zażółć'.IndexOf('ó')",
    "'zażółć'[3]",
    "'abc'[-10]",
    "@(1,2,3)[-10]",
    "@(1,2,3)[9223372036854775807]",
    "@()[0]",
    "'abc'[1..-1]",
    "'zażółć' -split 'ó'",
    "'zażółć' -replace 'ó','o'",
    "'a' -split ''",
    "'{0}{1}' -f 1",
    "'{99999999999}' -f 1",
    "'{0' -f 1",
    "'{' -f 1",
    "[char]-1",
    "[char]99999999",
    "[int]'99999999999999999999'",
    "[int]9223372036854775807",
    "[byte]300",
    "[System.Text.Encoding]::Unicode.GetString([byte[]](1,2,3))",
    "[System.Text.Encoding]::UTF8.GetString([byte[]](255,254))",
    "[Convert]::FromBase64String('abc')",
    "[Convert]::FromBase64String('====')",
    "[System.Text.Encoding]::Unicode.GetString([Convert]::FromBase64String('YQ=='))",
    "powershell -enc",
    "powershell -enc ''",
    "powershell -enc YQ==",
    "powershell -e '!!!'",
    "powershell -encodedcommand -enc",
    "& {}",
    "& {param($a)} ",
    "function f {} ; f 1 2 3",
    "$null.foo.bar()",
    "$null[0]",
    "$x = @{}; $x[$null]",
    "switch ($null) {}",
    "switch (1) { default {} default {} }",
    "1..-1",
    "'a'..'z'",
    "[char]'a'..[char]'c'",
    "\"$(\"",
    "\"$(1\"",
    "'''",
    "@'\n'@",
    "@\"\n\"@",
    "$",
    "${",
    "${}",
    "$env:",
    "-",
    "--",
    "!",
    "[",
    "[]",
    "[int]",
    "[int]::",
    "::",
    "()",
    "@(",
    "@{",
    "@{a=}",
    "{",
    "}",
    "|",
    "| x",
    "1 |",
    "a.b.c(",
    ".",
    "&",
    "&&",
    "`",
    "#",
    "<#",
    "class A { }",
    "class A { [int]$x; A() { } }",
    "class",
    "function",
    "if",
    "if ()",
    "while",
    "for (;;) { break }",
    "foreach",
    "try { } catch",
    "1 -as [int]",
    "1 -is",
    "'a' -like",
    "'a' -match '('",
    "'a' -replace '(', 'b'",
    "'a' -split '('",
    "'a' -match '\\'",
    "-join",
    "-join $null",
    "1 -bxor",
    "-not",
    "[System.Convert]::ToInt32('ff', 16)",
    "[System.Convert]::ToInt32('ff', 0)",
    "[System.Convert]::ToInt32('zz', 16)",
    "[System.Convert]::ToString(5, 0)",
    "[string]::new('a', -1)",
    "[string]::Join()",
    "'abc'.Split()",
    "'abc'.Insert(10, 'x')",
    "'abc'.Replace('', 'x')",
    "'abc'.LastIndexOf('c', 100)",
    "'abc'.IndexOf('c', 100)",
    "'{0:N70000}' -f 1.5",
    "'{0:F70000}' -f 1.5",
    "'{0:N}' -f 1.5",
    "'{0:}' -f 1.5",
    "'{0,70000}' -f 1",
    "'{0,-2147483648}' -f 1",
    "'{0,2147483647}' -f 1",
    "'{0:#,0}' -f 1",
    "'{ż}' -f 1",
    "'{0,ż}' -f 1",
    "'{0:ż}' -f 1",
    "'a'.Split('a', -9223372036854775807 - 1)",
    "'a'.Split('a', -5)",
    "@(1,2) * 0",
    "'a' * 0",
    "class B : A { }",
    "class A { [int]$x; A() { } }; [A]::new()",
    "'{0:' + '0' * 70000 + '}' -f 1",
];

// each case on a 1 MB stack (Windows main thread size), with a timeout; a stack overflow aborts the whole run
fn depth_matrix() {
    let patterns: &[(&str, &str, &str)] = &[
        ("paren", "(", ")"),
        ("block", "& {", "}"),
        ("sub", "$(", ")"),
        ("dq", "\"$(", ")\""),
        ("idx", "$a[", "]"),
        ("if", "if(1){", "}"),
        ("ifelse", "if(0){1}else{", "}"),
        ("arr", "@(", ")"),
        ("hash", "@{a=", "}"),
        ("neg", "-(", ")"),
        ("method", "'a'.Replace((", "),'b')"),
        ("plus", "+(", ")"),
        ("not", "!(", ")"),
        ("cast", "[int](", ")"),
        ("sb", "{", "}"),
        ("cmd", "echo (", ")"),
    ];
    let only = std::env::args().nth(2);
    let depths: Vec<usize> = match std::env::args().nth(3) {
        Some(d) => d.split(',').map(|d| d.parse().unwrap()).collect(),
        None => vec![5, 10, 20, 40, 60, 80, 100, 300, 1500, 5000],
    };
    for n in depths {
        println!("--- depth {n}");
        let mut cases: Vec<(String, String)> = patterns
            .iter()
            .filter(|(name, _, _)| only.as_deref().is_none_or(|o| o == "all" || o == *name))
            .map(|(name, o, c)| (name.to_string(), format!("$a=1;{}1{}", o.repeat(n), c.repeat(n))))
            .collect();
        cases.push((
            "func".into(),
            format!("function f($n) {{ if ($n -gt 0) {{ f ($n - 1) }} else {{ 'done' }} }}; f {n}"),
        ));
        for (name, script) in cases {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::Builder::new()
                .stack_size(1 << 20)
                .spawn(move || {
                    let t = std::time::Instant::now();
                    let r = PowerShellSession::new().parse_script(&script).map(|r| r.result());
                    let _ = tx.send(format!("{:?} [{} ms]", r, t.elapsed().as_millis()));
                })
                .unwrap();
            let res = rx
                .recv_timeout(std::time::Duration::from_secs(20))
                .unwrap_or_else(|_| "TIMEOUT".into());
            let short: String = res.chars().take(70).collect();
            println!("{name:>7}: {short}");
        }
    }
    std::process::exit(0);
}

fn run(script: &str) -> Option<String> {
    LAST.with(|l| l.borrow_mut().clear());
    let res = panic::catch_unwind(|| {
        let mut ps = PowerShellSession::new();
        let _ = ps.parse_script(script);
        let _ = PowerShellSession::new().parse_command(script);
    });
    // the hook fires even when the parser catches the panic internally
    let last = LAST.with(|l| l.borrow().clone());
    (res.is_err() || !last.is_empty()).then_some(last)
}

fn main() {
    panic::set_hook(Box::new(|info| {
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        LAST.with(|l| *l.borrow_mut() = format!("{loc} | {msg}"));
    }));

    if std::env::args().nth(1).as_deref() == Some("--depth") {
        depth_matrix();
        return;
    }

    if let Some(script) = std::env::args().nth(1) {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let t = std::time::Instant::now();
            let r = PowerShellSession::new().parse_script(&script).map(|r| r.result());
            let _ = tx.send(format!("[{} ms] {:?}", t.elapsed().as_millis(), r));
        });
        let res = rx
            .recv_timeout(std::time::Duration::from_secs(15))
            .unwrap_or_else(|_| "TIMEOUT".into());
        println!("{}", res.chars().take(200).collect::<String>());
        std::process::exit(0);
    }

    // location -> (count, shortest input)
    let mut found: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut record = |input: &str, panic: String| {
        let key = panic.split(" | ").next().unwrap_or_default().to_string();
        let e = found.entry(key).or_insert((0, input.to_string()));
        e.0 += 1;
        if input.len() < e.1.len() {
            e.1 = input.to_string();
        }
        let _ = panic;
    };

    let mut runs = 0;
    for s in TARGETED {
        runs += 1;
        if let Some(p) = run(s) {
            println!("TARGETED PANIC: {s:?}\n    {p}");
            record(s, p);
        }
    }

    let corpus = [
        "test_scripts/comprehensive/input.ps1",
        "test_scripts/parser_focused/input.ps1",
        "test_scripts/stress/input.ps1",
        "test_scripts/enc_oneliner.ps1",
    ];
    for path in corpus {
        let text = std::fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = text.lines().collect();

        // every single line on its own, and every line truncated at each char boundary
        for line in &lines {
            let idxs: Vec<usize> = line.char_indices().map(|(i, _)| i).chain([line.len()]).collect();
            for &i in &idxs {
                runs += 1;
                let s = &line[..i];
                if let Some(p) = run(s) {
                    record(s, p);
                }
            }
        }

        // whole file truncated at every line boundary
        for n in 0..=lines.len() {
            runs += 1;
            let s = lines[..n].join("\n");
            if let Some(p) = run(&s) {
                record(&s, p);
            }
        }
    }

    println!("\n=== {runs} runs, {} distinct panic locations ===", found.len());
    for (loc, (count, input)) in &found {
        let short: String = input.chars().take(160).collect();
        println!("{loc}  (x{count})\n    shortest: {short:?}");
    }
}
