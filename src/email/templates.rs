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
        }
    }

    pub fn subject(&self) -> String {
        match self {
            Self::Welcome { .. } => "Welcome to CVenom! 🎉".into(),
            Self::PaymentReceipt { .. } => "CVenom — Payment Confirmation".into(),
            Self::ReferralReward { credits_earned, .. } => format!("You earned {} credits!", credits_earned),
            Self::CvReady { profile, .. } => format!("Your CV for {} is ready", profile),
            Self::PortfolioReady { profile, .. } => format!("Your portfolio for {} is ready", profile),
            Self::CoverLetterReady { profile } => format!("Cover letter for {} is ready", profile),
            Self::LowCredits { balance } => format!("Low credit balance: {} remaining", balance),
            Self::AccountDeleted => "Your CVenom account has been deleted".into(),
            Self::CvImported { profile, .. } => format!("CV imported: {}", profile),
            Self::TranslationReady { target_lang, .. } => format!("Translation to {} is ready", target_lang),
            Self::AtsResults { job_title, company, .. } => format!("ATS analysis: {} at {}", job_title, company),
            Self::CreditAdjustment { amount, .. } => {
                if *amount >= 0 { format!("You received {} credits", amount) }
                else { format!("Credit adjustment: {}", amount) }
            }
            Self::BdWelcome { .. } => "Welcome to the CVenom Partner Program!".into(),
            Self::CommissionEarned { commission_dollars, .. } => format!("Commission earned: ${:.2}", commission_dollars),
            Self::CommissionPaid { total_paid, .. } => format!("Commission payout: ${:.2}", total_paid),
            Self::Nudge { credits, .. } => if *credits > 0 { format!("You have {credits} credits — create your first CV!") } else { "Create your first CV with CVenom".into() },
            Self::WinBack { .. } => "We miss you! Here's what's new on CVenom".into(),
            Self::NewTemplate { template_name } => format!("New template available: {}", template_name),
            Self::AdminNewUser { user_email, .. } => format!("[CVenom] New user: {}", user_email),
            Self::AdminActivity { user_email, action, .. } => format!("[CVenom] {} — {}", action, user_email),
        }
    }

    pub fn html_body(&self) -> String {
        let content = match self {
            // ── Tier 1 ───────────────────────────────────────────────────────
            Self::Welcome { name, credits } => format!(
                r#"<h1>Welcome to CVenom, {name}!</h1>
<p>Your account is ready. We've added <strong>{credits} free credits</strong> to get you started.</p>
<h2>Quick Start</h2>
<ul>
  <li>Create your first profile and upload your CV</li>
  <li>Generate polished PDFs with professional templates</li>
  <li>Optimize your CV for specific job postings with ATS analysis</li>
  <li>Generate tailored cover letters</li>
</ul>
<p>Each generation costs a few credits. You can always buy more when you need them.</p>"#
            ),

            Self::PaymentReceipt { amount_cents, credits_added, new_balance } => {
                let dollars = *amount_cents as f64 / 100.0;
                format!(
                    r#"<h1>Payment Confirmed</h1>
<p>Thank you for your purchase!</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Amount</td><td style="padding:4px 12px">${dollars:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Credits added</td><td style="padding:4px 12px">{credits_added}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">New balance</td><td style="padding:4px 12px">{new_balance}</td></tr>
</table>"#
                )
            }

            Self::ReferralReward { credits_earned, referral_type } => format!(
                r#"<h1>Referral Reward!</h1>
<p>You earned <strong>{credits_earned} credits</strong> from a {referral_type} referral.</p>
<p>Keep sharing your referral link to earn more!</p>"#
            ),

            Self::CvReady { profile, filename, download_url } => format!(
                r#"<h1>Your CV is Ready</h1>
<p>Your CV for <strong>{profile}</strong> has been generated.</p>
<p><a href="{download_url}" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Download {filename}</a></p>
<p style="color:#64748B;font-size:13px">This link expires in 1 hour.</p>"#
            ),

            Self::PortfolioReady { profile, filename, download_url } => format!(
                r#"<h1>Your Portfolio is Ready</h1>
<p>Your portfolio for <strong>{profile}</strong> has been generated.</p>
<p><a href="{download_url}" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Download {filename}</a></p>
<p style="color:#64748B;font-size:13px">This link expires in 1 hour.</p>"#
            ),

            Self::CoverLetterReady { profile } => format!(
                r#"<h1>Cover Letter Ready</h1>
<p>Your cover letter for <strong>{profile}</strong> has been generated.</p>
<p>You can view and download it from the CVenom editor.</p>"#
            ),

            Self::LowCredits { balance } => format!(
                r#"<h1>Low Credit Balance</h1>
<p>You have <strong>{balance} credits</strong> remaining.</p>
<p>Top up to continue generating CVs, portfolios, and cover letters without interruption.</p>
<p><a href="https://cvenom.com" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Buy Credits</a></p>"#
            ),

            Self::AccountDeleted => r#"<h1>Account Deleted</h1>
<p>Your CVenom account and all associated data have been permanently removed.</p>
<p>If this was a mistake, you can sign up again at any time — but your previous data cannot be recovered.</p>
<p>Thank you for using CVenom.</p>"#.into(),

            // ── Tier 2 ───────────────────────────────────────────────────────
            Self::CvImported { profile, lang } => format!(
                r#"<h1>CV Imported Successfully</h1>
<p>Your CV has been imported as profile <strong>{profile}</strong> (detected language: {lang}).</p>
<p>You can now edit it in the CVenom editor and generate polished PDFs.</p>
<p><a href="https://studio.cvenom.com" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Open Editor</a></p>"#
            ),

            Self::TranslationReady { profile, source_lang, target_lang } => format!(
                r#"<h1>Translation Complete</h1>
<p>Your CV for <strong>{profile}</strong> has been translated from <strong>{source_lang}</strong> to <strong>{target_lang}</strong>.</p>
<p>You can review and download the translated version from the editor.</p>"#
            ),

            Self::AtsResults { profile, job_title, company, before_score, after_score } => {
                let score_html = match (before_score, after_score) {
                    (Some(before), Some(after)) => format!(
                        r#"<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Before</td><td style="padding:4px 12px">{before}%</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">After</td><td style="padding:4px 12px">{after}%</td></tr>
</table>"#
                    ),
                    _ => String::new(),
                };
                format!(
                    r#"<h1>ATS Analysis Complete</h1>
<p>Your CV for <strong>{profile}</strong> has been analyzed against <strong>{job_title}</strong> at <strong>{company}</strong>.</p>
{score_html}
<p>Check the full analysis in the CVenom editor for keyword matches and optimization suggestions.</p>"#
                )
            }

            Self::CreditAdjustment { amount, reason, new_balance } => {
                let verb = if *amount >= 0 { "added to" } else { "removed from" };
                let abs = amount.unsigned_abs();
                format!(
                    r#"<h1>Credit Adjustment</h1>
<p><strong>{abs} credits</strong> have been {verb} your account.</p>
<p><strong>Reason:</strong> {reason}</p>
<p>Your new balance is <strong>{new_balance} credits</strong>.</p>"#
                )
            }

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
<p>Every user who signs up with your link earns you a commission on their purchases.</p>"#
                )
            }

            Self::CommissionEarned { customer_email, amount_dollars, commission_dollars } => format!(
                r#"<h1>Commission Earned!</h1>
<p>A customer ({customer_email}) made a purchase and you earned a commission.</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Purchase amount</td><td style="padding:4px 12px">${amount_dollars:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Your commission</td><td style="padding:4px 12px">${commission_dollars:.2}</td></tr>
</table>
<p>Commissions are paid out periodically. Check your dashboard for details.</p>"#
            ),

            Self::CommissionPaid { total_paid, rows } => format!(
                r#"<h1>Commission Payout</h1>
<p>Your commissions have been marked as paid.</p>
<table style="border-collapse:collapse;margin:16px 0">
  <tr><td style="padding:4px 12px;font-weight:bold">Total paid</td><td style="padding:4px 12px">${total_paid:.2}</td></tr>
  <tr><td style="padding:4px 12px;font-weight:bold">Transactions</td><td style="padding:4px 12px">{rows}</td></tr>
</table>
<p>Thank you for being a CVenom partner!</p>"#
            ),

            // ── Tier 3 ───────────────────────────────────────────────────────
            Self::Nudge { name, credits } => {
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
<p><a href="https://studio.cvenom.com" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Create Your CV</a></p>"#
                )
            },

            Self::WinBack { name } => format!(
                r#"<h1>We Miss You, {name}!</h1>
<p>It's been a while since your last visit. Your profile and data are still safe and waiting for you.</p>
<h2>What's new:</h2>
<ul>
  <li>New professional templates</li>
  <li>Improved ATS optimization</li>
  <li>AI-powered cover letter generation</li>
  <li>Portfolio generation from your projects</li>
</ul>
<p><a href="https://studio.cvenom.com" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Visit CVenom</a></p>"#
            ),

            Self::NewTemplate { template_name } => format!(
                r#"<h1>New Template Available!</h1>
<p>We've just added a new CV template: <strong>{template_name}</strong>.</p>
<p>Try it out with your existing profile — no extra setup needed.</p>
<p><a href="https://studio.cvenom.com" style="display:inline-block;padding:10px 20px;background:#6366F1;color:white;text-decoration:none;border-radius:6px">Try It Now</a></p>"#
            ),

            // ── Admin notifications ───────────────────────────────────────────
            Self::AdminNewUser { user_email, credits_granted } => format!(
                r#"<h2 style="color:#0F172A">🎉 New user signed up</h2>
<table style="border-collapse:collapse;width:100%">
  <tr><td style="padding:6px 0;color:#64748B;width:140px">Email</td><td style="padding:6px 0;font-weight:bold">{user_email}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Credits granted</td><td style="padding:6px 0">{credits_granted}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Time</td><td style="padding:6px 0">{}</td></tr>
</table>
<p style="margin-top:16px"><a href="https://studio.cvenom.com/en/admin/bd" style="display:inline-block;padding:8px 16px;background:#0F172A;color:white;text-decoration:none;border-radius:6px;font-size:13px">View in Admin</a></p>"#,
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
            ),

            Self::AdminActivity { user_email, action, detail } => format!(
                r#"<h2 style="color:#0F172A">⚡ User activity</h2>
<table style="border-collapse:collapse;width:100%">
  <tr><td style="padding:6px 0;color:#64748B;width:140px">Action</td><td style="padding:6px 0;font-weight:bold">{action}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">User</td><td style="padding:6px 0">{user_email}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Detail</td><td style="padding:6px 0">{detail}</td></tr>
  <tr><td style="padding:6px 0;color:#64748B">Time</td><td style="padding:6px 0">{}</td></tr>
</table>"#,
                chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
            ),
        };

        wrap_layout(&content)
    }
}

fn wrap_layout(content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"></head>
<body style="margin:0;padding:0;background:#F8FAFC;font-family:Arial,Helvetica,sans-serif">
<div style="max-width:600px;margin:0 auto;background:#fff;border-radius:8px;overflow:hidden;margin-top:24px;margin-bottom:24px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
  <div style="background:#0F172A;padding:24px 32px">
    <span style="color:white;font-size:22px;font-weight:bold">CVenom</span>
  </div>
  <div style="padding:32px">{content}</div>
  <div style="padding:16px 32px;background:#F8FAFC;color:#64748B;font-size:12px;text-align:center">
    CVenom — Professional CV Generator<br>
    <a href="https://cvenom.com" style="color:#6366F1">cvenom.com</a>
  </div>
</div>
</body>
</html>"#
    )
}
