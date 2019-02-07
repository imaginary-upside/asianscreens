extern crate kuchiki;
extern crate reqwest;
extern crate failure;

use kuchiki::traits::*;
use failure::Error;

pub struct Actress {
    pub birthdate: String,
}

pub fn find(actress: &str) -> Result<Actress, Error> {
    let url = grab_search(actress)?.unwrap();
    let res = reqwest::get(&url)?.text()?;
    parse_actress(&res)
}

fn parse_actress(html: &str) -> Result<Actress, Error> {
    let birthdate = find_row_value(html, "DOB:")?;

    Ok(Actress {
        birthdate: birthdate,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find() {
        assert_eq!(find("ren mitsuki").unwrap().birthdate, "10/29/93");
    }
}
