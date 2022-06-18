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
                <span>{{exchange_name}} on {{blockchain_name}}<br>Pair {{pair_address}}</span>
            </div>
            <div id="base-placeholder" class="placeholder placeholder--text"></div>
            <div id="quote-placeholder" class="placeholder placeholder--text"></div>
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
        
        const titlePlaceholder = document.querySelector("#page-title-placeholder");
        const title = document.createElement("h1");
        title.className = "page-title";
        title.textContent = `${data.base.symbol}-${data.quote.symbol}`;
        titlePlaceholder.replaceWith(title);
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