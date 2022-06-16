use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
struct PairProfile<'a> {
    title: &'a str,
    blockchain_name: &'a str,
    exchange_name: &'a str,
    pair_address: &'a str
}

pub fn render(
    title: &str,
    blockchain_name: &str,
    exchange_name: &str,
    pair_address: &str) -> String {

    let head_content = r##"

    "##;

    let inner_content = r##"
        <h1 class="page-header">{{exchange_name}} on {{blockchain_name}}</h1>
        <p>Searching for sandwiches on {{pair_address}}...</p>
    "##;

    let script_content = r##"
    
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&PairProfile {
        title,
        blockchain_name,
        exchange_name,
        pair_address
    })
}