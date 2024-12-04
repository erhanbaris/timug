mod config;
mod context;
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

use extensions::{
    alertbox::AlertBox, codeblock::Codeblock, fontawesome::FontAwesome, gist::Gist, info::Info,
    quote::Quote, social_media_share::SocialMediaShare,
};
use template::RenderEngine;

fn main() -> Result<()> {
    let mut engine = RenderEngine::new();
    engine.register_extension::<Codeblock>()?;
    engine.register_extension::<Quote>()?;
    engine.register_extension::<Gist>()?;
    engine.register_extension::<AlertBox>()?;
    engine.register_extension::<FontAwesome>()?;
    engine.register_extension::<Info>()?;
    engine.register_extension::<SocialMediaShare>()?;
    engine.run()?;
    Ok(())
}
