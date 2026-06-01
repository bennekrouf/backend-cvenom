pub enum EmailKind {
    // ── Tier 1 ───────────────────────────────────────────────────────────────
    Welcome { name: String, credits: i64 },
    PaymentReceipt { amount_cents: i64, credits_added: i64, new_balance: i64 },
    ReferralReward { credits_earned: i64, referral_type: String },
    CvReady { profile: String, filename: String, download_url: String },
    PortfolioReady { profile: String, filename: String, download_url: String },
    CoverLetterReady { profile: String },
    LowCredits { balance: i64 },
    AccountDeleted,
    // ── Tier 2 ───────────────────────────────────────────────────────────────
    CvImported { profile: String, lang: String },
    TranslationReady { profile: String, source_lang: String, target_lang: String },
    AtsResults { profile: String, job_title: String, company: String, before_score: Option<u8>, after_score: Option<u8> },
    CreditAdjustment { amount: i64, reason: String, new_balance: i64 },
    BdWelcome { name: String, referral_code: String, commission_rate: f64 },
    CommissionEarned { customer_email: String, amount_dollars: f64, commission_dollars: f64 },
    CommissionPaid { total_paid: f64, rows: u64 },
    // ── Tier 3 ───────────────────────────────────────────────────────────────
    Nudge { name: String, credits: i64 },
    WinBack { name: String },
    NewTemplate { template_name: String },
    // ── Admin notifications ───────────────────────────────────────────────────
    AdminNewUser { user_email: String, credits_granted: i64 },
    AdminActivity { user_email: String, action: String, detail: String },
    AdminCvImportFailed {
        user_email: String,
        filename: String,
        error_summary: String,
        saved_path: String,
    },
}

