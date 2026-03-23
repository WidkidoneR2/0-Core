// Credential health scoring — 0-100
// Age: 30%, Strength: 40%, Type: 30%

pub fn score(secret: &str, age_days: i64) -> u32 {
    let strength = score_strength(secret);
    let age_score = score_age(age_days);
    let type_score = score_type(secret);
    ((strength as f32 * 0.4) + (age_score as f32 * 0.3) + (type_score as f32 * 0.3)) as u32
}

fn score_strength(secret: &str) -> u32 {
    let len = secret.len();
    let has_upper = secret.chars().any(|c| c.is_uppercase());
    let has_lower = secret.chars().any(|c| c.is_lowercase());
    let has_digit = secret.chars().any(|c| c.is_ascii_digit());
    let has_special = secret.chars().any(|c| !c.is_alphanumeric());
    let charset = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&b| b)
        .count();
    let entropy = (len as f32) * (charset as f32 * 6.5).log2();
    (entropy.min(100.0)) as u32
}

fn score_age(age_days: i64) -> u32 {
    match age_days {
        0..=30 => 100,
        31..=60 => 85,
        61..=90 => 70,
        91..=180 => 50,
        181..=365 => 25,
        _ => 10,
    }
}

fn score_type(secret: &str) -> u32 {
    let len = secret.len();
    if len >= 32 {
        100
    } else if len >= 20 {
        80
    } else if len >= 12 {
        60
    } else if len >= 8 {
        40
    } else {
        20
    }
}

pub fn score_label(score: u32) -> (&'static str, &'static str) {
    match score {
        90..=100 => ("🟢", "excellent"),
        70..=89 => ("🟢", "good"),
        50..=69 => ("🟡", "fair"),
        30..=49 => ("🟠", "weak"),
        _ => ("🔴", "critical"),
    }
}
