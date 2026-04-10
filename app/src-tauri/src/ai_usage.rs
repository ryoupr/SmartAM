use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static USAGE: LazyLock<Mutex<AiUsageStore>> = LazyLock::new(|| {
    Mutex::new(AiUsageStore::load())
});

static PRICING: LazyLock<Mutex<HashMap<String, ModelPricing>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static BUDGET_LIMIT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0); // cents (USD * 100)

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiUsageStore {
    pub monthly: HashMap<String, MonthlyUsage>, // key: "2026-04"
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MonthlyUsage {
    pub models: HashMap<String, ModelUsage>,
    pub total_cost_cents: u64,
    pub total_cost_microcents: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_cents: u64,
    pub cost_microcents: u64, // 1 microcent = 1/10000 cent for precision
    pub requests: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelPricing {
    pub input_per_1k: f64,  // USD per 1K input tokens
    pub output_per_1k: f64, // USD per 1K output tokens
}

#[derive(Debug, Serialize, Clone)]
pub struct UsageSummary {
    pub month: String,
    pub models: Vec<ModelUsageSummary>,
    pub total_cost_usd: f64,
    pub budget_limit_usd: f64,
    pub budget_remaining_usd: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModelUsageSummary {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub requests: u64,
}

fn usage_file() -> std::path::PathBuf {
    let dir = dirs::data_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".smartam"));
    dir.join("com.smartam.app").join("ai_usage.json")
}

fn current_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

impl AiUsageStore {
    fn load() -> Self {
        let path = usage_file();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) {
        let path = usage_file();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, serde_json::to_string_pretty(self).unwrap_or_default());
    }
}

pub fn set_budget_limit(usd: f64) {
    BUDGET_LIMIT.store((usd * 100.0) as u64, std::sync::atomic::Ordering::Relaxed);
    crate::trace::trace("AI_USAGE", &format!("budget limit set to ${:.2}", usd));
}

pub fn check_budget() -> Result<(), String> {
    let limit = BUDGET_LIMIT.load(std::sync::atomic::Ordering::Relaxed);
    if limit == 0 { return Ok(()); } // No limit set

    let store = USAGE.lock().unwrap();
    let month = current_month();
    let cost = store.monthly.get(&month).map(|m| m.total_cost_microcents).unwrap_or(0);
    if cost >= limit * 10000 {
        Err(format!("AI利用料金が上限（${:.2}）に達しました。設定画面から上限を変更してください。", limit as f64 / 100.0))
    } else {
        Ok(())
    }
}

pub fn record_usage(model: &str, input_tokens: u64, output_tokens: u64) {
    let pricing = get_pricing(model);
    let cost_usd = input_tokens as f64 / 1000.0 * pricing.input_per_1k
        + output_tokens as f64 / 1000.0 * pricing.output_per_1k;
    let cost_microcents = (cost_usd * 1_000_000.0) as u64; // 1 USD = 1,000,000 microcents

    let mut store = USAGE.lock().unwrap();
    let month = current_month();
    let monthly = store.monthly.entry(month).or_default();
    let model_usage = monthly.models.entry(model.to_string()).or_default();
    model_usage.input_tokens += input_tokens;
    model_usage.output_tokens += output_tokens;
    model_usage.cost_microcents += cost_microcents;
    model_usage.cost_cents = model_usage.cost_microcents / 10000;
    model_usage.requests += 1;
    monthly.total_cost_microcents += cost_microcents;
    monthly.total_cost_cents = monthly.total_cost_microcents / 10000;
    let total_mc = monthly.total_cost_microcents;

    store.save();

    crate::trace::trace("AI_USAGE", &format!(
        "model={} in={} out={} cost=${:.6} total=${:.4}",
        model, input_tokens, output_tokens,
        cost_usd,
        total_mc as f64 / 1_000_000.0
    ));
}

