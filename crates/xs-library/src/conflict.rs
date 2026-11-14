//! Copies a sync service makes when two computers change one file at once (LIB-029):
//! "Report (Ana's conflicted copy 2026-11-12).xshot" (Dropbox),
//! "Report (copia en conflicto de Ana 2026-11-12).xshot", "Report (Ana Conflict).png"
//! (Box), "Report-LAPTOP-ANA.xshot" (OneDrive, the computer's name). Such a copy is a
//! conflict only when the original is beside it.

/// The original's name of a conflict copy's `stem`: candidates, best first.
pub fn original_stems(stem: &str) -> Vec<String> {
    let mut out = Vec::new();
    // "Name (… conflicted copy …)", "(… en conflicto …)", "(… Conflict)".
    if let Some(open) = stem.rfind(" (") {
        let inner = stem[open + 2..].trim_end_matches(')').to_lowercase();
        if inner.contains("conflicted copy") || inner.contains("en conflicto") || inner.contains("conflict") {
            out.push(stem[..open].to_string());
        }
    }
    // OneDrive: "Name-COMPUTER", with the computer's name in capitals.
    let mut cut = stem.len();
    while let Some(dash) = stem[..cut].rfind('-') {
        let tail = &stem[dash + 1..];
        let computer = tail.len() >= 3
            && tail
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
            && tail.chars().any(|c| c.is_ascii_uppercase());
        if !computer || dash == 0 {
            break;
        }
        out.push(stem[..dash].to_string());
        cut = dash;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_original_of_each_service() {
        assert_eq!(
            original_stems("Report (Ana's conflicted copy 2026-11-12)"),
            ["Report"]
        );
        assert_eq!(
            original_stems("Informe (copia en conflicto de Ana 2026-11-12)"),
            ["Informe"]
        );
        assert_eq!(original_stems("Report (Ana Conflict)"), ["Report"]);
        assert_eq!(
            original_stems("Captura-DESKTOP-ABC12"),
            ["Captura-DESKTOP", "Captura"]
        );
        assert!(original_stems("Captura 2026-11-12 10.00.00").is_empty());
        assert!(original_stems("Plan (final)").is_empty());
        assert!(original_stems("Diagram-v2").is_empty());
    }
}
