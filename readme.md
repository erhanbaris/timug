# Timug Static Page Generator

No nonsense, just a static page generator. It has been created for personal blog creation purposes. It fulfills the purposes for which it was created.

Works with jinja2 template engine. And also, yaml front matter used to determinate spesific definitions. That can be renderable or order of the menu. It does not support to many features but basic features are good enough for many users.

Default template is based on Tailwind CSS but it does not need nodejs or any other package manager.
**Timug** is a single binary and it is enough to run it. It is fast and easy to use.
Later, I will add more features to it and provide bencharmk results.
It is still under development and I am using it for my personal blog.
Default theme is not perfect but it is enough for me. I am looking for contributors to improve it.

You can see the example blog [here](https://www.erhanbaris.com/).

## Installation

**Timug** developed with Rust programming language. You need to install Rust programming language to use it. You can install Rust with [rustup](https://rustup.rs/).

Here is the installation steps:
```bash
cargo install timug
```

## Usage

### Create a new project
```bash
timug init
timug --path /home/user/my_blog/ init
```
It will create required directories and files for you.
You should open an modify **timug.yaml** file. 

### Create a new post/page
```bash
timug create post "My super new post"
timug create page "My super new page"
```

Also, you can create draft post/page with **--draft** parameter.
```bash
timug create post "My super new post" --draft
timug create page "My super new page" --draft
```

### Publish your project
```bash
timug deploy
timug --path /home/user/my_blog/ deploy
```
The command will generate static files and copy them to the **public** directory. You can deploy this directory to your web server.

### Live preview
```bash
timug start
timug --path /home/user/my_blog/ start
timug --path /home/user/my_blog/ start 9090 # For custom port
```
It will start a local server and you can preview your blog on your browser.

### Log levels
**Timug** has 5 log levels. You can set log level with **--log** parameter. Default log level is **info**. All commands has log level parameter.

Available values:
```
off:   Disable all output
error: Set log level to `error`
warn:  Set log level to `warn`
info:  Set log level to `info`
debug: Set log level to `debug`
trace: Set log level to `trace`
```

Example:
```bash
timug --log off start
timug --log error deploy
```

### Help menu
You can see all available commands with help command.
```bash
timug --help
```

### Configuration
You can configure your project with **timug.yaml** file. Here is the default configuration:
```yaml
title: My Blog
description: My super blog
theme: default
deployment-folder: public # Folder for deployment. Usually it is located under the blog-path
blog-path: . # Blog path for posts and pages
author: Erhan Baris
email: erhanbaris@gmail.com
site-url: https://www.erhanbaris.com/

navs: # Navigation menu
  - name: Home
    link: /
  - name: Posts
    link: /posts.html
  - name: Books
    link: /books.html
  - name: About me
    link: /about.html

contacts:
  - icon: fas fa-at # Fontawesome v5 icon class
    name: Email
    address: erhanbaris@gmail.com
  - icon: fab fa-github
    name: Github
    address: https://github.com/erhanbaris
  - icon: fab fa-linkedin-in
    name: Linkedin
    address: https://www.linkedin.com/in/ruslan-asenov/

reading: # Currently reading book
  name: Iron Gold
  series_name: "Red Rising Saga #4"
  author: Pierce Brown
  image : https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1716325988i/33257757.jpg
  link: https://www.goodreads.com/book/show/33257757-iron-gold

projects:
  - name: Timug Static Page Generator
    link: https://github.com/erhanbaris/timug
  - name: OneResume.IO
    link: https://www.oneresume.io/
  - name: SmartCalc
    link: https://erhanbaris.github.io/smartcalc-app/
  - name: Karamel Programming Language
    link: https://github.com/erhanbaris/karamel
  - name: 6502 Assembler
    link: https://github.com/erhanbaris/timu6502asm
  - name: Yummy Game Server
    link: https://erhanbaris.github.io/yummy/

analytics: # Analytics services
  google-analytics: G-XXXXXXXX # Google Analytics ID
  microsoft-clarity: XXXXXXXX # Microsoft Clarity ID

stats: # Stats for the blog
    link: https://timug-page-infos-2.erhanbaris.workers.dev/
```