pub fn get_summary() -> UsageSummary {
    let store = USAGE.lock().unwrap();
    let month = current_month();
    let limit = BUDGET_LIMIT.load(std::sync::atomic::Ordering::Relaxed) as f64 / 100.0;

    let monthly = store.monthly.get(&month);
    let total_cost = monthly.map(|m| m.total_cost_microcents as f64 / 1_000_000.0).unwrap_or(0.0);
    let models: Vec<ModelUsageSummary> = monthly
        .map(|m| {
            m.models.iter().map(|(name, u)| ModelUsageSummary {
                model: name.clone(),
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
                cost_usd: u.cost_microcents as f64 / 1_000_000.0,
                requests: u.requests,
            }).collect()
        })
        .unwrap_or_default();

    UsageSummary {
        month,
        models,
        total_cost_usd: total_cost,
        budget_limit_usd: limit,
        budget_remaining_usd: if limit > 0.0 { (limit - total_cost).max(0.0) } else { -1.0 },
    }
}

fn get_pricing(model: &str) -> ModelPricing {
    let cache = PRICING.lock().unwrap();
    if let Some(p) = cache.get(model) {
        return p.clone();
    }
    drop(cache);
    // Fallback: common Bedrock model pricing (USD per 1K tokens)
    // These are approximate defaults; fetch_pricing() updates them
    let (input, output) = if model.contains("claude") && model.contains("sonnet") {
        (0.003, 0.015)
    } else if model.contains("claude") && model.contains("haiku") {
        (0.0008, 0.004)
    } else if model.contains("claude") && model.contains("opus") {
        (0.015, 0.075)
    } else if model.contains("llama") || model.contains("mistral") {
        (0.0002, 0.0002)
    } else {
        (0.003, 0.015) // default
    };
    ModelPricing { input_per_1k: input, output_per_1k: output }
}

pub async fn fetch_pricing() {
    crate::trace::trace("AI_USAGE", "Fetching Bedrock pricing...");
    let client = reqwest::Client::new();
    let resp = client.get("https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AmazonBedrock/current/index.json")
        .send().await;

    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        _ => { crate::trace::trace("AI_USAGE", "Failed to fetch pricing"); return; }
    };

    #[derive(Deserialize)]
    struct PriceIndex {
        products: HashMap<String, Product>,
        terms: Terms,
    }
    #[derive(Deserialize)]
    struct Product {
        sku: String,
        attributes: HashMap<String, String>,
    }
    #[derive(Deserialize)]
    struct Terms {
        #[serde(rename = "OnDemand")]
        on_demand: HashMap<String, HashMap<String, TermDetail>>,
    }
    #[derive(Deserialize)]
    struct TermDetail {
        #[serde(rename = "priceDimensions")]
        price_dimensions: HashMap<String, PriceDim>,
    }
    #[derive(Deserialize)]
    struct PriceDim {
        #[serde(rename = "pricePerUnit")]
        price_per_unit: HashMap<String, String>,
        unit: String,
        description: String,
    }

    let data: PriceIndex = match resp.json().await {
        Ok(d) => d,
        Err(e) => { crate::trace::trace("AI_USAGE", &format!("Parse pricing failed: {e}")); return; }
    };

    let mut pricing_map: HashMap<String, (Option<f64>, Option<f64>)> = HashMap::new();

    for (sku, product) in &data.products {
        let model_id = product.attributes.get("model").or_else(|| product.attributes.get("modelId"));
        let usage_type = product.attributes.get("usagetype").map(|s| s.as_str()).unwrap_or("");
        let model_id = match model_id {
            Some(m) => m.clone(),
            None => continue,
        };

        if let Some(term_map) = data.terms.on_demand.get(sku) {
            for term in term_map.values() {
                for dim in term.price_dimensions.values() {
                    let price: f64 = dim.price_per_unit.get("USD")
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(0.0);
                    if price == 0.0 { continue; }

                    let entry = pricing_map.entry(model_id.clone()).or_insert((None, None));
                    let desc_lower = dim.description.to_lowercase();
                    if desc_lower.contains("input") || usage_type.contains("Input") {
                        entry.0 = Some(price * 1000.0); // convert per-token to per-1K
                    } else if desc_lower.contains("output") || usage_type.contains("Output") {
                        entry.1 = Some(price * 1000.0);
                    }
                }
            }
        }
    }

    let mut cache = PRICING.lock().unwrap();
    for (model, (input, output)) in pricing_map {
        if let (Some(i), Some(o)) = (input, output) {
            cache.insert(model, ModelPricing { input_per_1k: i, output_per_1k: o });
        }
    }
    crate::trace::trace("AI_USAGE", &format!("Pricing loaded: {} models", cache.len()));
}
