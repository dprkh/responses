use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"{{#if_locale "ar"}}
{{i18n "system.title"}}
{{else}}
{{i18n "system.title"}}
{{/if_locale}}"#;

    // Test English
    let template_en = responses::prompt::PromptTemplate::from_content(content)?
        .with_locale("en")?;
    let result_en = template_en.render(&json!({}))?;
    println!("English result: '{}'", result_en);
    
    // Test Arabic
    let template_ar = responses::prompt::PromptTemplate::from_content(content)?
        .with_locale("ar")?;
    let result_ar = template_ar.render(&json!({}))?;
    println!("Arabic result: '{}'", result_ar);

    Ok(())
}