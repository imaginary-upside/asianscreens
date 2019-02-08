extern crate failure;
extern crate kuchiki;
extern crate reqwest;

use failure::Error;
use kuchiki::traits::*;

pub struct Actress {
    pub birthdate: Option<String>,
}

pub fn find(actress: &str) -> Result<Option<Actress>, Error> {
    let mut res = reqwest::get(&create_actress_url(actress))?;
    if !res.status().is_success() {
        let mut split: Vec<&str> = actress.split(" ").collect();
        split.reverse();
        res = reqwest::get(&create_actress_url(&split.join(" ")))?;
    }
    Ok(parse_actress(&res.text()?)?)
}

fn create_actress_url(actress: &str) -> String {
    format!(
        "https://www.asianscreens.com/{}2.asp",
        actress.to_lowercase().replace(" ", "_")
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
    let month = split.next()?;
    let day = split.next()?;
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
        assert_eq!(find("matsushita saeko").unwrap().unwrap().birthdate.unwrap(), "1990-9-30");
    }
}
