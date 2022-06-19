use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
struct PairProfile<'a> {
    title: &'a str,
    blockchain_name: &'a str,
    blockchain_id_str: &'a str,
    exchange_name: &'a str,
    exchange_id_str: &'a str,
    pair_address: &'a str,
    api_url: &'a str
}

pub fn render(
    title: &str,
    blockchain_name: &str,
    blockchain_id_str: &str,
    exchange_name: &str,
    exchange_id_str: &str,
    pair_address: &str,
    api_url: &str
) -> String {

    let head_content = r##"

    "##;

    let inner_content = r##"
        <div id="page-title-placeholder" class="placeholder placeholder--title"></div>
        <section id="pair-metadata" class="card col">
            <div class="section-label">
                <span>Pair {{pair_address}} on </span>
                <span class="strong">{{exchange_name}} / {{blockchain_name}}</span>
            </div>
            <div class="section-body">
                <div id="base-placeholder" class="placeholder placeholder--text"></div>
                <div id="quote-placeholder" class="placeholder placeholder--text"></div>
            </div>
        </section>
    "##;

    let script_content = r##"
        const params = new URLSearchParams({
            blockchain: "{{blockchain_id_str}}",
            exchange: "{{exchange_id_str}}",
            pair_address: "{{pair_address}}"
        });

        const apiEndpoint = `{{api_url}}?${params.toString()}`;
        const response = await fetch(apiEndpoint);
        const data = await response.json();
        
        const title = document.createElement("h1");
        title.className = "page-title";
        title.textContent = `${data.base.symbol}-${data.quote.symbol}`;
        const titlePlaceholder = document.querySelector("#page-title-placeholder");
        titlePlaceholder.replaceWith(title);

        const base = document.createElement("div");
        base.className = "token-metadata";
        const baseName = document.createElement("span");
        const baseSymbol = document.createElement("span");
        const baseAddress = document.createElement("span");
        baseName.className = "token-name strong";
        baseSymbol.className = "token-symbol";
        baseAddress.className = "token-address";
        baseName.textContent = ` · ${data.base.name} `;
        baseSymbol.textContent = `(${data.base.symbol})`;
        baseAddress.textContent = ` at ${data.base.address}`;
        base.appendChild(baseName);
        base.appendChild(baseSymbol);
        base.appendChild(baseAddress);
        const basePlaceholder = document.querySelector("#base-placeholder");
        basePlaceholder.replaceWith(base);
        
        const quote = document.createElement("div");
        quote.className = "token-metadata";
        const quoteName = document.createElement("span");
        const quoteSymbol = document.createElement("span");
        const quoteAddress = document.createElement("span");
        quoteName.className = "token-name strong";
        quoteSymbol.className = "token-symbol";
        quoteAddress.className = "token-address";
        quoteName.textContent = ` · ${data.quote.name} `;
        quoteSymbol.textContent = `(${data.quote.symbol})`;
        quoteAddress.textContent = ` at ${data.quote.address}`;
        quote.appendChild(quoteName);
        quote.appendChild(quoteSymbol);
        quote.appendChild(quoteAddress);
        const quotePlaceholder = document.querySelector("#quote-placeholder");
        quotePlaceholder.replaceWith(quote);
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&PairProfile {
        title,
        blockchain_name,
        blockchain_id_str,
        exchange_name,
        exchange_id_str,
        pair_address,
        api_url
    })
}