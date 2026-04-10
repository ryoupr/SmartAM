use serde::{Deserialize, Serialize};
use crate::LlmConfig;

// ---- OpenAI-compatible types ----
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

// ---- Bedrock Converse types ----
#[derive(Serialize)]
struct ConverseRequest {
    messages: Vec<ConverseMessage>,
    system: Vec<ConverseTextBlock>,
}

#[derive(Serialize)]
struct ConverseMessage {
    role: String,
    content: Vec<ConverseContent>,
}

#[derive(Serialize)]
struct ConverseContent {
    text: String,
}

#[derive(Serialize)]
struct ConverseTextBlock {
    text: String,
}

#[derive(Deserialize)]
struct ConverseResponse {
    output: ConverseOutput,
    usage: Option<ConverseUsage>,
}

#[derive(Deserialize)]
struct ConverseUsage {
    #[serde(rename = "inputTokens")]
    input_tokens: Option<u64>,
    #[serde(rename = "outputTokens")]
    output_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct ConverseOutput {
    message: ConverseOutputMessage,
}

#[derive(Deserialize)]
struct ConverseOutputMessage {
    content: Vec<ConverseOutputContent>,
}

#[derive(Deserialize)]
struct ConverseOutputContent {
    text: Option<String>,
}

fn is_bedrock_api_key(config: &LlmConfig) -> bool {
    !config.api_key.is_empty() && config.base_url.contains("bedrock-runtime")
}

async fn chat(config: &LlmConfig, messages: Vec<ChatMessage>) -> Result<String, String> {
    crate::ai_usage::check_budget()?;
    if is_bedrock_api_key(config) {
        bedrock_converse(config, messages).await
    } else {
        openai_chat(config, messages).await
    }
}

async fn openai_chat(config: &LlmConfig, messages: Vec<ChatMessage>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}/v1/chat/completions", config.base_url))
        .json(&ChatRequest { model: config.model.clone(), messages });
    if !config.api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", config.api_key));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("LLMリクエスト失敗: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("LLMエラー ({status}): {body}"));
    }

    let data: ChatResponse = resp.json().await.map_err(|e| format!("レスポンス解析失敗: {e}"))?;
    if let Some(usage) = &data.usage {
        crate::ai_usage::record_usage(
            &config.model,
            usage.prompt_tokens.unwrap_or(0),
            usage.completion_tokens.unwrap_or(0),
        );
    }
    data.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or("空のレスポンス".into())
}

async fn bedrock_converse(config: &LlmConfig, messages: Vec<ChatMessage>) -> Result<String, String> {
    // Separate system message from user/assistant messages
    let mut system_blocks = Vec::new();
    let mut converse_msgs = Vec::new();

    for msg in &messages {
        // Bedrock rejects empty text fields
        let text = if msg.content.trim().is_empty() { " ".to_string() } else { msg.content.clone() };
        if msg.role == "system" {
            system_blocks.push(ConverseTextBlock { text });
        } else {
            converse_msgs.push(ConverseMessage {
                role: msg.role.clone(),
                content: vec![ConverseContent { text }],
            });
        }
    }

    let url = format!("{}/model/{}/converse", config.base_url, config.model);
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&ConverseRequest { messages: converse_msgs, system: system_blocks })
        .send()
        .await
        .map_err(|e| format!("Bedrockリクエスト失敗: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Bedrockエラー ({status}): {body}"));
    }

    let data: ConverseResponse = resp.json().await.map_err(|e| format!("レスポンス解析失敗: {e}"))?;
    if let Some(usage) = &data.usage {
        crate::ai_usage::record_usage(
            &config.model,
            usage.input_tokens.unwrap_or(0),
            usage.output_tokens.unwrap_or(0),
        );
    }
    data.output.message.content.first()
        .and_then(|c| c.text.clone())
        .ok_or("空のレスポンス".into())
}

/// Simple system+user chat for use by other modules (e.g. calendar).
pub async fn raw_chat(llm: &LlmConfig, system: &str, user: &str) -> Result<String, String> {
    let messages = vec![
        ChatMessage { role: "system".into(), content: system.to_string() },
        ChatMessage { role: "user".into(), content: user.to_string() },
    ];
    chat(llm, messages).await
}

