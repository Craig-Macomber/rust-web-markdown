use pulldown_cmark_wikilink::{ParserOffsetIter, LinkType, Event};
pub use pulldown_cmark_wikilink::Options;
use web_sys::MouseEvent;

use core::ops::Range;
use std::collections::HashMap;

mod render;
use render::Renderer;

mod utils;

mod component;


pub struct ElementAttributes<'a, F: WebFramework> {
    pub classes: Vec<String>,
    pub style: Option<String>,
    pub inner_html: Option<String>,
    pub on_click: Option<F::Callback<'a, MouseEvent, ()>>
}

impl<'a, F: WebFramework> Default for ElementAttributes<'a, F> {
    fn default() -> Self {
        Self {
            style: None,
            classes: vec![],
            inner_html: None,
            on_click: None
        }
    }
}

pub enum HtmlElement {
    Div,
    Span,
    Paragraph,
    BlockQuote,
    Ul,
    Ol(i32),
    Li,
    Heading(u8),
    Table,
    Thead,
    Trow,
    Tcell,
    Italics,
    Bold,
    StrikeThrough,
    Pre,
    Code
}

pub trait WebFramework: Clone {
    type View;
    type HtmlCallback<'a, T>: Clone;
    type Callback<'a, A, B>: Clone;
    type Setter<T>: Clone;
    fn set<T>(&self, setter: &Self::Setter<T>, value: T);
    fn send_debug_info(&self, info: Vec<String>);
    fn el_with_attributes(&self, e: HtmlElement, inside: Self::View, attributes: ElementAttributes<Self>) -> Self::View;
    fn el(&self, e: HtmlElement, inside: Self::View) -> Self::View {
        self.el_with_attributes(e, inside, Default::default())
    }
    fn el_hr(&self, attributes: ElementAttributes<Self>) -> Self::View;
    fn el_br(&self)-> Self::View;
    fn el_fragment(&self, children: Vec<Self::View>) -> Self::View;
    fn el_a(&self, children: Self::View, href: &str) -> Self::View;
    fn el_img(&self, src: &str, alt: &str) -> Self::View;
    fn el_empty(&self) -> Self::View {
        self.el_fragment(vec![])
    }
    fn el_text(&self, text: &str) -> Self::View;
    fn mount_dynamic_link(&self, rel: &str, href: &str, integrity: &str, crossorigin: &str);
    fn el_input_checkbox(&self, checked: bool, attributes: ElementAttributes<Self>) -> Self::View;
    fn call_callback<'a, A, B>(callback: &Self::Callback<'a, A,B>, input: A) -> B;
    fn call_html_callback<'a, T>(callback: &Self::HtmlCallback<'a, T>, input: T) -> Self::View;
    fn make_callback<'a, A, B, F: Fn(A)->B + 'a>(f: F) -> Self::Callback<'a, A, B>;
}

#[derive(Clone, Debug)]
pub struct MarkdownMouseEvent {
    /// the original mouse event triggered when a text element was clicked on
    pub mouse_event: MouseEvent,

    /// the corresponding range in the markdown source, as a slice of [`u8`][u8]
    pub position: Range<usize>,

    // TODO: add a clonable tag for the type of the element
    // pub tag: pulldown_cmark::Tag<'a>,
}


/// the description of a link, used to render it with a custom callback.
/// See [pulldown_cmark::Tag::Link] for documentation
pub struct LinkDescription<F: WebFramework> {
    /// the url of the link
    pub url: String,

    /// the html view of the element under the link
    pub content: F::View,

    /// the title of the link. 
    /// If you don't know what it is, don't worry: it is ofter empty
    pub title: String,

    /// the type of link
    pub link_type: LinkType,

    /// wether the link is an image
    pub image: bool,
}


#[derive(PartialEq)]
pub struct MdComponentProps<F: WebFramework> {
    pub attributes: Vec<(String, String)>,
    pub children: F::View
}


#[derive(Clone)]
pub struct MarkdownProps<'a, F: WebFramework> 
{
    pub on_click: Option<&'a F::Callback<'a, MarkdownMouseEvent, ()>>,

    pub render_links: Option<&'a F::HtmlCallback<'a, LinkDescription<F>>>,

    pub theme: Option<&'a str>,

    pub wikilinks: bool,

    pub hard_line_breaks: bool,

    pub parse_options: Option<&'a pulldown_cmark_wikilink::Options>,

    pub components: &'a HashMap<String, F::HtmlCallback<'a, MdComponentProps<F>>>,

    pub frontmatter: Option<&'a F::Setter<String>>
}

impl<'a, F: WebFramework> Copy for MarkdownProps<'a, F> {}

pub fn render_markdown<'a, F: WebFramework>(
    cx: F, 
    source: &'a str, 
    props: MarkdownProps<'a, F>
    ) -> F::View {

    let parse_options_default = Options::all();
    let options = props.parse_options.unwrap_or(&parse_options_default);
    let mut stream: Vec<_>
        = ParserOffsetIter::new_ext(source, *options, props.wikilinks).collect();

    if props.hard_line_breaks {
        for (r, _) in &mut stream {
            if *r == Event::SoftBreak {
                *r = Event::HardBreak
            }
        }
    }

    let elements = Renderer::new(cx.clone(), props, &mut stream.into_iter())
        .collect::<Vec<_>>();


    cx.mount_dynamic_link(
        "stylesheet",
        "https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css",
        "sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI",
        "anonymous"
    );

    cx.el_fragment(elements)
}
