use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
pub struct NotFoundPage<'a> {
    title: &'a str,
    message: &'a str,
    home_url: &'a str
}

pub fn render(title: &str, message: &str, home_url: &str) -> String {
    let head_content = r##"

    "##;

    let inner_content = r##"
        <h1 class="page-title">404 Not Found</h1>
        <p>{{message}} <a href="{{home_url}}">Return Home</a></p>

    "##;

    let script_content = r##"
    
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&NotFoundPage {
        title,
        message,
        home_url
    })
}