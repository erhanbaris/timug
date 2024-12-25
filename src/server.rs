use axum::Router;
use console::style;
use notify::event::ModifyKind;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use snafu::ResultExt;
use std::net::SocketAddr;
use std::sync::mpsc;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::consts::DEFAULT_WEBSERVER_PORT;
use crate::context::get_context;
use crate::error::{CouldNotWatchFilesystemSnafu, FileSystemWatcherFailedSnafu};
use crate::tools::inner_deploy_pages;

pub fn start_webserver(port: Option<u16>) -> crate::Result<()> {
    let ctx = get_context(snafu::location!())?;
    let deployment_path = ctx.config.deployment_folder.clone();
    drop(ctx);

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let app = Router::new().nest_service("/", ServeDir::new(deployment_path));

                let addr = SocketAddr::from(([0, 0, 0, 0], port.unwrap_or(DEFAULT_WEBSERVER_PORT)));
                let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

                log::info!("Serving on {}", style(format!("http:://{}/", listener.local_addr().unwrap())).blue());

                axum::serve(listener, app.layer(TraceLayer::new_for_http()))
                    .await
                    .unwrap();
            });
    });

    change_watcher()?;

    Ok(())
}

fn change_watcher() -> crate::Result<()> {
    let ctx = get_context(snafu::location!())?;
    let deployment_folder = ctx.config.deployment_folder.clone();
    let blog_path = ctx.config.blog_path.clone();
    let git_folder = ctx.git_folder.as_ref().map(|path| path.join(".git"));
    drop(ctx);

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx).context(FileSystemWatcherFailedSnafu)?;
    watcher
        .watch(&blog_path, RecursiveMode::Recursive)
        .context(CouldNotWatchFilesystemSnafu { path: blog_path })?;

    for event in rx.into_iter().flatten() {
        let mut need_rebuilding = true;

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Name(_)) | EventKind::Remove(_) => event.paths.iter().for_each(|path| {
                let is_git_folder = git_folder
                    .as_ref()
                    .map(|git_path| path.starts_with(git_path))
                    .unwrap_or_default();
                let is_deployment_folder = path.starts_with(&deployment_folder);

                if !is_deployment_folder && !is_git_folder {
                    log::info!("{} has been updated", style(path.display()).yellow());

                    if need_rebuilding {
                        need_rebuilding = false;
                        let _ = inner_deploy_pages();
                    }
                }
            }),
            _ => (),
        };
    }

    Ok(())
}
