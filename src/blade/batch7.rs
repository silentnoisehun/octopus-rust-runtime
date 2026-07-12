#![allow(dead_code)]
#![allow(unused_variables)]

pub struct Batch7;

impl Batch7 {
    // ═══════════════════════════════════════════════════════════
    // AST SURGERY — kód manipuláció
    // ═══════════════════════════════════════════════════════════

    /// AST refaktor: unwrap → Result konvertálás
    /// Felismeri a függvényeket, .unwrap() hívásokat, és Result-té alakítja
    pub fn ast_refactor(code: &str) -> String {
        let lines: Vec<&str> = code.lines().collect();
        let mut result = Vec::new();
        let mut in_fn = false;
        let mut fn_name = String::new();
        let mut fn_lines: Vec<String> = Vec::new();
        let mut brace_depth = 0i32;
        let mut report_fns = 0u32;
        let mut report_unwraps = 0u32;

        for line in &lines {
            let trimmed = line.trim();

            // Függvény detektálás
            if trimmed.starts_with("fn ") && trimmed.contains('(') {
                if in_fn {
                    // Előző függvény lezárása
                    result.extend(process_fn(
                        &fn_name,
                        &fn_lines,
                        &mut report_fns,
                        &mut report_unwraps,
                    ));
                }
                // Új függvény kezdése
                fn_name = trimmed.split_whitespace().nth(1).unwrap_or("?").to_string();
                // Remove generics and params
                if let Some(paren) = fn_name.find('(') {
                    fn_name.truncate(paren);
                }
                fn_lines = Vec::new();
                fn_lines.push(line.to_string());
                in_fn = true;
                brace_depth = line.chars().filter(|&c| c == '{').count() as i32
                    - line.chars().filter(|&c| c == '}').count() as i32;
                continue;
            }

            if in_fn {
                fn_lines.push(line.to_string());
                brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
                brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;
                if brace_depth <= 0 {
                    // Függvény vége
                    result.extend(process_fn(
                        &fn_name,
                        &fn_lines,
                        &mut report_fns,
                        &mut report_unwraps,
                    ));
                    in_fn = false;
                }
                continue;
            }

            result.push(line.to_string());
        }

        // Utolsó függvény
        if in_fn {
            result.extend(process_fn(
                &fn_name,
                &fn_lines,
                &mut report_fns,
                &mut report_unwraps,
            ));
        }

        format!(
            "[ast-refactor] Függvények: {report_fns} | unwrap → Result: {report_unwraps}\n{}",
            result.join("\n")
        )
    }

    /// Kód minőség analízis (magnetoscanner)
    /// Tension score: .unwrap() = 10, todo!() = 20, unsafe = 15, panic! = 30
    pub fn code_quality(code: &str) -> String {
        let lines = code.lines().count();
        let unwraps = code.matches(".unwrap()").count();
        let todos = code.matches("todo!").count();
        let unsafes = code.matches("unsafe ").count();
        let panics = code.matches("panic!").count();
        let dead_code = code.matches("#[allow(dead_code)]").count();

        let tension = (unwraps * 10 + todos * 20 + unsafes * 15 + panics * 30) as f64;

        let verdict = if tension == 0.0 {
            "✅ Tiszta"
        } else if tension < 50.0 {
            "🟡 Figyelendő"
        } else if tension < 100.0 {
            "🟠 Ráncfelvarrás kell"
        } else {
            "🔴 Kritikus"
        };

        format!(
            "[code-quality] Sorok: {lines} | Tension: {tension:.0}\n\
             ̄  .unwrap(): {unwraps} | todo!(): {todos} | unsafe: {unsafes} | panic!: {panics} | dead_code: {dead_code}\n\
             ̄  Verdict: {verdict}"
        )
    }

    // ═══════════════════════════════════════════════════════════
    // CONNECTOME — dependency graph
    // ═══════════════════════════════════════════════════════════

