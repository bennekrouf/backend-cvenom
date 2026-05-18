pub enum EmailKind {
    Welcome { name: String, credits: i64 },
    PaymentReceipt { amount_cents: i64, credits_added: i64, new_balance: i64 },
    ReferralReward { credits_earned: i64, referral_type: String },
    CvReady { profile: String, filename: String, download_url: String },
    PortfolioReady { profile: String, filename: String, download_url: String },
    CoverLetterReady { profile: String },
    LowCredits { balance: i64 },
    AccountDeleted,
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
        }
    }

    pub fn html_body(&self) -> String {
        let content = match self {
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
