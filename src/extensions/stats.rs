use std::sync::Arc;

use minijinja::{
    render,
    value::{from_args, Kwargs, Object, ObjectRepr},
    Error, State, Value,
};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::context::get_context;

use super::Extension;

static HTML: &str = include_str!("stats.html");
static SCRIPTS: &str = r#"
<script src=" https://cdn.jsdelivr.net/npm/js-cookie@3.0.5/dist/js.cookie.min.js"></script>
<script>
    fetch('[url]').then(function (response) {
        return response.json();
    }).then(function (data) {
        if (data.status) {
            var post_views = document.getElementById("post-views");
            post_views.innerHTML = data.value.views.toString() + " views";

            var post_likes = document.getElementById("post-likes");
            post_likes.innerHTML = data.value.likes.toString() + " likes";

            let likes = Cookies.get('likes');

            if (likes) {
                let likes_object = JSON.parse(likes);
                const found = likes_object.find((element) => element == "[slug]");

                if (found) {
                    var likes_button = document.getElementById("likes-button");
                    likes_button.setAttribute("disabled", "");
                }
            }
        }
    }).catch(function (err) {
        console.warn('Something went wrong.', err);
    });

    function like() {
        fetch('[url]', {
            method: "POST"
        }).then(function (response) {
            return response.json();
        }).then(function (data) {
            if (data.status) {
                var post_views = document.getElementById("post-views");
                post_views.innerHTML = data.value.views.toString() + " views";

                var post_likes = document.getElementById("post-likes");
                post_likes.innerHTML = data.value.likes.toString() + " likes";

                var likes_object = [];
                let likes = Cookies.get('likes');
                if (likes) {
                    likes_object = JSON.parse(likes);
                }

                likes_object.push("[slug]")
                Cookies.set('likes', JSON.stringify(likes_object))

                var likes_button = document.getElementById("likes-button");
                likes_button.setAttribute("disabled", "");
            }
        }).catch(function (err) {
            // There was an error
            console.warn('Something went wrong.', err);
        });
    }
</script>
"#;

#[derive(Debug, Default, Serialize, Deserialize)]
struct StatsInfo {
    link: String,
}

pub struct Stats;

impl std::fmt::Debug for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stats")
    }
}

impl Stats {
    pub fn new() -> Stats {
        Stats {}
    }
}

impl Object for Stats {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let ctx = get_context();
        if let Some(config) = ctx.get_config::<StatsInfo>(Self::name()) {
            let (slug, _): (&str, Kwargs) = from_args(args)?;
            let env = state.env();
            let path = encode(slug);

            let url = if config.link.ends_with("/") {
                format!("{}{}.html", config.link, path)
            } else {
                format!("{}/{}.html", config.link, path)
            };

            let content = match ctx.get_template_page("stats.html") {
                Some(page) => {
                    render!(in env, page.content.as_str(), scripts => SCRIPTS.replace("[url]", &url).replace("[slug]", slug))
                }
                None => {
                    render!(in env, HTML, scripts => SCRIPTS.replace("[url]", &url).replace("[slug]", slug))
                }
            };

            return Ok(Value::from_safe_string(content));
        }

        Ok(Value::UNDEFINED)
    }
}

impl<'a> Extension<'a> for Stats {
    fn name() -> &'static str {
        "stats"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }
}