    /// Függőségi gráf elemzés
    /// Importok kinyerése Rust/Python/JS kódokból
    pub fn connectome(code: &str, language: &str) -> String {
        let imports: Vec<String> = match language {
            "rust" | "rs" => code
                .lines()
                .filter(|l| {
                    l.trim().starts_with("use ")
                        || l.trim().starts_with("mod ")
                        || l.trim().starts_with("pub mod ")
                })
                .map(|l| l.trim().to_string())
                .collect(),
            "python" | "py" => code
                .lines()
                .filter(|l| l.trim().starts_with("import ") || l.trim().starts_with("from "))
                .map(|l| l.trim().to_string())
                .collect(),
            "javascript" | "js" => code
                .lines()
                .filter(|l| l.trim().starts_with("import ") || l.trim().starts_with("require("))
                .map(|l| l.trim().to_string())
                .collect(),
            _ => {
                // Auto-detect: Rust
                code.lines()
                    .filter(|l| l.trim().starts_with("use ") || l.trim().starts_with("mod "))
                    .map(|l| l.trim().to_string())
                    .collect()
            }
        };

        // Hub azonosítás: melyik modult importálják a legtöbben
        let mut hub_counter: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for imp in &imports {
            let parts: Vec<&str> = imp.split_whitespace().collect();
            if let Some(&target) = parts.get(1) {
                let key = target.split("::").next().unwrap_or(target).to_string();
                *hub_counter.entry(key).or_insert(0) += 1;
            }
        }

        let mut hubs: Vec<_> = hub_counter.into_iter().collect();
        hubs.sort_by(|a, b| b.1.cmp(&a.1));

        let hub_info: String = hubs
            .iter()
            .take(5)
            .map(|(name, count)| format!("    {} ({}x)", name, count))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "[connectome] Importok: {}\n{}",
            imports.len(),
            if imports.is_empty() {
                "  (nincs)".to_string()
            } else {
                format!("  {}\n  Top hubok:\n{}", imports.join("\n  "), hub_info)
            }
        )
    }

    // ═══════════════════════════════════════════════════════════
    // DUPLICATE DETECTOR — duplikátum keresés
    // ═══════════════════════════════════════════════════════════

    /// Duplikált kód blokkok keresése
    /// Hasonló sorokat és ismétlődő mintázatokat detektál
    pub fn duplicate_detector(code: &str) -> String {
        let lines: Vec<&str> = code
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#"))
            .collect();

        // N-gram keresés (3 soros blokkok)
        let mut ngram_counts: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        for i in 0..lines.len().saturating_sub(2) {
            let block = lines[i..i + 3].join("\n");
            ngram_counts.entry(block).or_default().push(i);
        }

        // 3+ ismétlődés
        let mut duplicates: Vec<_> = ngram_counts
            .into_iter()
            .filter(|(_, positions)| positions.len() >= 3)
            .collect();
        duplicates.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        let total = duplicates.len();
        let dup_lines: usize = duplicates.iter().map(|(b, _)| b.lines().count()).sum();

        let details: String = duplicates
            .iter()
            .take(5)
            .map(|(block, positions)| {
                let preview: String = block.lines().take(2).collect::<Vec<_>>().join(" \\ ");
                format!("    {}x: \"{:.60}...\"", positions.len(), preview)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "[duplicate-detector] Duplikát blokkok: {total} | érintett sorok: {dup_lines}\n{}",
            if details.is_empty() {
                "  (nincs duplikáció)".to_string()
            } else {
                details
            }
        )
    }

    // ═══════════════════════════════════════════════════════════
    // SAFETY — OWASP biztonsági ellenőrzés
    // ═══════════════════════════════════════════════════════════

    /// Biztonsági ellenőrzés
    /// OWASP top 10 minták keresése kódban
    pub fn safety_check(code: &str, language: &str) -> String {
        let lower = code.to_lowercase();
        let mut findings: Vec<String> = Vec::new();

        // OWASP minták
        let patterns: Vec<(&str, &str, u32)> = vec![
            // SQL Injection
            (
                "sql_injection",
                "sql injection: format! vagy concat SQL lekérdezésben",
                10,
            ),
            // Command Injection
            (
                "command_injection",
                "shell injection: std::process::Command user inputdal",
                8,
            ),
            // Path Traversal
            (
                "path_traversal",
                "path traversal: user input fájlútvonalban",
                7,
            ),
            // Hardcoded secrets
            ("hardcoded_secret", "hardcoded API kulcs/token a kódban", 9),
            // Unsafe unwrap
            (
                "unsafe_unwrap",
                "unsafe .unwrap() hálózati vagy fájl műveleten",
                5,
            ),
            // XSS
            ("xss", "XSS: user input HTML-ben escape nélkül", 8),
        ];

        // Language-specific checks
        match language {
            "rust" | "rs" => {
                if lower.contains("format!(")
                    && lower.contains("select ")
                    && lower.contains("from ")
                {
                    findings.push(format!("  🔴 [SQLI] format! SQL lekérdezésben"));
                }
                if lower.contains("std::process::command")
                    && (lower.contains("user") || lower.contains("input"))
                {
                    findings.push(format!("  🟠 [CMDI] std::process::Command user inputtal"));
                }
                if lower.contains("std::fs::")
                    && (lower.contains("user") || lower.contains("input"))
                {
                    findings.push(format!("  🟡 [PATH] std::fs műveletek user inputtal"));
                }
                if lower.contains("api_key")
                    || lower.contains("api_secret")
                    || lower.contains("token")
                {
                    if lower.contains("=\"sk-") || lower.contains("=\"gsk_") {
                        findings.push(format!("  🔴 [SECRET] API kulcs a kódban"));
                    }
                }
                if lower.contains(".unwrap()")
                    && (lower.contains("http") || lower.contains("tcp") || lower.contains("file"))
                {
                    findings.push(format!("  🟡 [UNWRAP] .unwrap() hálózati/fájl műveleten"));
                }
            }
            "python" | "py" => {
                if lower.contains("execute(")
                    || (lower.contains("cur.execute")
                        && (lower.contains("+") || lower.contains("f\"")))
                {
                    findings.push(format!("  🔴 [SQLI] SQL injection veszély"));
                }
                if lower.contains("os.system") || lower.contains("subprocess.") {
                    findings.push(format!("  🟠 [CMDI] Shell hívás"));
                }
                if lower.contains("open(") && (lower.contains("user") || lower.contains("input")) {
                    findings.push(format!("  🟡 [PATH] Fájl megnyitás user inputtal"));
                }
            }
            "javascript" | "js" | "typescript" | "ts" => {
                if lower.contains("innerhtml") || lower.contains("dangerouslysetinnerhtml") {
                    findings.push(format!("  🔴 [XSS] innerHTML használat"));
                }
                if lower.contains("eval(") {
                    findings.push(format!("  🔴 [EVAL] eval() használat"));
                }
            }
            _ => {}
        }

        // Keyword-based generic checks
        if lower.contains("api_key") || lower.contains("password") || lower.contains("secret") {
            if lower.contains("=\"") && !lower.contains("env") {
                findings.push(format!("  🟡 [SECRET] Lehetséges hardcoded titok"));
            }
        }

        let severity = findings.iter().filter(|f| f.contains("🔴")).count();
        let warning = findings.iter().filter(|f| f.contains("🟠")).count();
        let info = findings.iter().filter(|f| f.contains("🟡")).count();

        format!(
            "[safety-check] OWASP: 🔴{severity} 🟠{warning} 🟡{info}\n{}",
            if findings.is_empty() {
                "  ✅ Tiszta".to_string()
            } else {
                findings.join("\n")
            }
        )
    }
}

