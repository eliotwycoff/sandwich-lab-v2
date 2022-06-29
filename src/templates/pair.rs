use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
struct PairProfile<'a> {
    title: &'a str,
    blockchain_name: &'a str,
    blockchain_str_id: &'a str,
    pair_address: &'a str,
    api_pair_url: &'a str,
    api_sandwich_url: &'a str,
    home_url: &'a str
}

pub fn render(
    title: &str,
    blockchain_name: &str,
    blockchain_str_id: &str,
    pair_address: &str,
    api_pair_url: &str,
    api_sandwich_url: &str,
    home_url: &str
) -> String {

    let head_content = r##"

    "##;

    let inner_content = r##"
        <div id="page-title-placeholder" class="placeholder placeholder--title"></div>
        <p>⇦ <a href={{home_url}} title="Home">Inspect a Different Pair</a></p>
        <section id="pair-metadata" class="card col">
            <div class="card-label">
                <span class="strong">Pair </span>
                <span>{{pair_address}} on </span>
                <span class="card-label__blockchain strong"> {{blockchain_name}}</span>
                <span id="card-label__exchange"></span>
            </div>
            <div class="card-body">
                <div id="base-placeholder" class="placeholder placeholder--text"></div>
                <div id="quote-placeholder" class="placeholder placeholder--text"></div>
            </div>
        </section>
        <section id="sandwiches" class="col">
            
        </section>
    "##;

    let script_content = r##"
        const params = new URLSearchParams({
            blockchain: "{{blockchain_str_id}}",
            pair: "{{pair_address}}"
        });

        // Get pair metadata from the api.
        const apiEndpoint = `{{api_pair_url}}?${params.toString()}`;
        const response = await fetch(apiEndpoint);
        const data = await response.json();

        // Set the exchange name.
        setElement("card-label__exchange", (element) => {
            element.textContent = `(${data.pair.exchange_name})`;
        });
        
        // Update the page title.
        const title = document.createElement("h1");
        title.className = "page-title";
        title.textContent = `${data.base.symbol}-${data.quote.symbol}`;
        const titlePlaceholder = document.querySelector("#page-title-placeholder");
        titlePlaceholder.replaceWith(title);

        // Update the base token information.
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
        
        // Update the quote token information.
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

        // Create the paginator.
        let paginator = new Paginator(
            "{{blockchain_str_id}}", 
            "{{pair_address}}", 
            "{{api_sandwich_url}}",
            document.querySelector("#sandwiches"));

        await paginator.runFetchLoop();
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&PairProfile {
        title,
        blockchain_name,
        blockchain_str_id,
        pair_address,
        api_pair_url,
        api_sandwich_url,
        home_url
    })
}