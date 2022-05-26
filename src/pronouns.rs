use protocol_derive::{Decode, Encode};

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
}