/// Segédfüggvény: egy függvény feldolgozása (ast_refactor-hoz)
fn process_fn(
    name: &str,
    lines: &[String],
    report_fns: &mut u32,
    report_unwraps: &mut u32,
) -> Vec<String> {
    let code = lines.join("\n");
    let unwrap_count = code.matches(".unwrap()").count() as u32;

    if name == "main" || name.starts_with("test_") || code.contains("Result<") || unwrap_count == 0
    {
        return lines.to_vec();
    }

    *report_fns += 1;
    *report_unwraps += unwrap_count;

    // Szöveges refaktor: .unwrap() → ?
    let result: Vec<String> = lines
        .iter()
        .map(|l| {
            if l.contains(".unwrap()") {
                l.replace(".unwrap()", "?")
            } else {
                l.to_string()
            }
        })
        .collect();

    // Ha nincs return type, adjunk hozzá
    let mut modified = false;
    let mut output = Vec::new();
    for (i, line) in result.iter().enumerate() {
        if !modified && line.contains("fn ") && line.contains("(") && !line.contains("->") {
            // Insert return type after the opening brace
            let new_line = format!("{} -> Result<(), Box<dyn std::error::Error>>", line);
            // But only if there are actually ? operators
            let has_question = result.iter().any(|l| l.contains('?'));
            if has_question {
                output.push(new_line);
                modified = true;
                continue;
            }
        }
        output.push(line.clone());
    }

    output
}
