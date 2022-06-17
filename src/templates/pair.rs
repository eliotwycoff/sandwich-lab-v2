use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
struct PairProfile<'a> {
    title: &'a str,
    blockchain_name: &'a str,
    exchange_name: &'a str,
    pair_address: &'a str,
    base_name: &'a str,
    base_symbol: &'a str,
    base_address: &'a str,
    base_decimals: u8,
    quote_name: &'a str,
    quote_symbol: &'a str,
    quote_address: &'a str,
    quote_decimals: u8
}

pub fn render(
    title: &str,
    blockchain_name: &str,
    exchange_name: &str,
    pair_address: &str,
    base_name: &str,
    base_symbol: &str,
    base_address: &str,
    base_decimals: u8,
    quote_name: &str,
    quote_symbol: &str,
    quote_address: &str,
    quote_decimals: u8) -> String {

    let head_content = r##"

    "##;

    let inner_content = r##"
        <h1 class="page-header">{{exchange_name}} on {{blockchain_name}}</h1>
        <p>Searching for sandwiches on {{pair_address}}...</p>
        <h2>Token Metadata</h2>
        <p><strong>{{ base_name }} ({{ base_symbol }})</strong> at {{ base_address }} ({{ base_decimals }} decimals)</p>
        <p><strong>{{ quote_name }} ({{ quote_symbol }})</strong> at {{ quote_address }} ({{ quote_decimals }} decimals)</p>
    "##;

    let script_content = r##"
    
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&PairProfile {
        title,
        blockchain_name,
        exchange_name,
        pair_address,
        base_name,
        base_symbol,
        base_address,
        base_decimals,
        quote_name,
        quote_symbol,
        quote_address,
        quote_decimals
    })
}