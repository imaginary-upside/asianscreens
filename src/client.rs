extern crate failure;
extern crate kuchiki;
extern crate reqwest;

use failure::Error;
use kuchiki::traits::*;

pub struct Actress {
    pub birthdate: Option<String>,
}

pub fn find(name: &str) -> Result<Option<Actress>, Error> {
    recursive_find(name, 2)
}

fn recursive_find(name: &str, unique: u8) -> Result<Option<Actress>, Error> {
    let mut res = reqwest::get(&create_actress_url(name, unique))?;
    if !res.status().is_success() {
        let mut split: Vec<&str> = name.split(" ").collect();
        split.reverse();
        res = reqwest::get(&create_actress_url(&split.join(" "), unique))?;
    }
    let actress = match parse_actress(&res.text()?)? {
        Some(a) => a,
        None => return Ok(None),
    };
    match actress.birthdate {
        Some(_) => return Ok(Some(actress)),
        None => return recursive_find(name, unique + 1),
    };
}

fn create_actress_url(actress: &str, unique: u8) -> String {
    format!(
        "https://www.asianscreens.com/{}{}.asp",
        actress.to_lowercase().replace(" ", "_"),
        unique
    )
}

fn parse_actress(html: &str) -> Result<Option<Actress>, Error> {
    let birthdate = match find_row_value(html, "DOB:")?.map(|v| convert_date(&v)) {
        Some(v) => v,
        None => return Ok(None),
    };

    Ok(Some(Actress {
        birthdate: birthdate,
    }))
}

fn find_row_value(html: &str, key: &str) -> Result<Option<String>, Error> {
    let doc = kuchiki::parse_html().one(html);
    for m in doc.select("tr:nth-child(2) b").unwrap() {
        let key_text = m.text_contents().trim().to_string();

        if key_text.contains(key) {
            let parent = m
                .as_node()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap();
            let text = parent.text_contents().trim().to_string();
            if text == "n/a" {
                return Ok(None);
            } else {
                return Ok(Some(
                    text.splitn(2, "\n").nth(1).unwrap().trim().to_string(),
                ));
            }
        }
    }

    return Ok(None);
}

fn convert_date(original: &str) -> Option<String> {
    let mut split = original.split("/");
    let month = format!("{:0>2}", split.next()?);
    let day = format!("{:0>2}", split.next()?);
    let year = match split.next()?.parse::<i8>().ok()? {
        v if v >= 20 => format!("19{}", v),
        v if v < 20 => format!("20{}", v),
        _ => return None,
    };

    Some(format!("{}-{}-{}", year, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find() {
        assert_eq!(
            find("ren mitsuki").unwrap().unwrap().birthdate.unwrap(),
            "1993-10-29"
        );
    }

    #[test]
    fn test_find_fail_safely() {
        assert!(find("will never match").unwrap().is_none());
    }

    #[test]
    fn test_find_backwards_name() {
        assert_eq!(
            find("matsushita saeko")
                .unwrap()
                .unwrap()
                .birthdate
                .unwrap(),
            "1990-09-30"
        );
    }

    #[test]
    fn test_find_duplicate_name() {
        assert_eq!(
            find("kaori").unwrap().unwrap().birthdate.unwrap(),
            "1975-05-08"
        );
    }
}
