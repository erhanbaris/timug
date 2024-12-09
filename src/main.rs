mod config;
mod context;
mod engine;
mod error;
mod extensions;
mod filters;
mod functions;
mod page;
mod pages;
mod post;
mod posts;
mod template;
mod tools;

use anyhow::Result;

use engine::RenderEngine;
use extensions::{
    alertbox::AlertBox, codeblock::Codeblock, contacts::Contacts, fontawesome::FontAwesome,
    gist::Gist, info::Info, projects::Projects, quote::Quote, reading::Reading,
    social_media_share::SocialMediaShare,
};

fn main() -> Result<()> {
    let mut engine = RenderEngine::new();
    engine.register_extension::<Codeblock>()?;
    engine.register_extension::<Quote>()?;
    engine.register_extension::<Gist>()?;
    engine.register_extension::<AlertBox>()?;
    engine.register_extension::<FontAwesome>()?;
    engine.register_extension::<Info>()?;
    engine.register_extension::<SocialMediaShare>()?;
    engine.register_extension::<Reading>()?;
    engine.register_extension::<Projects>()?;
    engine.register_extension::<Contacts>()?;
    engine.run()?;
    Ok(())
}
