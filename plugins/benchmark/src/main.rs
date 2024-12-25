use std::fs::File;
use std::io::Write;
use std::path::Path;

use lipsum::{lipsum, lipsum_with_rng, lipsum_words_with_rng};
use rand::prelude::*;

use rand::seq::SliceRandom;

fn main() {
    let lorem_tags = lipsum_with_rng(rand::thread_rng(), 20).replace(",", "").replace(".", "");
    let tags = lorem_tags.split(' ').collect::<Vec<_>>();
    let posts_path = Path::new("/Users/erhanbaris/Downloads/pi_tc/posts");


    for i in 0..1_000 {
        let title = lipsum_with_rng(rand::thread_rng(), rand::thread_rng().gen_range(5..12));
        let description = lipsum_words_with_rng(rand::thread_rng(), 1000);
        let mut post_tags = Vec::new();

        for _ in 0..rand::thread_rng().gen_range(5..12) {
            if let Some(tag) = tags.choose(&mut rand::thread_rng()) {
                post_tags.push(tag);
            }
        }

        let slug = title.to_lowercase().replace(" ", "-").replace(",", "").replace(".", "");

        let mut file = File::create(posts_path.join(format!("{}.md", i))).unwrap();
        let content = format!("---
title: {}
slug: {}
lang: en
date: 2024-12-10 17:07:45
tags:
{}
---
{}
", title, slug, post_tags.iter().map(|tag| format!(" - {}\r\n", tag)).collect::<Vec<String>>().join(""), description);

        file.write_all(content.as_bytes()).unwrap();
    }
}