pub async fn summarize(llm: &LlmConfig, mail_body: &str) -> Result<String, String> {
    let messages = vec![
        ChatMessage { role: "system".into(), content: "メール要約アシスタント。厳守ルール:\n- 5行以内。超過禁止\n- 日時・場所・金額・人名・番号など具体値を省略しない\n- 挨拶・敬語・感想・提案は一切不要。事実のみ箇条書き的に詰める\n- 「〜が記載」等の抽象表現禁止。値を直接書く\n- ユーザーに必要なアクション（返信要否・期限等）があれば最後に1行で明記".into() },
        ChatMessage { role: "user".into(), content: "明日15:00のご予約のお知らせ\n予約番号 BE74681795\n来店日時 2026年04月11日（土）15:00\nサロン名 ＣＬＡＰＳ 杉田店\nメニュー カット 4,950円\n今回の利用ポイント 200ポイント\nお支払い予定金額 4,750円".into() },
        ChatMessage { role: "assistant".into(), content: "明日4/11(土) 15:00にCLAPS杉田店でカット予約あり（予約番号: BE74681795）。料金4,950円から200pt利用で支払い4,750円。スマート支払い。キャンセルは本日23:59までマイページから可能。".into() },
        ChatMessage { role: "user".into(), content: "田中様\n4/15(火) 14:00-15:00 の定例会議を、会議室Bから会議室Aに変更させていただきます。\nアジェンダ: Q2予算レビュー、新プロジェクト進捗報告\n参加者: 田中、佐藤、鈴木\nご確認のほどよろしくお願いいたします。".into() },
        ChatMessage { role: "assistant".into(), content: "4/15(火) 14:00-15:00の定例会議の会議室がBからAに変更。アジェンダはQ2予算レビューと新プロジェクト進捗報告。参加者は田中・佐藤・鈴木の3名。確認返信が必要。".into() },
        ChatMessage { role: "user".into(), content: "ご注文ありがとうございます。\n注文番号: #A-20260409-1234\n商品: AirPods Pro (第2世代) × 1\n金額: ¥39,800（税込）\nお届け予定: 2026年4月12日\nお届け先: 東京都品川区... \n配送状況はマイページからご確認いただけます。".into() },
        ChatMessage { role: "assistant".into(), content: "AirPods Pro (第2世代) を¥39,800で注文済み（注文番号: #A-20260409-1234）。4/12配届け予定、届け先は品川区。配送状況はマイページで確認可能。".into() },
        ChatMessage { role: "user".into(), content: mail_body.to_string() },
    ];
    chat(llm, messages).await
}

pub async fn draft_nuances(llm: &LlmConfig, mail_body: &str) -> Result<Vec<crate::Nuance>, String> {
    let messages = vec![
        ChatMessage { role: "system".into(), content: "あなたはメール返信アシスタントです。以下のメールを分析し、返答のニュアンスを提案してください。\n\nルール:\n- 送信専用・配信専用・noreplyなど返信不要のメールの場合は [{\"icon\":\"🚫\",\"label\":\"返信不要\",\"description\":\"このメールは送信専用のため返信できません\"}] のみ返す（他の選択肢は不要）\n- 返信可能なメールの場合は最大5個。承諾・辞退・保留・質問・委任などの具体的なニュアンスを提案する\n- 各ニュアンスのdescriptionは具体的に（例:「日程OKと伝える」）\n\nJSON配列で返してください。形式: [{\"icon\":\"✅\",\"label\":\"承諾する\",\"description\":\"日程OKと伝える\"}]".into() },
        ChatMessage { role: "user".into(), content: mail_body.to_string() },
    ];
    let raw = chat(llm, messages).await?;
    let start = raw.find('[').ok_or("JSON配列が見つかりません")?;
    let end = raw.rfind(']').ok_or("JSON配列が見つかりません")? + 1;
    serde_json::from_str(&raw[start..end]).map_err(|e| format!("JSON解析失敗: {e}"))
}

pub async fn draft_reply(llm: &LlmConfig, mail_body: &str, nuance: &str, instruction: &str) -> Result<String, String> {
    let prompt = if instruction.is_empty() {
        format!("以下のメールに「{nuance}」のニュアンスで返信文を生成してください。署名は不要です。")
    } else {
        format!("以下のメールに「{nuance}」のニュアンスで返信文を生成してください。追加指示: {instruction}。署名は不要です。")
    };
    let messages = vec![
        ChatMessage { role: "system".into(), content: "あなたはビジネスメール返信アシスタントです。自然な日本語で返信文のみを出力してください。".into() },
        ChatMessage { role: "user".into(), content: format!("{prompt}\n\n---\n{mail_body}") },
    ];
    chat(llm, messages).await
}

pub async fn translate(llm: &LlmConfig, text: &str, target_lang: &str) -> Result<String, String> {
    let is_html = text.contains('<') && text.contains('>');
    let system = if is_html {
        format!("Translate ONLY the human-readable text inside the HTML to {target_lang}. Keep ALL HTML tags, attributes, URLs, and structure exactly unchanged. Output the translated HTML only, no explanation.")
    } else {
        format!("以下のテキストを{target_lang}に翻訳してください。翻訳文のみを出力してください。")
    };
    let messages = vec![
        ChatMessage { role: "system".into(), content: system },
        ChatMessage { role: "user".into(), content: text.to_string() },
    ];
    chat(llm, messages).await
}
