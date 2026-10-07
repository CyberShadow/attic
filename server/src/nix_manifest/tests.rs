use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A hypothetical manifest.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct HypotheticalManifest {
    #[serde(rename = "StoreDir")]
    store_dir: PathBuf,

    #[serde(rename = "WantMassQuery")]
    want_mass_query: bool,
}

#[test]
fn test_basic() {
    let manifest = r#"
StoreDir: /nix/store
WantMassQuery: 1
    "#;

    let expected = HypotheticalManifest {
        store_dir: PathBuf::from("/nix/store"),
        want_mass_query: true,
    };

    let parsed = super::from_str::<HypotheticalManifest>(manifest).unwrap();
    assert_eq!(parsed, expected);

    // TODO: Use the actual Nix parser to reparse the resulting manifest?
    let round_trip = super::to_string(&parsed).unwrap();

    // FIXME: This is pretty fragile. Just testing that it can be parsed again should
    // be enough.
    assert_eq!(manifest.trim(), round_trip.trim());

    let parsed2 = super::from_str::<HypotheticalManifest>(&round_trip).unwrap();
    assert_eq!(parsed2, expected);
}

#[test]
fn test_unquoted_number() {
    let manifest = r#"
StoreDir: 12345
WantMassQuery: 1
    "#;

    let expected = HypotheticalManifest {
        store_dir: PathBuf::from("12345"),
        want_mass_query: true,
    };

    let parsed = super::from_str::<HypotheticalManifest>(manifest).unwrap();
    assert_eq!(parsed, expected);
}

/// A hypothetical manifest with a repeated key.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct RepeatedManifest {
    #[serde(rename = "Sig")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sigs: Vec<String>,

    #[serde(rename = "Signer")]
    signer: String,
}

#[test]
fn test_repeated_key() {
    let manifest = r#"
Sig: a:1
Sig: b:2
Signer: c
    "#;

    let expected = RepeatedManifest {
        sigs: vec!["a:1".to_string(), "b:2".to_string()],
        signer: "c".to_string(),
    };

    let parsed = super::from_str::<RepeatedManifest>(manifest).unwrap();
    assert_eq!(parsed, expected);

    let round_trip = super::to_string(&parsed).unwrap();
    assert_eq!(manifest.trim(), round_trip.trim());

    let empty = RepeatedManifest {
        sigs: vec![],
        signer: "c".to_string(),
    };
    let round_trip = super::to_string(&empty).unwrap();
    assert_eq!("Signer: c", round_trip.trim());
    assert_eq!(
        super::from_str::<RepeatedManifest>(&round_trip).unwrap(),
        empty
    );
}
