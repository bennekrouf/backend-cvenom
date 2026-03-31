// src/web/handlers/cv_handlers/cover_letter_export.rs
//! Cover letter export handler — converts plain text cover letter to .docx
//!
//!   POST /cover-letter/export
//!   Body: { cover_letter, name, lang }
//!   → Returns a formatted .docx binary. No credit cost (format conversion only).

use crate::auth::AuthenticatedUser;
use crate::web::types::{DocxResponse, StandardErrorResponse};
use docx_rs::*;
use graflog::app_log;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use crate::web::ServerConfig;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CoverLetterExportRequest {
    /// The plain-text cover letter to export.
    pub cover_letter: String,
    /// Candidate name — used in the filename.
    pub name: String,
    /// Language code ("en" or "fr").
    pub lang: String,
}

pub async fn cover_letter_export_handler(
    request: Json<CoverLetterExportRequest>,
    _auth: AuthenticatedUser,
    _config: &State<ServerConfig>,
) -> Result<DocxResponse, Json<StandardErrorResponse>> {
    let data = &request.0;

    app_log!(info, "Generating .docx cover letter for '{}'", data.name);

    let docx_bytes = build_cover_letter_docx(&data.cover_letter, &data.name)
        .map_err(|e| {
            app_log!(error, "DOCX generation failed: {}", e);
            Json(StandardErrorResponse::new(
                format!("DOCX generation failed: {}", e),
                "DOCX_GENERATION_ERROR".to_string(),
                vec!["Try again or use the copy button".to_string()],
                None,
            ))
        })?;

    let safe_name = data.name.replace(' ', "_").to_lowercase();
    let filename = format!("cover_letter_{}_{}.docx", safe_name, data.lang);

    Ok(DocxResponse::new(docx_bytes, filename))
}

/// Build a professionally formatted .docx from plain cover letter text.
///
/// The text is split on blank lines (double `\n`) to create paragraphs.
/// Each paragraph gets comfortable spacing and a readable Calibri font.
fn build_cover_letter_docx(text: &str, _name: &str) -> anyhow::Result<Vec<u8>> {
    // Split text into paragraphs on blank lines; fall back to single-line splits.
    let raw_paragraphs: Vec<&str> = text.split("\n\n").collect();
    let paragraphs: Vec<String> = raw_paragraphs
        .iter()
        .flat_map(|block| {
            let trimmed = block.trim();
            if trimmed.is_empty() {
                vec![]
            } else {
                // Within a block, join lines with a space (soft line breaks become spaces)
                vec![trimmed.replace('\n', " ")]
            }
        })
        .collect();

    let mut doc = Docx::new()
        // Page margins: 2.5 cm all sides (in twentieths of a point: 1 cm ≈ 567 twips)
        .page_margin(
            PageMargin::new()
                .top(1418)    // ~2.5 cm
                .bottom(1418)
                .left(1701)   // ~3 cm
                .right(1701),
        );

    for (i, para_text) in paragraphs.iter().enumerate() {
        let is_first = i == 0;

        let run = Run::new()
            .add_text(para_text.clone())
            .fonts(RunFonts::new().ascii("Calibri").hi_ansi("Calibri"))
            .size(24); // 24 half-points = 12pt

        let line_val = if is_first { 360i32 } else { 276i32 };

        let paragraph = Paragraph::new()
            .add_run(run)
            .line_spacing(LineSpacing::new().line(line_val).after(160));

        doc = doc.add_paragraph(paragraph);
    }

    let mut buf = Vec::new();
    doc.build().pack(&mut std::io::Cursor::new(&mut buf))?;
    Ok(buf)
}
