use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
pub struct NotFoundPage<'a> {
    title: &'a str,
    home_url: &'a str
}

pub fn render(title: &str, home_url: &str) -> String {
    let head_content = r##"

    "##;

    let inner_content = r##"
        <h1 class="page-header">404 Not Found</h1>
        <p>This isn't the content you're looking for. <a href="{{home_url}}">Return Home</a></p>

    "##;

    let script_content = r##"
    
    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&NotFoundPage {
        title,
        home_url
    })
}