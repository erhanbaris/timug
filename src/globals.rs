use minijinja::Environment;

use crate::context::TimugContext;

pub fn build_globals(env: &mut Environment<'_>, context: &mut TimugContext) {
    env.add_global("author_name", context.config.author.clone());
    env.add_global("author_email", context.config.email.clone());
    env.add_global("site_url", context.config.site_url.clone());
    env.add_global("lang", context.config.lang.clone());
    env.add_global("description", context.config.description.clone());
    env.add_global("blog_name", context.config.title.clone());
}
