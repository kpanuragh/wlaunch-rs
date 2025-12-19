use crate::core::{Item, ItemType};

pub struct WebSearchManager;

impl WebSearchManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let mut items = Vec::new();

        if query.is_empty() {
            // Show available search engines
            items.push(
                Item::new("websearch:google", "Google Search", ItemType::WebSearch)
                    .with_description("Search with Google (prefix: g)")
                    .with_icon("web-browser"),
            );
            items.push(
                Item::new("websearch:github", "GitHub Search", ItemType::WebSearch)
                    .with_description("Search GitHub (prefix: gh)")
                    .with_icon("web-browser"),
            );
            items.push(
                Item::new("websearch:youtube", "YouTube Search", ItemType::WebSearch)
                    .with_description("Search YouTube (prefix: yt)")
                    .with_icon("web-browser"),
            );
            items.push(
                Item::new("websearch:duckduckgo", "DuckDuckGo Search", ItemType::WebSearch)
                    .with_description("Search with DuckDuckGo (prefix: ddg)")
                    .with_icon("web-browser"),
            );
            items.push(
                Item::new("websearch:wikipedia", "Wikipedia Search", ItemType::WebSearch)
                    .with_description("Search Wikipedia (prefix: wiki)")
                    .with_icon("web-browser"),
            );
            return items;
        }

        // Parse search engine and query
        let (engine, search_query) = self.parse_query(query);

        if !search_query.is_empty() {
            let (name, url) = self.get_search_url(&engine, &search_query);

            let mut item = Item::new(
                format!("websearch:{}:{}", engine, search_query),
                format!("Search {}: {}", name, search_query),
                ItemType::WebSearch,
            )
            .with_description(format!("Open {} in browser", url))
            .with_icon("web-browser");

            item.metadata.search_engine = Some(engine);
            item.metadata.query = Some(search_query);
            item.metadata.url = Some(url);

            items.push(item);
        }

        items
    }

    fn parse_query(&self, query: &str) -> (String, String) {
        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        let prefix = parts.first().unwrap_or(&"").to_lowercase();
        let remainder = parts.get(1).unwrap_or(&"").to_string();

        match prefix.as_str() {
            "google" => ("google".to_string(), remainder),
            "github" => ("github".to_string(), remainder),
            "youtube" => ("youtube".to_string(), remainder),
            "ddg" | "duckduckgo" => ("duckduckgo".to_string(), remainder),
            "wiki" | "wikipedia" => ("wikipedia".to_string(), remainder),
            "stackoverflow" | "so" => ("stackoverflow".to_string(), remainder),
            "reddit" => ("reddit".to_string(), remainder),
            "amazon" => ("amazon".to_string(), remainder),
            "npm" => ("npm".to_string(), remainder),
            "crates" | "cratesio" => ("crates".to_string(), remainder),
            "pypi" => ("pypi".to_string(), remainder),
            _ => ("google".to_string(), query.to_string()),
        }
    }

    fn get_search_url(&self, engine: &str, query: &str) -> (String, String) {
        let encoded_query = urlencoding::encode(query);

        match engine {
            "google" => (
                "Google".to_string(),
                format!("https://www.google.com/search?q={}", encoded_query),
            ),
            "github" => (
                "GitHub".to_string(),
                format!("https://github.com/search?q={}", encoded_query),
            ),
            "youtube" => (
                "YouTube".to_string(),
                format!("https://www.youtube.com/results?search_query={}", encoded_query),
            ),
            "duckduckgo" => (
                "DuckDuckGo".to_string(),
                format!("https://duckduckgo.com/?q={}", encoded_query),
            ),
            "wikipedia" => (
                "Wikipedia".to_string(),
                format!("https://en.wikipedia.org/wiki/Special:Search?search={}", encoded_query),
            ),
            "stackoverflow" => (
                "Stack Overflow".to_string(),
                format!("https://stackoverflow.com/search?q={}", encoded_query),
            ),
            "reddit" => (
                "Reddit".to_string(),
                format!("https://www.reddit.com/search/?q={}", encoded_query),
            ),
            "amazon" => (
                "Amazon".to_string(),
                format!("https://www.amazon.com/s?k={}", encoded_query),
            ),
            "npm" => (
                "npm".to_string(),
                format!("https://www.npmjs.com/search?q={}", encoded_query),
            ),
            "crates" => (
                "crates.io".to_string(),
                format!("https://crates.io/search?q={}", encoded_query),
            ),
            "pypi" => (
                "PyPI".to_string(),
                format!("https://pypi.org/search/?q={}", encoded_query),
            ),
            _ => (
                "Google".to_string(),
                format!("https://www.google.com/search?q={}", encoded_query),
            ),
        }
    }
}

impl Default for WebSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

// Simple URL encoding
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push('+'),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}
