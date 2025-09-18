use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Write};
use std::process;
use std::time::Duration;

/// RSS represents the root XML structure of an RSS feed
#[derive(Deserialize, Debug)]
struct Rss {
    #[serde(rename = "channel")]
    channel: Channel,
}

/// Channel represents the main content container in an RSS feed
#[derive(Deserialize, Debug)]
struct Channel {
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "description")]
    description: String,
    #[serde(rename = "link")]
    link: String,
    #[serde(rename = "item")]
    items: Vec<Item>,
}

/// Item represents a single article in the RSS feed
#[derive(Deserialize, Debug)]
struct Item {
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "link")]
    link: String,
    #[serde(rename = "description")]
    description: String,
    #[serde(rename = "pubDate")]
    pub_date: String,
}

/// RssReader handles RSS feed operations
struct RssReader {
    category_urls: HashMap<u32, &'static str>,
    html_tag_regex: Regex,
    client: reqwest::Client,
}

impl RssReader {
    /// Creates a new RssReader instance
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut category_urls = HashMap::new();
        category_urls.insert(1, "https://www.adnkronos.com/RSS_PrimaPagina.xml");
        category_urls.insert(2, "https://www.adnkronos.com/RSS_Ultimora.xml");
        category_urls.insert(3, "https://www.adnkronos.com/RSS_Politica.xml");
        category_urls.insert(4, "https://www.adnkronos.com/RSS_Esteri.xml");
        category_urls.insert(5, "https://www.adnkronos.com/RSS_Cronaca.xml");
        category_urls.insert(6, "https://www.adnkronos.com/RSS_Economia.xml");
        category_urls.insert(7, "https://www.adnkronos.com/RSS_Finanza.xml");
        category_urls.insert(8, "https://www.adnkronos.com/RSS_Sport.xml");

        let html_tag_regex = Regex::new(r"<[^>]*>")?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(RssReader {
            category_urls,
            html_tag_regex,
            client,
        })
    }

    /// Removes HTML tags and special characters from the input text
    fn remove_tags(&self, text: &str) -> String {
        // Remove HTML tags using regex
        let clean_text = self.html_tag_regex.replace_all(text, "");

        // Remove special characters
        clean_text.replace("&nbsp;", " ")
    }

    /// Fetches and parses the RSS feed from the given URL
    async fn fetch_rss_feed(&self, url: &str) -> Result<Rss, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Unexpected status code: {}", response.status()).into());
        }

        let content = response.text().await?;
        let rss: Rss = serde_xml_rs::from_str(&content)?;

        Ok(rss)
    }

    /// Displays the available RSS categories
    fn print_menu(&self) {
        println!("Adnkronos RSS Reader");
        println!("0: Exit");
        println!("1: Prima Pagina");
        println!("2: Ultim'ora");
        println!("3: Politica");
        println!("4: Esteri");
        println!("5: Cronaca");
        println!("6: Economia");
        println!("7: Finanza");
        println!("8: Sport");
        print!("\nSelect category number: ");
        io::stdout().flush().unwrap();
    }

    /// Gets user input for category selection
    fn get_category_input(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let category: u32 = input.trim().parse()?;
        Ok(category)
    }

    /// Displays RSS feed content
    fn display_feed(&self, rss: &Rss) {
        println!("\nTitle: {}", rss.channel.title);
        println!("Link: {}", rss.channel.link);
        println!("Description: {}\n", rss.channel.description);

        for item in &rss.channel.items {
            println!("Title: {}", item.title);
            println!("Link: {}", item.link);
            println!("Description: {}", self.remove_tags(&item.description));
            println!("Published: {}\n", item.pub_date);
            println!(
                "--------------------------------------------------------------------------------"
            );
        }
    }

    /// Runs the RSS reader application
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.print_menu();

        let category = self.get_category_input()?;

        if category == 0 {
            process::exit(0);
        }

        let url = self
            .category_urls
            .get(&category)
            .ok_or("Invalid category number")?;

        let rss = self.fetch_rss_feed(url).await?;
        self.display_feed(&rss);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = RssReader::new()?;

    if let Err(e) = reader.run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}

// Test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_tags() {
        let reader = RssReader::new().unwrap();
        let html = "<p>Hello <b>world</b>!</p>&nbsp;Test";
        let cleaned = reader.remove_tags(html);
        assert_eq!(cleaned, "Hello world! Test");
    }

    #[test]
    fn test_category_urls() {
        let reader = RssReader::new().unwrap();
        assert!(reader.category_urls.contains_key(&1));
        assert!(reader.category_urls.contains_key(&3));
        assert!(!reader.category_urls.contains_key(&10));
    }
}
