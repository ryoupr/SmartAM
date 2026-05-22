use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{LazyLock, Mutex};
use chrono::Datelike;

static USAGE: LazyLock<Mutex<AiUsageStore>> = LazyLock::new(|| {
    Mutex::new(AiUsageStore::load())
});

static PRICING: LazyLock<Mutex<HashMap<String, ModelPricing>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static BUDGET_LIMIT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const MAX_HISTORY: usize = 100;
const MAX_DAILY_DAYS: usize = 90;
const MAX_MONTHLY_MONTHS: usize = 24;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiUsageStore {
    pub monthly: HashMap<String, MonthlyUsage>,
    #[serde(default)]
    pub daily: BTreeMap<String, DailyUsage>,
    #[serde(default)]
    pub history: VecDeque<UsageLogEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DailyUsage {
    pub cost_microcents: u64,
    pub requests: u64,
    pub features: HashMap<String, FeatureUsage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FeatureUsage {
    pub cost_microcents: u64,
    pub requests: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageLogEntry {
    pub timestamp: String,
    pub model: String,
    pub feature: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MonthlyUsage {
    pub models: HashMap<String, ModelUsage>,
    #[serde(default)]
    pub total_cost_cents: u64, // kept for backward compat on read
    pub total_cost_microcents: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cost_cents: u64, // kept for backward compat on read
    pub cost_microcents: u64,
    pub requests: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelPricing {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
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

#[derive(Debug, Serialize, Clone)]
pub struct DailyCostEntry {
    pub date: String,
    pub cost_usd: f64,
    pub requests: u64,
    pub is_estimated: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct FeatureCostEntry {
    pub feature: String,
    pub cost_usd: f64,
    pub requests: u64,
}

// [中7] Result返却
fn usage_file() -> Result<std::path::PathBuf, String> {
    let dir = dirs::data_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "Cannot determine data directory".to_string())?;
    Ok(dir.join("com.smartam.app").join("ai_usage.json"))
}

fn current_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

impl AiUsageStore {
    fn load() -> Self {
        let path = match usage_file() {
            Ok(p) => p,
            Err(e) => { log::error!("usage_file error: {e}"); return Self::default(); }
        };
        match std::fs::read_to_string(&path) {
            Ok(s) => match serde_json::from_str(&s) {
                Ok(store) => store,
                Err(e) => {
                    log::error!("ai_usage.json parse error: {e}, creating backup");
                    let bak = path.with_extension("json.bak");
                    let _ = std::fs::copy(&path, &bak);
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    // [高3] Result返却 + [中16] パーミッション0o600
    fn save(&self) -> Result<(), String> {
        let path = usage_file()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("serialize: {e}"))?;
        std::fs::OpenOptions::new()
            .write(true).create(true).truncate(true).mode(0o600)
            .open(&path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, json.as_bytes()))
            .map_err(|e| format!("write failed: {e}"))
    }

    // [高5] 古いデータを削除
    fn prune(&mut self) {
        let cutoff_daily = (chrono::Local::now() - chrono::Duration::days(MAX_DAILY_DAYS as i64))
            .format("%Y-%m-%d").to_string();
        self.daily.retain(|k, _| k.as_str() >= cutoff_daily.as_str());

        if self.monthly.len() > MAX_MONTHLY_MONTHS {
            let mut keys: Vec<String> = self.monthly.keys().cloned().collect();
            keys.sort();
            let remove_count = keys.len() - MAX_MONTHLY_MONTHS;
            for k in keys.into_iter().take(remove_count) {
                self.monthly.remove(&k);
            }
        }

        while self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
    }
}

pub fn set_budget_limit(usd: f64) {
    // [中11] バリデーション
    let usd = if !usd.is_finite() || usd < 0.0 { 0.0 } else { usd };
    BUDGET_LIMIT.store((usd * 100.0) as u64, std::sync::atomic::Ordering::Relaxed);
    log::info!("AI budget limit set to ${:.2}", usd);
}

pub fn check_budget() -> Result<(), String> {
    let limit = BUDGET_LIMIT.load(std::sync::atomic::Ordering::Relaxed);
    if limit == 0 { return Ok(()); }

    let store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
    let month = current_month();
    let cost = store.monthly.get(&month).map(|m| m.total_cost_microcents).unwrap_or(0);
    if cost >= limit * 10000 {
        Err(format!("AI利用料金が上限（${:.2}）に達しました。設定画面から上限を変更してください。", limit as f64 / 100.0))
    } else {
        Ok(())
    }
}

// [高3] Mutex外save + [中12] clone→drop→save
pub fn record_usage(model: &str, input_tokens: u64, output_tokens: u64, feature: &str) {
    let pricing = get_pricing(model);
    let cost_usd = input_tokens as f64 / 1000.0 * pricing.input_per_1k
        + output_tokens as f64 / 1000.0 * pricing.output_per_1k;
    if !cost_usd.is_finite() { return; }
    let cost_microcents = (cost_usd * 1_000_000.0).round() as u64;

    let snapshot = {
        let mut store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
        let month = current_month();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let monthly = store.monthly.entry(month).or_default();
        let model_usage = monthly.models.entry(model.to_string()).or_default();
        model_usage.input_tokens += input_tokens;
        model_usage.output_tokens += output_tokens;
        model_usage.cost_microcents += cost_microcents;
        model_usage.requests += 1;
        monthly.total_cost_microcents += cost_microcents;

        let daily = store.daily.entry(today).or_default();
        daily.cost_microcents += cost_microcents;
        daily.requests += 1;
        let feat = daily.features.entry(feature.to_string()).or_default();
        feat.cost_microcents += cost_microcents;
        feat.requests += 1;

        store.history.push_back(UsageLogEntry {
            timestamp: chrono::Local::now().to_rfc3339(),
            model: model.to_string(),
            feature: feature.to_string(),
            input_tokens,
            output_tokens,
            cost_usd,
        });

        store.prune();
        store.clone() // clone before dropping lock
    }; // Mutex released here

    if let Err(e) = snapshot.save() {
        log::error!("AI usage save failed: {e}");
    }
    log::trace!("AI usage: model={} feature={} in={} out={} cost=${:.6}", model, feature, input_tokens, output_tokens, cost_usd);
}

pub fn get_summary() -> UsageSummary {
    let store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
    let month = current_month();
    build_summary(&store, &month)
}

pub fn get_available_months() -> Vec<String> {
    let store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
    let mut months: Vec<String> = store.monthly.keys().cloned().collect();
    months.sort();
    months.reverse();
    months
}

// [中11] month バリデーション
pub fn get_summary_for_month(month: &str) -> Result<UsageSummary, String> {
    if !is_valid_month(month) {
        return Err("Invalid month format (expected YYYY-MM)".into());
    }
    let store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
    Ok(build_summary(&store, month))
}

// [中11] days バリデーション
pub fn get_daily_costs(days: u32) -> Vec<DailyCostEntry> {
    let days = days.min(MAX_DAILY_DAYS as u32);
    let store = match USAGE.lock() {
        Ok(s) => s,
        Err(_) => { log::error!("Mutex poisoned in get_daily_costs"); return vec![]; }
    };
    let today = chrono::Local::now().date_naive();
    let entries: Vec<DailyCostEntry> = (0..days)
        .map(|i| {
            let date = today - chrono::Duration::days(i as i64);
            let key = date.format("%Y-%m-%d").to_string();
            let entry = store.daily.get(&key);
            DailyCostEntry {
                date: key,
                cost_usd: entry.map(|e| e.cost_microcents as f64 / 1_000_000.0).unwrap_or(0.0),
                requests: entry.map(|e| e.requests).unwrap_or(0),
                is_estimated: false,
            }
        })
        .collect();

    let has_daily = entries.iter().any(|e| e.cost_usd > 0.0);
    if !has_daily {
        let mut result: Vec<DailyCostEntry> = Vec::with_capacity(days as usize);
        for i in 0..days {
            let date = today - chrono::Duration::days(i as i64);
            let month_key = date.format("%Y-%m").to_string();
            let day_of_month = date.day() as f64;
            let days_in_month = if date.month() == 12 {
                31.0
            } else {
                (chrono::NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
                    .unwrap_or(date)
                    - chrono::NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
                        .unwrap_or(date))
                .num_days() as f64
            };
            let monthly_cost = store.monthly.get(&month_key)
                .map(|m| m.total_cost_microcents as f64 / 1_000_000.0)
                .unwrap_or(0.0);
            let effective_days = if month_key == today.format("%Y-%m").to_string() {
                day_of_month
            } else {
                days_in_month
            };
            result.push(DailyCostEntry {
                date: date.format("%Y-%m-%d").to_string(),
                cost_usd: if effective_days > 0.0 { monthly_cost / effective_days } else { 0.0 },
                requests: 0,
                is_estimated: true,
            });
        }
        result.reverse();
        return result;
    }

    let mut result = entries;
    result.reverse();
    result
}

pub fn get_feature_costs(month: &str) -> Vec<FeatureCostEntry> {
    if !is_valid_month(month) { return vec![]; }
    let store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
    let mut features: HashMap<String, (u64, u64)> = HashMap::new();
    for (date, daily) in &store.daily {
        if date.starts_with(month) {
            for (feat, usage) in &daily.features {
                let entry = features.entry(feat.clone()).or_default();
                entry.0 += usage.cost_microcents;
                entry.1 += usage.requests;
            }
        }
    }
    features.into_iter()
        .map(|(feature, (mc, req))| FeatureCostEntry {
            feature,
            cost_usd: mc as f64 / 1_000_000.0,
            requests: req,
        })
        .collect()
}

pub fn get_history() -> Vec<UsageLogEntry> {
    let store = USAGE.lock().unwrap_or_else(|e| e.into_inner());
    store.history.iter().cloned().collect()
}

// --- helpers ---

fn build_summary(store: &AiUsageStore, month: &str) -> UsageSummary {
    let limit = BUDGET_LIMIT.load(std::sync::atomic::Ordering::Relaxed) as f64 / 100.0;
    let monthly = store.monthly.get(month);
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
        month: month.to_string(),
        models,
        total_cost_usd: total_cost,
        budget_limit_usd: limit,
        budget_remaining_usd: if limit > 0.0 { (limit - total_cost).max(0.0) } else { -1.0 },
    }
}

fn is_valid_month(month: &str) -> bool {
    chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d").is_ok()
}

fn get_pricing(model: &str) -> ModelPricing {
    let cache = PRICING.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(p) = cache.get(model) {
        return p.clone();
    }
    drop(cache);
    let (input, output) = if model.contains("claude") && model.contains("sonnet") {
        (0.003, 0.015)
    } else if model.contains("claude") && model.contains("haiku") {
        (0.0008, 0.004)
    } else if model.contains("claude") && model.contains("opus") {
        (0.015, 0.075)
    } else if model.contains("llama") || model.contains("mistral") {
        (0.0002, 0.0002)
    } else {
        (0.003, 0.015)
    };
    ModelPricing { input_per_1k: input, output_per_1k: output }
}

// [高1] サイズ制限引き上げ + フォールバック改善
pub async fn fetch_pricing() {
    log::debug!("Fetching Bedrock pricing...");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = client.get("https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AmazonBedrock/current/index.json")
        .send().await;

    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        _ => { log::warn!("Failed to fetch Bedrock pricing, using fallback"); return; }
    };

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => { log::warn!("Failed to read pricing body: {e}"); return; }
    };
    // [高1] 50MBまで許容
    if bytes.len() > 50_000_000 {
        log::warn!("Pricing response too large: {} bytes", bytes.len());
        return;
    }

    #[derive(Deserialize)]
    struct PriceIndex {
        products: HashMap<String, Product>,
        terms: Terms,
    }
    #[derive(Deserialize)]
    struct Product {
        #[allow(dead_code)]
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
        #[allow(dead_code)]
        unit: String,
        description: String,
    }

    let data: PriceIndex = match serde_json::from_slice(&bytes) {
        Ok(d) => d,
        Err(e) => { log::warn!("Parse pricing failed: {e}"); return; }
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
                        entry.0 = Some(price * 1000.0);
                    } else if desc_lower.contains("output") || usage_type.contains("Output") {
                        entry.1 = Some(price * 1000.0);
                    }
                }
            }
        }
    }

    let mut cache = PRICING.lock().unwrap_or_else(|e| e.into_inner());
    for (model, (input, output)) in pricing_map {
        if let (Some(i), Some(o)) = (input, output) {
            cache.insert(model, ModelPricing { input_per_1k: i, output_per_1k: o });
        }
    }
    log::info!("Bedrock pricing loaded: {} models", cache.len());
}