impl EmailKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Welcome { .. } => "welcome",
            Self::PaymentReceipt { .. } => "payment_receipt",
            Self::ReferralReward { .. } => "referral_reward",
            Self::CvReady { .. } => "cv_ready",
            Self::PortfolioReady { .. } => "portfolio_ready",
            Self::CoverLetterReady { .. } => "cover_letter_ready",
            Self::LowCredits { .. } => "low_credits",
            Self::AccountDeleted => "account_deleted",
            Self::CvImported { .. } => "cv_imported",
            Self::TranslationReady { .. } => "translation_ready",
            Self::AtsResults { .. } => "ats_results",
            Self::CreditAdjustment { .. } => "credit_adjustment",
            Self::BdWelcome { .. } => "bd_welcome",
            Self::CommissionEarned { .. } => "commission_earned",
            Self::CommissionPaid { .. } => "commission_paid",
            Self::Nudge { .. } => "nudge",
            Self::WinBack { .. } => "win_back",
            Self::NewTemplate { .. } => "new_template",
            Self::AdminNewUser { .. } => "admin_new_user",
            Self::AdminActivity { .. } => "admin_activity",
            Self::AdminCvImportFailed { .. } => "admin_cv_import_failed",
        }
    }

    /// Whether this email category can be disabled by the user.
    /// Transactional emails (welcome, payment, account) and admin emails are always sent.
    pub fn is_optional(&self) -> bool {
        matches!(
            self,
            Self::CvReady { .. }
                | Self::PortfolioReady { .. }
                | Self::CoverLetterReady { .. }
                | Self::CvImported { .. }
                | Self::TranslationReady { .. }
                | Self::AtsResults { .. }
                | Self::Nudge { .. }
                | Self::WinBack { .. }
                | Self::NewTemplate { .. }
        )
    }

    pub fn subject(&self, lang: &str) -> String {
        match self {
            Self::Welcome { .. } => match lang {
                "fr" => "Bienvenue sur CVenom ! 🎉".into(),
                "de" => "Willkommen bei CVenom! 🎉".into(),
                _ => "Welcome to CVenom! 🎉".into(),
            },
            Self::PaymentReceipt { .. } => match lang {
                "fr" => "CVenom — Confirmation de paiement".into(),
                "de" => "CVenom — Zahlungsbestätigung".into(),
                _ => "CVenom — Payment Confirmation".into(),
            },
            Self::ReferralReward { credits_earned, .. } => match lang {
                "fr" => format!("Vous avez gagné {} crédits !", credits_earned),
                "de" => format!("Sie haben {} Credits verdient!", credits_earned),
                _ => format!("You earned {} credits!", credits_earned),
            },
            Self::CvReady { profile, .. } => match lang {
                "fr" => format!("Votre CV pour {} est prêt", profile),
                "de" => format!("Ihr CV für {} ist fertig", profile),
                _ => format!("Your CV for {} is ready", profile),
            },
            Self::PortfolioReady { profile, .. } => match lang {
                "fr" => format!("Votre portfolio pour {} est prêt", profile),
                "de" => format!("Ihr Portfolio für {} ist fertig", profile),
                _ => format!("Your portfolio for {} is ready", profile),
            },
            Self::CoverLetterReady { profile } => match lang {
                "fr" => format!("Lettre de motivation pour {} prête", profile),
                "de" => format!("Anschreiben für {} ist fertig", profile),
                _ => format!("Cover letter for {} is ready", profile),
            },
            Self::LowCredits { balance } => match lang {
                "fr" => format!("Solde faible : {} crédits restants", balance),
                "de" => format!("Niedriges Guthaben: {} verbleibend", balance),
                _ => format!("Low credit balance: {} remaining", balance),
            },
            Self::AccountDeleted => match lang {
                "fr" => "Votre compte CVenom a été supprimé".into(),
                "de" => "Ihr CVenom-Konto wurde gelöscht".into(),
                _ => "Your CVenom account has been deleted".into(),
            },
            Self::CvImported { profile, .. } => match lang {
                "fr" => format!("CV importé : {}", profile),
                "de" => format!("CV importiert: {}", profile),
                _ => format!("CV imported: {}", profile),
            },
            Self::TranslationReady { target_lang, .. } => match lang {
                "fr" => format!("Traduction en {} terminée", target_lang),
                "de" => format!("Übersetzung nach {} fertig", target_lang),
                _ => format!("Translation to {} is ready", target_lang),
            },
            Self::AtsResults { job_title, company, .. } => match lang {
                "fr" => format!("Analyse ATS : {} chez {}", job_title, company),
                "de" => format!("ATS-Analyse: {} bei {}", job_title, company),
                _ => format!("ATS analysis: {} at {}", job_title, company),
            },
            Self::CreditAdjustment { amount, .. } => match lang {
                "fr" => if *amount >= 0 { format!("Vous avez reçu {} crédits", amount) }
                        else { format!("Ajustement de crédits : {}", amount) },
                "de" => if *amount >= 0 { format!("Sie haben {} Credits erhalten", amount) }
                        else { format!("Credit-Anpassung: {}", amount) },
                _ => if *amount >= 0 { format!("You received {} credits", amount) }
                     else { format!("Credit adjustment: {}", amount) },
            },
            Self::BdWelcome { .. } => match lang {
                "fr" => "Bienvenue dans le programme partenaire CVenom !".into(),
                _ => "Welcome to the CVenom Partner Program!".into(),
            },
            Self::CommissionEarned { commission_dollars, .. } => format!("Commission earned: ${:.2}", commission_dollars),
            Self::CommissionPaid { total_paid, .. } => format!("Commission payout: ${:.2}", total_paid),
            Self::Nudge { credits, .. } => match lang {
                "fr" => if *credits > 0 { format!("Vous avez {credits} crédits — créez votre premier CV !") }
                        else { "Créez votre premier CV avec CVenom".into() },
                "de" => if *credits > 0 { format!("Sie haben {credits} Credits — erstellen Sie Ihren ersten CV!") }
                        else { "Erstellen Sie Ihren ersten CV mit CVenom".into() },
                _ => if *credits > 0 { format!("You have {credits} credits — create your first CV!") }
                     else { "Create your first CV with CVenom".into() },
            },
            Self::WinBack { .. } => match lang {
                "fr" => "Vous nous manquez ! Découvrez les nouveautés CVenom".into(),
                "de" => "Wir vermissen Sie! Entdecken Sie die Neuheiten bei CVenom".into(),
                _ => "We miss you! Here's what's new on CVenom".into(),
            },
            Self::NewTemplate { template_name } => match lang {
                "fr" => format!("Nouveau template disponible : {}", template_name),
                "de" => format!("Neue Vorlage verfügbar: {}", template_name),
                _ => format!("New template available: {}", template_name),
            },
            // Admin emails — always English
            Self::AdminNewUser { user_email, .. } => format!("[CVenom] New user: {}", user_email),
            Self::AdminActivity { user_email, action, .. } => format!("[CVenom] {} — {}", action, user_email),
            Self::AdminCvImportFailed { user_email, filename, .. } => {
                format!("[CVenom] CV import failed — {} ({})", user_email, filename)
            }
        }
    }

    pub fn html_body(&self, lang: &str) -> String {
        let btn = |url: &str, label: &str| -> String {
            format!(r#"<a href="{url}" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">{label}</a>"#)
        };
        let open_editor = || match lang {
            "fr" => btn("https://studio.cvenom.com", "Ouvrir l'éditeur"),
            "de" => btn("https://studio.cvenom.com", "Editor öffnen"),
            _ => btn("https://studio.cvenom.com", "Open Editor"),
        };

        let content = match self {
            // ── Tier 1 ───────────────────────────────────────────────────────
            Self::Welcome { name, credits } => match lang {
                "fr" => format!(
                    r#"<h1>Bienvenue sur CVenom, {name} !</h1>
<p>Votre compte est prêt. Nous avons ajouté <strong>{credits} crédits gratuits</strong> pour commencer.</p>
<h2>Démarrage rapide</h2>
<ul>
  <li>Créez votre premier profil et importez votre CV</li>
  <li>Générez des PDF professionnels avec nos templates</li>
  <li>Optimisez votre CV pour des offres spécifiques avec l'analyse ATS</li>
  <li>Générez des lettres de motivation personnalisées</li>
</ul>
<p>Chaque génération coûte quelques crédits. Vous pouvez en acheter davantage à tout moment.</p>"#),
                "de" => format!(
                    r#"<h1>Willkommen bei CVenom, {name}!</h1>
<p>Ihr Konto ist bereit. Wir haben <strong>{credits} kostenlose Credits</strong> hinzugefügt.</p>
<h2>Schnellstart</h2>
<ul>
  <li>Erstellen Sie Ihr erstes Profil und laden Sie Ihren CV hoch</li>
  <li>Generieren Sie professionelle PDFs mit unseren Vorlagen</li>
  <li>Optimieren Sie Ihren CV für bestimmte Stellenangebote mit ATS-Analyse</li>
  <li>Generieren Sie maßgeschneiderte Anschreiben</li>
</ul>
<p>Jede Generierung kostet ein paar Credits. Sie können jederzeit mehr kaufen.</p>"#),
                _ => format!(
                    r#"<h1>Welcome to CVenom, {name}!</h1>
<p>Your account is ready. We've added <strong>{credits} free credits</strong> to get you started.</p>
<h2>Quick Start</h2>
<ul>
  <li>Create your first profile and upload your CV</li>
  <li>Generate polished PDFs with professional templates</li>
  <li>Optimize your CV for specific job postings with ATS analysis</li>
  <li>Generate tailored cover letters</li>
</ul>
<p>Each generation costs a few credits. You can always buy more when you need them.</p>"#),
            },

            Self::PaymentReceipt { amount_cents, credits_added, new_balance } => {
                let dollars = *amount_cents as f64 / 100.0;
                match lang {
                    "fr" => format!(
                        r#"<h1>Paiement confirmé</h1>
<p>Merci pour votre achat !</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Montant</td><td style="padding:4px 12px">${dollars:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Crédits ajoutés</td><td style="padding:4px 12px">{credits_added}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Nouveau solde</td><td style="padding:4px 12px">{new_balance}</td></tr>
</table>"#),
                    _ => format!(
                        r#"<h1>Payment Confirmed</h1>
<p>Thank you for your purchase!</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Amount</td><td style="padding:4px 12px">${dollars:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Credits added</td><td style="padding:4px 12px">{credits_added}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">New balance</td><td style="padding:4px 12px">{new_balance}</td></tr>
</table>"#),
                }
            }

            Self::ReferralReward { credits_earned, referral_type } => match lang {
                "fr" => format!(
                    r#"<h1>Récompense de parrainage !</h1>
<p>Vous avez gagné <strong>{credits_earned} crédits</strong> grâce à un parrainage {referral_type}.</p>
<p>Continuez à partager votre lien pour gagner davantage !</p>"#),
                _ => format!(
                    r#"<h1>Referral Reward!</h1>
<p>You earned <strong>{credits_earned} credits</strong> from a {referral_type} referral.</p>
<p>Keep sharing your referral link to earn more!</p>"#),
            },

            Self::CvReady { profile, filename, download_url } => match lang {
                "fr" => format!(
                    r#"<h1>Votre CV est prêt</h1>
<p>Votre CV pour <strong>{profile}</strong> a été généré.</p>
<p>{}</p>
<p style="color:#64748B;font-size:13px">Ce lien expire dans 1 heure.</p>"#, btn(download_url, &format!("Télécharger {filename}"))),
                "de" => format!(
                    r#"<h1>Ihr CV ist fertig</h1>
<p>Ihr CV für <strong>{profile}</strong> wurde erstellt.</p>
<p>{}</p>
<p style="color:#64748B;font-size:13px">Dieser Link läuft in 1 Stunde ab.</p>"#, btn(download_url, &format!("Herunterladen {filename}"))),
                _ => format!(
                    r#"<h1>Your CV is Ready</h1>
<p>Your CV for <strong>{profile}</strong> has been generated.</p>
<p>{}</p>
<p style="color:#64748B;font-size:13px">This link expires in 1 hour.</p>"#, btn(download_url, &format!("Download {filename}"))),
            },

            Self::PortfolioReady { profile, filename, download_url } => match lang {
                "fr" => format!(
                    r#"<h1>Votre portfolio est prêt</h1>
<p>Votre portfolio pour <strong>{profile}</strong> a été généré.</p>
<p>{}</p>
<p style="color:#64748B;font-size:13px">Ce lien expire dans 1 heure.</p>"#, btn(download_url, &format!("Télécharger {filename}"))),
                _ => format!(
                    r#"<h1>Your Portfolio is Ready</h1>
<p>Your portfolio for <strong>{profile}</strong> has been generated.</p>
<p>{}</p>
<p style="color:#64748B;font-size:13px">This link expires in 1 hour.</p>"#, btn(download_url, &format!("Download {filename}"))),
            },

            Self::CoverLetterReady { profile } => match lang {
                "fr" => format!(
                    r#"<h1>Lettre de motivation prête</h1>
<p>Votre lettre de motivation pour <strong>{profile}</strong> a été générée.</p>
<p>Vous pouvez la consulter et la télécharger depuis l'éditeur CVenom.</p>"#),
                _ => format!(
                    r#"<h1>Cover Letter Ready</h1>
<p>Your cover letter for <strong>{profile}</strong> has been generated.</p>
<p>You can view and download it from the CVenom editor.</p>"#),
            },

            Self::LowCredits { balance } => match lang {
                "fr" => format!(
                    r#"<h1>Solde de crédits faible</h1>
<p>Il vous reste <strong>{balance} crédits</strong>.</p>
<p>Rechargez pour continuer à générer des CV, portfolios et lettres de motivation.</p>
<p>{}</p>"#, btn("https://cvenom.com", "Acheter des crédits")),
                _ => format!(
                    r#"<h1>Low Credit Balance</h1>
<p>You have <strong>{balance} credits</strong> remaining.</p>
<p>Top up to continue generating CVs, portfolios, and cover letters without interruption.</p>
<p>{}</p>"#, btn("https://cvenom.com", "Buy Credits")),
            },

            Self::AccountDeleted => match lang {
                "fr" => r#"<h1>Compte supprimé</h1>
<p>Votre compte CVenom et toutes les données associées ont été définitivement supprimés.</p>
<p>Si c'est une erreur, vous pouvez vous réinscrire à tout moment — mais vos données précédentes ne pourront pas être récupérées.</p>
<p>Merci d'avoir utilisé CVenom.</p>"#.into(),
                _ => r#"<h1>Account Deleted</h1>
<p>Your CVenom account and all associated data have been permanently removed.</p>
<p>If this was a mistake, you can sign up again at any time — but your previous data cannot be recovered.</p>
<p>Thank you for using CVenom.</p>"#.into(),
            },

            // ── Tier 2 ───────────────────────────────────────────────────────
            Self::CvImported { profile, lang: detected_lang } => match lang {
                "fr" => format!(
                    r#"<h1>CV importé avec succès</h1>
<p>Votre CV a été importé sous le profil <strong>{profile}</strong> (langue détectée : {detected_lang}).</p>
<p>Vous pouvez maintenant le modifier dans l'éditeur et générer des PDF professionnels.</p>
<p>{}</p>"#, open_editor()),
                _ => format!(
                    r#"<h1>CV Imported Successfully</h1>
<p>Your CV has been imported as profile <strong>{profile}</strong> (detected language: {detected_lang}).</p>
<p>You can now edit it in the CVenom editor and generate polished PDFs.</p>
<p>{}</p>"#, open_editor()),
            },

            Self::TranslationReady { profile, source_lang, target_lang } => match lang {
                "fr" => format!(
                    r#"<h1>Traduction terminée</h1>
<p>Votre CV pour <strong>{profile}</strong> a été traduit de <strong>{source_lang}</strong> vers <strong>{target_lang}</strong>.</p>
<p>Vous pouvez consulter et télécharger la version traduite depuis l'éditeur.</p>"#),
                _ => format!(
                    r#"<h1>Translation Complete</h1>
<p>Your CV for <strong>{profile}</strong> has been translated from <strong>{source_lang}</strong> to <strong>{target_lang}</strong>.</p>
<p>You can review and download the translated version from the editor.</p>"#),
            },

            Self::AtsResults { profile, job_title, company, before_score, after_score } => {
                let score_html = match (before_score, after_score) {
                    (Some(before), Some(after)) => {
                        let (lbl_before, lbl_after) = match lang {
                            "fr" => ("Avant", "Après"),
                            "de" => ("Vorher", "Nachher"),
                            _ => ("Before", "After"),
                        };
                        format!(
                            r#"<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">{lbl_before}</td><td style="padding:4px 12px">{before}%</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">{lbl_after}</td><td style="padding:4px 12px">{after}%</td></tr>
</table>"#)
                    },
                    _ => String::new(),
                };
                match lang {
                    "fr" => format!(
                        r#"<h1>Analyse ATS terminée</h1>
<p>Votre CV pour <strong>{profile}</strong> a été analysé pour le poste <strong>{job_title}</strong> chez <strong>{company}</strong>.</p>
{score_html}
<p>Consultez l'analyse complète dans l'éditeur CVenom pour les correspondances de mots-clés et les suggestions d'optimisation.</p>"#),
                    _ => format!(
                        r#"<h1>ATS Analysis Complete</h1>
<p>Your CV for <strong>{profile}</strong> has been analyzed against <strong>{job_title}</strong> at <strong>{company}</strong>.</p>
{score_html}
<p>Check the full analysis in the CVenom editor for keyword matches and optimization suggestions.</p>"#),
                }
            }

            Self::CreditAdjustment { amount, reason, new_balance } => match lang {
                "fr" => {
                    let verb = if *amount >= 0 { "ajoutés à" } else { "retirés de" };
                    let abs = amount.unsigned_abs();
                    format!(
                        r#"<h1>Ajustement de crédits</h1>
<p><strong>{abs} crédits</strong> ont été {verb} votre compte.</p>
<p><strong>Raison :</strong> {reason}</p>
<p>Votre nouveau solde est de <strong>{new_balance} crédits</strong>.</p>"#)
                }
                _ => {
                    let verb = if *amount >= 0 { "added to" } else { "removed from" };
                    let abs = amount.unsigned_abs();
                    format!(
                        r#"<h1>Credit Adjustment</h1>
<p><strong>{abs} credits</strong> have been {verb} your account.</p>
<p><strong>Reason:</strong> {reason}</p>
<p>Your new balance is <strong>{new_balance} credits</strong>.</p>"#)
                }
            },

            // BD / Commission emails — English only (B2B context)
            Self::BdWelcome { name, referral_code, commission_rate } => {
                let pct = (*commission_rate * 100.0) as u32;
                format!(
                    r#"<h1>Welcome to the Partner Program, {name}!</h1>
<p>You're now registered as a CVenom Business Developer.</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Referral code</td><td style="padding:4px 12px"><code>{referral_code}</code></td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Commission rate</td><td style="padding:4px 12px">{pct}%</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Share link</td><td style="padding:4px 12px"><a href="https://cvenom.com?ref={referral_code}">cvenom.com?ref={referral_code}</a></td></tr>
</table>
<p>Every user who signs up with your link earns you a commission on their purchases.</p>"#)
            }

            Self::CommissionEarned { customer_email, amount_dollars, commission_dollars } => format!(
                r#"<h1>Commission Earned!</h1>
<p>A customer ({customer_email}) made a purchase and you earned a commission.</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Purchase amount</td><td style="padding:4px 12px">${amount_dollars:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Your commission</td><td style="padding:4px 12px">${commission_dollars:.2}</td></tr>
</table>
<p>Commissions are paid out periodically. Check your dashboard for details.</p>"#),

            Self::CommissionPaid { total_paid, rows } => format!(
                r#"<h1>Commission Payout</h1>
<p>Your commissions have been marked as paid.</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Total paid</td><td style="padding:4px 12px">${total_paid:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Transactions</td><td style="padding:4px 12px">{rows}</td></tr>
</table>
<p>Thank you for being a CVenom partner!</p>"#),

            // ── Tier 3 ───────────────────────────────────────────────────────
            Self::Nudge { name, credits } => match lang {
                "fr" => {
                    let credits_line = if *credits > 0 {
                        format!("<p>Vous avez encore <strong>{credits} crédits</strong> — c'est suffisant pour plusieurs générations.</p>")
                    } else {
                        "<p>Votre première génération de CV n'est qu'à quelques clics.</p>".to_string()
                    };
                    format!(
                        r#"<h1>Votre CV vous attend, {name} !</h1>
<p>Vous vous êtes inscrit récemment mais n'avez pas encore généré votre premier CV.</p>
{credits_line}
<h2>Commencez en 3 étapes :</h2>
<ol>
  <li>Importez votre CV existant ou saisissez vos informations</li>
  <li>Choisissez un template professionnel</li>
  <li>Générez et téléchargez votre PDF</li>
</ol>
<p>{}</p>"#, btn("https://studio.cvenom.com", "Créer votre CV"))
                }
                _ => {
                    let credits_line = if *credits > 0 {
                        format!("<p>You still have <strong>{credits} credits</strong> — that's enough for several CV generations.</p>")
                    } else {
                        "<p>Your first CV generation is just a few clicks away.</p>".to_string()
                    };
                    format!(
                        r#"<h1>Your CV is Waiting, {name}!</h1>
<p>You signed up recently but haven't generated your first CV yet.</p>
{credits_line}
<h2>Get started in 3 steps:</h2>
<ol>
  <li>Upload your existing CV or paste your information</li>
  <li>Choose a professional template</li>
  <li>Generate and download your polished PDF</li>
</ol>
<p>{}</p>"#, btn("https://studio.cvenom.com", "Create Your CV"))
                }
            },

            Self::WinBack { name } => match lang {
                "fr" => format!(
                    r#"<h1>Vous nous manquez, {name} !</h1>
<p>Cela fait un moment que vous n'êtes pas venu. Votre profil et vos données sont toujours là.</p>
<h2>Quoi de neuf :</h2>
<ul>
  <li>Nouveaux templates professionnels</li>
  <li>Optimisation ATS améliorée</li>
  <li>Génération de lettres de motivation par IA</li>
  <li>Génération de portfolio à partir de vos projets</li>
</ul>
<p>{}</p>"#, btn("https://studio.cvenom.com", "Revenir sur CVenom")),
                _ => format!(
                    r#"<h1>We Miss You, {name}!</h1>
<p>It's been a while since your last visit. Your profile and data are still safe and waiting for you.</p>
<h2>What's new:</h2>
<ul>
  <li>New professional templates</li>
  <li>Improved ATS optimization</li>
  <li>AI-powered cover letter generation</li>
  <li>Portfolio generation from your projects</li>
</ul>
<p>{}</p>"#, btn("https://studio.cvenom.com", "Visit CVenom")),
            },

            Self::NewTemplate { template_name } => match lang {
                "fr" => format!(
                    r#"<h1>Nouveau template disponible !</h1>
<p>Nous venons d'ajouter un nouveau template de CV : <strong>{template_name}</strong>.</p>
<p>Essayez-le avec votre profil existant — aucune configuration supplémentaire nécessaire.</p>
<p>{}</p>"#, btn("https://studio.cvenom.com", "Essayer maintenant")),
                _ => format!(
                    r#"<h1>New Template Available!</h1>
<p>We've just added a new CV template: <strong>{template_name}</strong>.</p>
<p>Try it out with your existing profile — no extra setup needed.</p>
<p>{}</p>"#, btn("https://studio.cvenom.com", "Try It Now")),
            },

            // ── Admin notifications (always English) ─────────────────────────
            Self::AdminNewUser { user_email, credits_granted } => format!(
                r#"<h2 style="color:#0F172A">🎉 New user signed up</h2>
<table style="border-collapse:collapse;width:100%">
  <tr><td style="padding:6px 0;color:#64748B;width:140px">Email</td><td style="padding:6px 0;font-weight:bold">{user_email}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Credits granted</td><td style="padding:6px 0">{credits_granted}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Time</td><td style="padding:6px 0">{}</td></tr>
</table>
<p style="margin-top:16px"><a href="https://studio.cvenom.com/en/admin/bd" style="display:inline-block;padding:8px 16px;background:#0F172A;color:white;text-decoration:none;border-radius:6px;font-size:13px">View in Admin</a></p>"#,
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")),

            Self::AdminActivity { user_email, action, detail } => format!(
                r#"<h2 style="color:#0F172A">⚡ User activity</h2>
<table style="border-collapse:collapse;width:100%">
  <tr><td style="padding:6px 0;color:#64748B;width:140px">Action</td><td style="padding:6px 0;font-weight:bold">{action}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">User</td><td style="padding:6px 0">{user_email}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Detail</td><td style="padding:6px 0">{detail}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Time</td><td style="padding:6px 0">{}</td></tr>
</table>"#,
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")),

            Self::AdminCvImportFailed { user_email, filename, error_summary, saved_path } => {
                let escaped_error = error_summary
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                format!(
                    r#"<h2 style="color:#B91C1C">🚨 CV import failed</h2>
<p>A user tried to import a CV but the conversion failed. The uploaded file has been preserved on the server for debugging.</p>
<table style="border-collapse:collapse;width:100%">
  <tr><td style="padding:6px 0;color:#64748B;width:140px">User</td><td style="padding:6px 0;font-weight:bold">{user_email}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Filename</td><td style="padding:6px 0">{filename}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Saved path</td><td style="padding:6px 0"><code>{saved_path}</code></td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Time</td><td style="padding:6px 0">{}</td></tr>
</table>
<h3 style="color:#0F172A;margin-top:24px">Error</h3>
<pre style="background:#F8FAFC;padding:12px;border-radius:6px;font-size:12px;white-space:pre-wrap;word-break:break-word">{escaped_error}</pre>
<p style="color:#64748B;font-size:12px;margin-top:16px">SSH into the backend host and retrieve the file from the saved path above to reproduce the issue locally.</p>"#,
                    chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"))
            }
        };

        wrap_layout(&content, lang)
    }
}

fn wrap_layout(content: &str, lang: &str) -> String {
    let tagline = match lang {
        "fr" => "CVenom — Générateur de CV professionnel",
        "de" => "CVenom — Professioneller CV-Generator",
        _ => "CVenom — Professional CV Generator",
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"></head>
<body style="margin:0;padding:0;background:#F8FAFC;font-family:Arial,Helvetica,sans-serif">
<div style="max-width:600px;margin:0 auto;background:#fff;border-radius:8px;overflow:hidden;margin-top:24px;margin-bottom:24px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
  <div style="background:#0F172A;padding:24px 32px">
    <span style="color:white;font-size:22px;font-weight:bold">CVenom</span>
  </div>
  <div style="padding:32px">{content}</div>
  <div style="padding:16px 32px;background:#F8FAFC;color:#64748B;font-size:12px;text-align:center">
    {tagline}<br>
    <a href="https://cvenom.com" style="color:#6366F1">cvenom.com</a>
  </div>
</div>
</body>
</html>"#
    )
}
