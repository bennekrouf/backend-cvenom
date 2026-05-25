// Integration tests: compile each Typst template to PDF using the real typst binary.
// Set TYPST_BIN env var to override the binary path (useful in CI).

use std::path::PathBuf;
use std::process::Command;

const MIN_TOML: &str = r#"
name = "Test User"
job_title = "Software Engineer"
summary = "12 years of experience in software development and architecture."
key_competencies = ["Rust", "DevOps", "System Architecture"]
sectors = ["Finance", "Public Sector", "LegalTech"]
tools = "Docker, Git, GitHub Actions, VS Code"
areas_of_expertise = ["CI/CD implementation", "Hexagonal architecture", "Team leadership"]

[[projects]]
title = "cvenom"
role = "Tech Lead"
date = "2024 – Present"
description = "AI-powered CV generator with Typst backend and multi-tenant architecture."
technologies = ["Rust", "Typst", "Next.js", "SQLite"]
highlights = ["Built multi-tenant PDF pipeline", "Integrated GPT-4 for CV optimisation"]
url = "https://cvenom.com"

[[projects]]
title = "Open Source CLI"
role = "Author"
date = "2023"
description = "Developer productivity tool written in Rust."
technologies = ["Rust", "WASM"]
highlights = ["1000+ GitHub stars"]
url = ""

[languages]
native = ["French"]
fluent = ["English"]

[skills]
"Backend" = ["Rust", "Node.js", "Java"]
"DevOps" = ["Docker", "GitHub Actions", "Kubernetes"]
"Frontend" = ["React", "TypeScript"]

[[education]]
type = "diploma"
title = "MSc Computer Science, University of Lyon"
date = "2005"

[[education]]
type = "certification"
title = "AWS Certified Solutions Architect"
date = "2022"
"#;

// Stub experiences.typ covering all function signatures used across templates.
const EXPERIENCES_STUB: &str = r#"
#import "template.typ": dated_experience, experience_details

#let get_work_experience() = {
  dated_experience(
    "Senior Software Engineer",
    date: "2020 – Present",
    company: "Acme Corp, Switzerland",
    description: "Cloud-native platform team.",
    content: [
      #experience_details("Designed and delivered microservices in Rust")
      #experience_details("Led a team of 5 engineers across two time zones")
    ]
  )
  dated_experience(
    "Software Engineer",
    date: "2015 – 2020",
    company: "Startup SA, France",
    content: [
      #experience_details("Built React/Node.js full-stack application")
    ]
  )
}

#let get_key_insights() = (
  "Experienced technical lead with 12+ years delivering complex systems",
  "Expert in Rust, Node.js, and cloud-native architectures",
  "Strong background in DevOps, CI/CD, and team coaching",
)

#let structured_experience_full(..args) = { get_work_experience() }
"#;

fn typst_bin() -> String {
    std::env::var("TYPST_BIN").unwrap_or_else(|_| "typst".to_string())
}

