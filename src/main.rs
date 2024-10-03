use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

fn custom_sort(a: &str, b: &str) -> Ordering {
    let a_parts: Vec<&str> = a.split(":").collect();
    let b_parts: Vec<&str> = b.split(":").collect();
    let a_coin = a_parts[1].trim_end_matches("USDT");
    let b_coin = b_parts[1].trim_end_matches("USDT");

    // Sayısal kısımları ve alfabetik kısımları ayır
    let (a_num, a_alpha) = split_numeric_alpha(a_coin);
    let (b_num, b_alpha) = split_numeric_alpha(b_coin);

    match (a_num, b_num) {
        // Her iki coin de sayısal bir önek içeriyorsa
        (Some(a_val), Some(b_val)) => {
            match b_val.partial_cmp(&a_val).unwrap_or(Ordering::Equal) {
                Ordering::Equal => a_alpha.cmp(&b_alpha), // Sayılar eşitse alfabetik sırala
                other => other, // Sayılar farklıysa büyükten küçüğe sırala
            }
        }
        // Sadece bir coin sayısal önek içeriyorsa, sayısal olan önce gelir
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        // İki coin de sayısal önek içermiyorsa alfabetik sırala
        (None, None) => a_alpha.cmp(&b_alpha),
    }
}

fn split_numeric_alpha(s: &str) -> (Option<f64>, String) {
    let numeric_part: String = s
        .chars()
        .take_while(|c| c.is_digit(10) || *c == '.')
        .collect();
    let alpha_part: String = s.chars().skip(numeric_part.len()).collect();

    let numeric_value = numeric_part.parse::<f64>().ok();
    (numeric_value, alpha_part)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.gateio.ws/api/v4/spot/currency_pairs";
    let response = reqwest::get(url).await?.text().await?;
    let data: Vec<Value> = serde_json::from_str(&response)?;

    let mut markets: Vec<String> = data
        .into_iter()
        .filter(|pair| pair["quote"].as_str().unwrap_or("") == "USDT")
        .map(|pair| format!("GATEIO:{}USDT", pair["base"].as_str().unwrap()))
        .collect();

    markets.sort_by(|a, b| custom_sort(a, b));

    let mut file = File::create("gateio_usdt_markets.txt")?;
    for market in markets {
        writeln!(file, "{}", market)?;
    }

    println!("Veriler başarıyla 'gateio_usdt_markets.txt' dosyasına yazıldı.");
    Ok(())
}
