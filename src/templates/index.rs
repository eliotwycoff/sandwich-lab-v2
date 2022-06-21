use ramhorns::{ Template, Content };
use super::base::wrap_in_html;

#[derive(Content)]
struct IndexPage<'a> {
    title: &'a str,
    inspect_url: &'a str,
    blockchains: Vec<Blockchain<'a>>
}

#[derive(Content, Eq, Ord, PartialEq, PartialOrd)]
pub struct Blockchain<'a> {
    pub name: &'a str,
    pub str_id: &'a str
}

pub fn render(
    title: &str, 
    inspect_url: &str,
    blockchains: Vec<Blockchain>
) -> String {
    let head_content = r##"

    "##;

    let inner_content = r##"
        <h1 class="page-title">{{title}}</h1>
        <form class="options" method="get" action="{{inspect_url}}">
            <p class="options__instructions">Choose Your Blockchain</p>
            <ul class="options__list">
                {{#blockchains}}
                <li class="options__item">
                    <input 
                        type="radio" 
                        value="{{str_id}}" 
                        name="blockchain" 
                        id="{{str_id}}" 
                        class="options__item__input"
                        required />
                    <label for="{{str_id}}" class="options__item__label">{{name}}</label>
                </li>
                {{/blockchains}}
            </ul>
            <p class="options_instructions">Input an LP Pair Address</p>
            <input 
                type="text" 
                id="pair_address" 
                name="pair" 
                placeholder="e.g. 0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc" 
                pattern="0x[0-9a-fA-F]{40}"
                title="a complete Ethereum address starting with 0x" 
                class="options__pair__address"
                required />
            <button 
                type="submit" 
                id="pair-button" 
                class="options__button">Search for Sandwiches</button>
        </form>
    "##;

    let script_content = r##"

    "##;

    let source = wrap_in_html(head_content, inner_content, script_content);
    let template = Template::new(source).unwrap();

    template.render(&IndexPage {
        title,
        inspect_url,
        blockchains
    })
}