fn compile_template(template_name: &str) -> Result<(), String> {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let tpl_dir = templates_dir.join(template_name);

    if !tpl_dir.exists() {
        return Err(format!("template directory not found: {}", tpl_dir.display()));
    }

    let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;

    // Copy all files from the template directory
    for entry in std::fs::read_dir(&tpl_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_file() {
            std::fs::copy(entry.path(), tmp.path().join(entry.file_name()))
                .map_err(|e| e.to_string())?;
        }
    }

    // Copy shared font_config.typ
    let font_cfg = templates_dir.join("font_config.typ");
    if font_cfg.exists() {
        std::fs::copy(&font_cfg, tmp.path().join("font_config.typ"))
            .map_err(|e| e.to_string())?;
    }

    std::fs::write(tmp.path().join("cv_params.toml"), MIN_TOML).map_err(|e| e.to_string())?;
    std::fs::write(tmp.path().join("experiences.typ"), EXPERIENCES_STUB)
        .map_err(|e| e.to_string())?;

    let out = Command::new(typst_bin())
        .args(["compile", "main.typ", "output.pdf", "--input", "lang=en"])
        .current_dir(tmp.path())
        .output()
        .map_err(|e| format!("could not run typst (set TYPST_BIN if needed): {e}"))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

#[test]
fn portfolio_compiles_en() {
    compile_template("portfolio").expect("portfolio (en) failed to compile");
}

#[test]
fn portfolio_compiles_fr() {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let tpl_dir = templates_dir.join("portfolio");
    let tmp = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(&tpl_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            std::fs::copy(entry.path(), tmp.path().join(entry.file_name())).unwrap();
        }
    }
    let font_cfg = templates_dir.join("font_config.typ");
    if font_cfg.exists() { std::fs::copy(&font_cfg, tmp.path().join("font_config.typ")).unwrap(); }
    std::fs::write(tmp.path().join("cv_params.toml"), MIN_TOML).unwrap();
    // Portfolio doesn't use experiences.typ — no stub needed
    let out = Command::new(typst_bin())
        .args(["compile", "main.typ", "output.pdf", "--input", "lang=fr"])
        .current_dir(tmp.path())
        .output().expect("could not run typst");
    assert!(out.status.success(), "portfolio (fr) failed:\n{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn enterprise2_compiles_en() {
    compile_template("enterprise2").expect("enterprise2 (en) failed to compile");
}

#[test]
fn enterprise2_compiles_fr() {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let tpl_dir = templates_dir.join("enterprise2");
    let tmp = tempfile::tempdir().unwrap();

    for entry in std::fs::read_dir(&tpl_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            std::fs::copy(entry.path(), tmp.path().join(entry.file_name())).unwrap();
        }
    }
    let font_cfg = templates_dir.join("font_config.typ");
    if font_cfg.exists() {
        std::fs::copy(&font_cfg, tmp.path().join("font_config.typ")).unwrap();
    }
    std::fs::write(tmp.path().join("cv_params.toml"), MIN_TOML).unwrap();
    std::fs::write(tmp.path().join("experiences.typ"), EXPERIENCES_STUB).unwrap();

    let out = Command::new(typst_bin())
        .args(["compile", "main.typ", "output.pdf", "--input", "lang=fr"])
        .current_dir(tmp.path())
        .output()
        .expect("could not run typst");

    assert!(
        out.status.success(),
        "enterprise2 (fr) failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn enterprise2_compiles_de() {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let tpl_dir = templates_dir.join("enterprise2");
    let tmp = tempfile::tempdir().unwrap();

    for entry in std::fs::read_dir(&tpl_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            std::fs::copy(entry.path(), tmp.path().join(entry.file_name())).unwrap();
        }
    }
    let font_cfg = templates_dir.join("font_config.typ");
    if font_cfg.exists() {
        std::fs::copy(&font_cfg, tmp.path().join("font_config.typ")).unwrap();
    }
    std::fs::write(tmp.path().join("cv_params.toml"), MIN_TOML).unwrap();
    std::fs::write(tmp.path().join("experiences.typ"), EXPERIENCES_STUB).unwrap();

    let out = Command::new(typst_bin())
        .args(["compile", "main.typ", "output.pdf", "--input", "lang=de"])
        .current_dir(tmp.path())
        .output()
        .expect("could not run typst");

    assert!(
        out.status.success(),
        "enterprise2 (de) failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ── Legal template ───────────────────────────────────────────────────────────

#[test]
fn legal_compiles_en() {
    compile_template("legal").expect("legal (en) failed to compile");
}

#[test]
fn legal_compiles_fr() {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let tpl_dir = templates_dir.join("legal");
    let tmp = tempfile::tempdir().unwrap();

    for entry in std::fs::read_dir(&tpl_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            std::fs::copy(entry.path(), tmp.path().join(entry.file_name())).unwrap();
        }
    }
    let font_cfg = templates_dir.join("font_config.typ");
    if font_cfg.exists() {
        std::fs::copy(&font_cfg, tmp.path().join("font_config.typ")).unwrap();
    }
    std::fs::write(tmp.path().join("cv_params.toml"), MIN_TOML).unwrap();
    std::fs::write(tmp.path().join("experiences.typ"), EXPERIENCES_STUB).unwrap();

    let out = Command::new(typst_bin())
        .args(["compile", "main.typ", "output.pdf", "--input", "lang=fr"])
        .current_dir(tmp.path())
        .output()
        .expect("could not run typst");

    assert!(
        out.status.success(),
        "legal (fr) failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// Smoke-test every template so a change to shared font_config.typ
// or experiences_template.typ doesn't silently break other templates.
#[test]
fn all_templates_compile() {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

    let failures: Vec<String> = std::fs::read_dir(&templates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter_map(|name| {
            compile_template(&name)
                .err()
                .map(|err| format!("[{name}] {err}"))
        })
        .collect();

    assert!(
        failures.is_empty(),
        "The following templates failed to compile:\n{}",
        failures.join("\n\n")
    );
}
