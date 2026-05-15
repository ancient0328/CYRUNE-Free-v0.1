#![forbid(unsafe_code)]

use crate::policy::FailureSpec;
use cyrune_core_contract::{CitationBundleId, ClaimId, CorrelationId, RuleId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Verbatim,
    Extractive,
    Derived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationMaterialClaim {
    pub text: String,
    pub claim_kind: ClaimKind,
    pub evidence_refs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationMaterial {
    pub claims: Vec<CitationMaterialClaim>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleReasoningRecord {
    pub claims: Vec<String>,
    pub decisions: Vec<String>,
    pub assumptions: Vec<String>,
    pub actions: Vec<String>,
    pub citations_used: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationClaim {
    pub claim_id: ClaimId,
    pub text: String,
    pub claim_kind: ClaimKind,
    pub evidence_refs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationBundle {
    pub bundle_id: CitationBundleId,
    pub correlation_id: CorrelationId,
    pub claims: Vec<CitationClaim>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationValidationOutput {
    pub bundle: CitationBundle,
    pub rr: SimpleReasoningRecord,
}

#[derive(Debug, Error)]
pub enum CitationError {
    #[error("{0}")]
    Invalid(String),
}

pub fn validate_citation_output(
    correlation_id: &CorrelationId,
    output_draft: &str,
    citation_material: &CitationMaterial,
    rr_material: &SimpleReasoningRecord,
) -> Result<CitationValidationOutput, FailureSpec> {
    let claim_units = split_claim_units(output_draft).map_err(|error| {
        FailureSpec::citation_denied(
            RuleId::parse("CIT-001").expect("static rule_id must be valid"),
            format!("accepted output cannot be segmented into claim units: {error}"),
            "output を bullet / numbered list / sentence 単位へ正規化して再実行する",
        )
        .expect("static failure spec must be valid")
    })?;
    if claim_units.len() != citation_material.claims.len() {
        return Err(FailureSpec::citation_denied(
            RuleId::parse("CIT-002").expect("static rule_id must be valid"),
            "accepted output contains bundle-external claim units",
            "citation_material.claims を output claim 数と一致させて再実行する",
        )
        .expect("static failure spec must be valid"));
    }

    let mut claims = Vec::with_capacity(claim_units.len());
    let mut used_evidence = BTreeSet::new();
    for (index, (claim_text, material_claim)) in claim_units
        .into_iter()
        .zip(&citation_material.claims)
        .enumerate()
    {
        if claim_text != material_claim.text.trim() {
            return Err(FailureSpec::citation_denied(
                RuleId::parse("CIT-003").expect("static rule_id must be valid"),
                "citation claim text does not match accepted output",
                "citation_material.claims の text を accepted output と同順同文に揃えて再実行する",
            )
            .expect("static failure spec must be valid"));
        }
        if material_claim.evidence_refs.is_empty() {
            return Err(FailureSpec::citation_denied(
                RuleId::parse("CIT-004").expect("static rule_id must be valid"),
                "citation bundle is missing required evidence references",
                "claim ごとに evidence_refs を付与して再実行する",
            )
            .expect("static failure spec must be valid"));
        }
        for evidence_ref in &material_claim.evidence_refs {
            if evidence_ref.evidence_id.trim().is_empty() {
                return Err(FailureSpec::citation_denied(
                    RuleId::parse("CIT-005").expect("static rule_id must be valid"),
                    "citation bundle contains empty evidence reference",
                    "evidence_refs の空 ID を除去して再実行する",
                )
                .expect("static failure spec must be valid"));
            }
            used_evidence.insert(evidence_ref.evidence_id.clone());
        }
        claims.push(CitationClaim {
            claim_id: ClaimId::parse(format!("CLM-{index:03}", index = index + 1))
                .expect("generated claim_id must be valid"),
            text: claim_text,
            claim_kind: material_claim.claim_kind,
            evidence_refs: material_claim.evidence_refs.clone(),
        });
    }

    if rr_material.claims
        != claims
            .iter()
            .map(|claim| claim.text.clone())
            .collect::<Vec<_>>()
    {
        return Err(FailureSpec::citation_denied(
            RuleId::parse("CIT-006").expect("static rule_id must be valid"),
            "rr claims are not aligned with citation bundle claims",
            "rr_material.claims を accepted output の claim 順に揃えて再実行する",
        )
        .expect("static failure spec must be valid"));
    }

    let rr_used = rr_material
        .citations_used
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if rr_used != used_evidence {
        return Err(FailureSpec::citation_denied(
            RuleId::parse("CIT-007").expect("static rule_id must be valid"),
            "rr citations_used is not aligned with citation bundle evidence refs",
            "rr_material.citations_used を citation bundle の evidence_refs と一致させて再実行する",
        )
        .expect("static failure spec must be valid"));
    }

    Ok(CitationValidationOutput {
        bundle: CitationBundle {
            bundle_id: CitationBundleId::from_correlation_id(correlation_id),
            correlation_id: correlation_id.clone(),
            claims,
        },
        rr: rr_material.clone(),
    })
}

impl CitationBundle {
    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, CitationError> {
        canonical_json_bytes(self)
    }
}

impl SimpleReasoningRecord {
    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, CitationError> {
        canonical_json_bytes(self)
    }
}

fn split_claim_units(output: &str) -> Result<Vec<String>, CitationError> {
    if output.trim().is_empty() {
        return Err(CitationError::Invalid(
            "output_draft must not be empty".to_string(),
        ));
    }
    if output.contains("```") {
        return Err(CitationError::Invalid(
            "code block is not allowed in accepted output".to_string(),
        ));
    }

    let mut claims = Vec::new();
    let mut paragraph_buffer = Vec::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            flush_plain_paragraph(&mut paragraph_buffer, &mut claims)?;
            continue;
        }
        if is_heading(line) || is_decoration(line) {
            continue;
        }
        if is_table_line(line) {
            return Err(CitationError::Invalid(
                "table is not allowed in accepted output".to_string(),
            ));
        }
        if let Some(claim) = strip_bullet(line) {
            flush_plain_paragraph(&mut paragraph_buffer, &mut claims)?;
            claims.push(claim.to_string());
            continue;
        }
        if let Some(claim) = strip_numbered(line) {
            flush_plain_paragraph(&mut paragraph_buffer, &mut claims)?;
            claims.push(claim.to_string());
            continue;
        }
        paragraph_buffer.push(line.to_string());
    }
    flush_plain_paragraph(&mut paragraph_buffer, &mut claims)?;

    if claims.is_empty() {
        return Err(CitationError::Invalid("no claim units found".to_string()));
    }
    Ok(claims)
}

fn flush_plain_paragraph(
    paragraph_buffer: &mut Vec<String>,
    claims: &mut Vec<String>,
) -> Result<(), CitationError> {
    if paragraph_buffer.is_empty() {
        return Ok(());
    }
    let paragraph = paragraph_buffer.join(" ");
    paragraph_buffer.clear();
    for sentence in split_sentences(&paragraph)? {
        claims.push(sentence);
    }
    Ok(())
}

fn split_sentences(paragraph: &str) -> Result<Vec<String>, CitationError> {
    let mut current = String::new();
    let mut sentences = Vec::new();
    for ch in paragraph.chars() {
        current.push(ch);
        if matches!(ch, '。' | '！' | '？' | '.' | '!' | '?') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }
    if sentences.is_empty() {
        return Err(CitationError::Invalid(
            "plain paragraph could not be segmented into claims".to_string(),
        ));
    }
    Ok(sentences)
}

fn strip_bullet(line: &str) -> Option<&str> {
    ["- ", "* ", "+ "]
        .into_iter()
        .find_map(|prefix| line.strip_prefix(prefix).map(str::trim))
}

fn strip_numbered(line: &str) -> Option<&str> {
    let digits = line.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let rest = &line[digits..];
    rest.strip_prefix(". ").map(str::trim)
}

fn is_heading(line: &str) -> bool {
    line.starts_with('#')
}

fn is_decoration(line: &str) -> bool {
    matches!(line, "---" | "***" | "___")
}

fn is_table_line(line: &str) -> bool {
    line.contains('|')
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CitationError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| CitationError::Invalid(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        CitationMaterial, CitationMaterialClaim, ClaimKind, EvidenceRef, SimpleReasoningRecord,
        validate_citation_output,
    };
    use cyrune_core_contract::CorrelationId;

    #[test]
    fn uncited_claim_is_rejected() {
        let error = validate_citation_output(
            &CorrelationId::parse("RUN-20260327-0201").unwrap(),
            "- first claim",
            &CitationMaterial {
                claims: vec![CitationMaterialClaim {
                    text: "first claim".to_string(),
                    claim_kind: ClaimKind::Extractive,
                    evidence_refs: Vec::new(),
                }],
            },
            &SimpleReasoningRecord {
                claims: vec!["first claim".to_string()],
                decisions: Vec::new(),
                assumptions: Vec::new(),
                actions: Vec::new(),
                citations_used: Vec::new(),
            },
        )
        .unwrap_err();
        assert_eq!(error.rule_id.as_str(), "CIT-004");
    }

    #[test]
    fn bullet_output_is_claim_addressable() {
        let validated = validate_citation_output(
            &CorrelationId::parse("RUN-20260327-0202").unwrap(),
            "- first claim\n- second claim",
            &CitationMaterial {
                claims: vec![
                    CitationMaterialClaim {
                        text: "first claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    },
                    CitationMaterialClaim {
                        text: "second claim".to_string(),
                        claim_kind: ClaimKind::Derived,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-2".to_string(),
                        }],
                    },
                ],
            },
            &SimpleReasoningRecord {
                claims: vec!["first claim".to_string(), "second claim".to_string()],
                decisions: vec!["keep result".to_string()],
                assumptions: Vec::new(),
                actions: Vec::new(),
                citations_used: vec!["EVID-1".to_string(), "EVID-2".to_string()],
            },
        )
        .unwrap();
        assert_eq!(validated.bundle.claims.len(), 2);
        assert_eq!(validated.bundle.claims[0].claim_id.as_str(), "CLM-001");
        assert_eq!(validated.bundle.claims[1].claim_id.as_str(), "CLM-002");
    }
}
