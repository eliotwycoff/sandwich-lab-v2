use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
struct IndexPage<'a> {
    title: &'a str,
    exchanges: Vec<Exchange<'a>>
}

#[derive(Content)]
pub struct Exchange<'a> {
    pub name: &'a str,
    pub blockchain: &'a str
}

pub fn render(title: &str, exchanges: Vec<Exchange>) -> String {
    let head_content = r##"

    "##;

    let inner_content = r##"
        <h1 class="page-header">{{title}}</h1>
        <p>Which exchange would you like to explore?</p>
        <ul>
            {{#exchanges}}
            <li><strong>{{name}}</strong> on {{ blockchain }}</li>
            {{/exchanges}}
        </ul>
    "##;

    let script_content = r##"
    
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&IndexPage {
        title,
        exchanges
    })
}