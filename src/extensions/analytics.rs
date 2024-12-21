use std::sync::Arc;

use minijinja::{
    value::{Object, ObjectRepr},
    Error, ErrorKind, State, Value,
};
use parking_lot::RwLockWriteGuard;
use serde::{Deserialize, Serialize};

use crate::context::TimugContext;

use super::Extension;

#[derive(Debug, Default, Serialize, Deserialize)]
struct AnalyticsInfo {
    #[serde(rename = "google-analytics")]
    pub google_analytics: Option<String>,

    #[serde(rename = "microsoft-clarity")]
    pub microsoft_clarity: Option<String>,
}

pub struct Analytics;

impl std::fmt::Debug for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "analytics")
    }
}

impl Analytics {
    pub fn new() -> Analytics {
        Analytics {}
    }
}

impl Object for Analytics {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(self: &Arc<Self>, _: &State<'_, '_>, _: &[Value]) -> Result<Value, Error> {
        Err(Error::new(ErrorKind::InvalidOperation, "Analytics is not callable"))
    }
}

impl<'a> Extension<'a> for Analytics {
    fn name() -> &'static str {
        "analytics"
    }

    fn register(env: &mut minijinja::Environment<'a>) {
        env.add_global(Self::name(), Value::from_object(Self::new()));
    }

    fn after_body(ctx: &'_ mut RwLockWriteGuard<'static, TimugContext>) {
        if let Some(config) = ctx.get_config::<AnalyticsInfo>(Self::name()) {
            if let Some(google) = &config.google_analytics {
                ctx.after_bodies.push(
                    format!(
                        r#"<!-- Google tag (gtag.js) -->
<script async src="https://www.googletagmanager.com/gtag/js?id={}"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){{dataLayer.push(arguments);}}
  gtag('js', new Date());

  gtag('config', '{}');
</script>"#,
                        google, google
                    )
                    .into(),
                );
            }

            if let Some(clarity) = &config.microsoft_clarity {
                ctx.after_bodies.push(
                    format!(
                        r#"<script type="text/javascript">
    (function(c,l,a,r,i,t,y){{
        c[a]=c[a]||function(){{(c[a].q=c[a].q||[]).push(arguments)}};
        t=l.createElement(r);t.async=1;t.src="https://www.clarity.ms/tag/"+i;
        y=l.getElementsByTagName(r)[0];y.parentNode.insertBefore(t,y);
    }})(window, document, "clarity", "script", "{}");
</script>"#,
                        clarity
                    )
                    .into(),
                );
            }
        }
    }
}
