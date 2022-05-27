use protocol_derive::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub const EXAMPLE_USAGE: &str = "{S} went to the park.
I went with {o}.
{S} brought {p} frisbee.
At least I think it was {pp}.
{S} threw the frisbee to {r}.";

pub fn make_presets() -> Vec<Pronouns> {
    // TODO add more from https://pronoun.is and https://askanonbinary.tumblr.com/pronouns
    let presets = [
        (false, false, "she", "her", "her", "hers", "herself"),
        (false, false, "he", "him", "him", "him", "himself"),
        (false, false, "they", "them", "their", "theirs", "themself"),
        (false, true, "they", "them", "their", "theirs", "themselves"),
        (false, false, "fae", "faer", "faer", "faers", "faerself"),
        (false, false, "e", "em", "eir", "eirs", "emself"),
        (true, false, "E", "Em", "Eir", "Eirs", "Emself"),
        (false, false, "it", "its", "its", "its", "itself"),
    ];

    presets
        .iter()
        .map(
            |(
                case_sensitive,
                plural,
                subject,
                object,
                possessive,
                possessive_pronoun,
                reflexive,
            )| {
                Pronouns {
                    case_sensitive: *case_sensitive,
                    plural: *plural,
                    subject: subject.to_string(),
                    object: object.to_string(),
                    possessive: possessive.to_string(),
                    possessive_pronoun: possessive_pronoun.to_string(),
                    reflexive: reflexive.to_string(),
                }
            },
        )
        .collect()
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct Pronouns {
    pub case_sensitive: bool,
    pub plural: bool,

    /// Ex. he, she, they, fae.
    pub subject: String,

    /// Ex. him, her, them, faer.
    pub object: String,

    /// Ex. his, her, their, faer.
    pub possessive: String,

    /// Ex. his, hers, theirs, faers.
    pub possessive_pronoun: String,

    /// Ex. himself, herself, themself, faerself.
    pub reflexive: String,
}

impl Pronouns {
    pub fn format_short(&self) -> String {
        format!("{}/{}", self.subject, self.object)
    }

    pub fn format_pronouns(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            self.subject, self.object, self.possessive, self.possessive_pronoun, self.reflexive
        )
    }

    pub fn format_usage(&self) -> Option<String> {
        let mut usages = Vec::new();

        if self.plural {
            usages.push("plural");
        }

        if self.case_sensitive {
            usages.push("case-sensitive");
        }

        if usages.len() > 0 {
            Some(usages.join(", "))
        } else {
            None
        }
    }

    pub fn format_full(&self) -> String {
        let pronouns = self.format_pronouns();
        if let Some(usage) = self.format_usage() {
            format!("{} [{}]", pronouns, usage)
        } else {
            pronouns
        }
    }

    pub fn make_table(&self) -> PronounTable {
        let capitalize = |s: &String| {
            if self.case_sensitive {
                s.to_string()
            } else if s.len() > 0 {
                let mut capitalized = s.get(0..1).unwrap().to_uppercase().to_string();
                capitalized.push_str(&s[1..]);
                capitalized.to_string()
            } else {
                "".to_string()
            }
        };

        PronounTable {
            case_sensitive: self.case_sensitive,
            plural: self.plural,
            subject: self.subject.clone(),
            object: self.object.clone(),
            possessive: self.possessive.clone(),
            possessive_pronoun: self.possessive_pronoun.clone(),
            reflexive: self.reflexive.clone(),
            subject_capitalized: capitalize(&self.subject),
            object_capitalized: capitalize(&self.object),
            possessive_capitalized: capitalize(&self.possessive),
            possessive_pronoun_capitalized: capitalize(&self.possessive_pronoun),
            reflexive_capitalized: capitalize(&self.reflexive),
        }
    }

    pub fn make_example_usage(&self) -> String {
        let table = self.make_table();
        let mut tt = tinytemplate::TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        tt.add_template("example_usage", EXAMPLE_USAGE).unwrap();
        tt.render("example_usage", &table).unwrap()
    }
}

/// Baked pronoun lookup table for formatting messages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PronounTable {
    pub case_sensitive: bool,
    pub plural: bool,

    #[serde(rename = "s")]
    pub subject: String,

    #[serde(rename = "o")]
    pub object: String,

    #[serde(rename = "p")]
    pub possessive: String,

    #[serde(rename = "pp")]
    pub possessive_pronoun: String,

    #[serde(rename = "r")]
    pub reflexive: String,

    #[serde(rename = "S")]
    pub subject_capitalized: String,

    #[serde(rename = "O")]
    pub object_capitalized: String,

    #[serde(rename = "P")]
    pub possessive_capitalized: String,

    #[serde(rename = "PP")]
    pub possessive_pronoun_capitalized: String,

    #[serde(rename = "R")]
    pub reflexive_capitalized: String,
}
