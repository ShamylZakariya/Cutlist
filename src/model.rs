use anyhow::{bail, Result};
use yaml_rust::Yaml;

fn f32_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

#[derive(Clone, Debug)]
pub struct Board {
    pub width: f32,
    pub length: f32,
    pub id: String,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        f32_eq(self.width, other.width) && f32_eq(self.length, other.length)
    }
}

impl Board {
    /// Parses a Board specification format string in form of: 96x6.5, which yields
    /// Board { length: 96, width: 6.5 }
    pub fn parse(spec: &str) -> Result<Board> {
        if let Some((length, remainder)) = spec.split_once("x") {
            let length = length.parse::<f32>()?;
            if let Some((width, id)) = remainder.split_once(":") {
                let width = width.parse::<f32>()?;
                let id = String::from(id);
                if length <= 0f32 {
                    bail!("Length must be greater than 0")
                }
                if width <= 0f32 {
                    bail!("Width must be greater than 0")
                }
                if id.is_empty() {
                    bail!("Id must be non-empty")
                }
                return Ok(Board { length, width, id });
            }
        }
        bail!("Invalid format string");
    }
}

impl Eq for Board {}

#[derive(Debug, Clone)]
pub struct Cut {
    pub length: f32,
    pub width: f32,
    pub count: i32,
    pub name: String, 
}

impl PartialEq for Cut {
    fn eq(&self, other: &Self) -> bool {
        f32_eq(self.length, other.length)
            && f32_eq(self.width, other.width)
            && self.count == other.count
            && self.name == other.name
    }
}

impl Eq for Cut {}

impl Cut {
    /// Parses a cut specification format string in form of: 2@12x4:Apron, which yields
    /// Cut { length: 12, width: 4, count: 2, name: "Apron" }
    pub fn parse(spec: &str) -> Result<Cut> {
        if let Some((count, remainder)) = spec.split_once("@") {
            let count = count.parse::<i32>()?;
            if count < 1 {
                bail!("Count must be at least 1");
            }

            if let Some((length, remainder)) = remainder.split_once("x") {
                let length = length.parse::<f32>()?;
                if length <= 0f32 {
                    bail!("Length must be greater than 0");
                }

                if let Some((width, remainder)) = remainder.split_once(":") {
                    let width = width.parse::<f32>()?;
                    if width <= 0f32 {
                        bail!("Width must be greater than 0");
                    }

                    let name = remainder.to_owned();
                    return Ok(Cut {
                        length,
                        width,
                        count,
                        name,
                    });
                }
            }
        }
        bail!("Invalid Cut format string")
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub margin: f32,
    pub boards: Vec<Board>,
    pub cutlist: Vec<Cut>,
}

impl Input {
    pub fn from(doc:&Yaml) -> Result<Input> {
        Ok(Self {
            margin: Self::margin(doc)?,
            boards: Self::boards(doc)?,
            cutlist: Self::cutlist(doc)?,
        })
    }

    fn margin(doc: &Yaml) -> Result<f32> {
        if let Some(margin) = doc["margin"].as_f64() {
            Ok(margin as f32)
        } else {
            Ok(0f32)
        }
    }

    fn boards(doc: &Yaml) -> Result<Vec<Board>> {
        let mut boards = Vec::new();
        if let Yaml::Array(ref doc_boards) = doc["boards"] {
            for doc_board in doc_boards {
                if let Some(doc_board) = doc_board.as_str() {
                    boards.push(Board::parse(doc_board)?);
                }
            }
        }

        if !boards.is_empty() {
            Ok(boards)
        } else {
            bail!("No boards specified")
        }
    }

    fn cutlist(doc: &Yaml) -> Result<Vec<Cut>> {
        let mut cutlist = Vec::new();

        if let Yaml::Array(ref doc_cutlist) = doc["cutlist"] {
            for doc_cut in doc_cutlist {
                if let Some(doc_cut) = doc_cut.as_str() {
                    cutlist.push(Cut::parse(doc_cut)?);
                }
            }
        }

        if !cutlist.is_empty() {
            Ok(cutlist)
        } else {
            bail!("No cuts specified")
        }
    }
}

#[cfg(test)]
mod spec_tests {
    use super::*;

    #[test]
    fn board_parse_accepts_valid_input() {
        assert_eq!(
            Board::parse("96.5x5.5:A").expect("Expected format to parse"),
            Board {
                length: 96.5,
                width: 5.5,
                id: "A".into(),
            }
        );
        assert_eq!(
            Board::parse("96x5:Foo").expect("Expected format to parse"),
            Board {
                length: 96f32,
                width: 5f32,
                id: "Foo".into()
            }
        );
    }

    #[test]
    fn board_parse_rejects_invalid_input() {
        // Board must have an id
        assert!(Board::parse("3x5").is_err());
        assert!(Board::parse("5x5:").is_err());

        // Board dimensions must be > 0
        assert!(Board::parse("-3x5.5:A").is_err());
        assert!(Board::parse("0x5.5:A").is_err());
        assert!(Board::parse("0.0x5.5:A").is_err());
        assert!(Board::parse("10x0:A").is_err());
        assert!(Board::parse("10x0.0:A").is_err());
        assert!(Board::parse("10x-0.01:A").is_err());
        assert!(Board::parse("10x-1:A").is_err());

        // Reject bad strings
        assert!(Board::parse("This x is not a format x string").is_err());
        assert!(Board::parse("This is not a format string").is_err());
        assert!(Board::parse("axb").is_err());
        assert!(Board::parse("38907rtu4obyio4ycbnq7890237890-7cb0  f").is_err());
    }

    #[test]
    fn cut_parse_acceps_valid_input() {
        assert_eq!(
            Cut::parse("2@12x4:Apron").expect("Expected format to parse"),
            Cut {
                length: 12f32,
                width: 4f32,
                count: 2,
                name: "Apron".to_owned()
            }
        );

        assert_eq!(
            Cut::parse("22@12.5x4.8:This has multiple words").expect("Expected format to parse"),
            Cut {
                length: 12.5f32,
                width: 4.8f32,
                count: 22,
                name: "This has multiple words".to_owned()
            }
        );
    }

    #[test]
    fn cut_parse_rejects_bad_input() {
        // count must be integer >= 1
        assert!(Cut::parse("1.2@44x8:Apron").is_err());
        assert!(Cut::parse("0@44x8:Apron").is_err());
        assert!(Cut::parse("-4@44x8:Apron").is_err());

        // Length must be > 0
        assert!(Cut::parse("1@0x8:Apron").is_err());
        assert!(Cut::parse("1@-1x8:Apron").is_err());

        // Width must be > 0
        assert!(Cut::parse("1@10x0:Apron").is_err());
        assert!(Cut::parse("1@10x-1:Apron").is_err());

        // We expect a name
        assert!(Cut::parse("1@10x4").is_err());

        // Reject garbage
        assert!(Cut::parse("This is not a cut format string").is_err());
        assert!(Cut::parse("1.2.3.4").is_err());
    }
}
