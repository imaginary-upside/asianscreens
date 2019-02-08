extern crate failure;
extern crate kuchiki;
extern crate reqwest;

use failure::Error;
use kuchiki::traits::*;

pub struct Actress {
    pub birthdate: String,
}

pub fn find(actress: &str) -> Result<Option<Actress>, Error> {
    let url = match grab_search(actress)? {
        Some(v) => v,
        None => return Ok(None),
    };
    let res = reqwest::get(&url)?.text()?;
    Ok(parse_actress(&res)?)
}

fn parse_actress(html: &str) -> Result<Option<Actress>, Error> {
    let birthdate = match convert_date(&find_row_value(html, "DOB:")?) {
        Some(v) => v,
        None => return Ok(None),
    };

    Ok(Some(Actress {
        birthdate: birthdate,
    }))
}

fn find_row_value(html: &str, key: &str) -> Result<String, Error> {
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
            return Ok(text.splitn(2, "\n").nth(1).unwrap().trim().to_string());
        }
    }

    return Ok("".to_string());
}

fn grab_search(actress: &str) -> Result<Option<String>, Error> {
    let url = format!(
        "https://www.asianscreens.com/search/index.asp?\
         zoom_sort=0&zoom_query=%22{}%22&zoom_per_page=10&zoom_and=0",
        actress
    );
    let res = reqwest::get(&url)?.text()?;

    if res.contains("No results found.") {
        return Ok(None);
    }

    let doc = kuchiki::parse_html().one(res);
    let redirect = doc
        .select_first(".result_block .result_title a")
        .unwrap()
        .to_owned()
        .as_node()
        .as_element()
        .unwrap()
        .attributes
        .borrow()
        .get("href")
        .map(|v| v.to_string());

    Ok(redirect)
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
            find("ren mitsuki").unwrap().unwrap().birthdate,
            "1993-10-29"
        );
    }

    #[test]
    fn test_find_fail_safely() {
        assert!(find("will never match").unwrap().is_none());
    }
}
