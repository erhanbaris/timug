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
timug server
timug --path /home/user/my_blog/ server
timug --path /home/user/my_blog/ server 9090 # For custom port
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

## Templating
Jinja2 template engine syntax is used for templating. You can use allmost all features of Jinja2 template engine. To get more information about supported syntax, you can take a look at (MiniJinja)[https://github.com/mitsuhiko/minijinja].

There are some built-in components, functions and filters. Here is the list of them:

### Functions
- **current_year**: Get current year. You can use it like that: `{{ current_year() }}`
- **post_url**: Get post URL. You can use it like that: `{{ post_url(post.slug) }}`
- **page_url**: Get page URL. You can use it like that: `{{ page_url(page.slug) }}`

### Filters
- **formatdatetime**: Convert date and time to spesific format. You can use it like that: `{{ post.date | formatdatetime("%B %d, %Y") }}`
- **url_encode**: Encode a string to URL format. You can use it like that: `{{ post.title | url_encode }}`


### Components
The component is a reusable part of the template. You can use it with **call** keyword. It uses the default template for rendering but it can be changed in the new template with a specific html file. New component template file should be located under the root template folder.

Here is the list of components:

#### **alertbox**
Create an alert box.

**Function arguments:**

| Argument | Information | Example | 
| ---------------- | ------ | ---- |
| style | It uses css class as a style | `success`, `fail` or `info` |
| title | Info boxes title             | `Pros` |

**Template arguments:**

| Argument | Information | Example | 
| ---------------- | ------ | ---- |
| content | Html body | `<b> Hello world </b>` |
| style | Class name             | `success`, `fail` or `info` |
| title | Info boxes title             | `Pros` |

Template name: **alertbox.html**

**Usage:**
```jinja
{% call alertbox('success', 'Pros') %}
1. Easy to use
2. Wiring diagram simple
3. Easy to coding
{% endcall %}
```

#### **quote**
Create a quote block.

**Function arguments:**
| Argument | Information | Example |
| ---------------- | ------ | ---- |
| position | it can be `right`, `left` or `center` to align quote | `center` |


**Template arguments:**
| Argument | Information | Example |
| ---------------- | ------ | ---- |
| content | Html body | `<b> Hello world </b>` |
| position | it can be `right`, `left` or `center` to align quote | `center` |

Template name: **quote.html**

**Usage:**
```jinja
{% call quote('right') %}
"Details matter. It’s worth waiting to get it right."
<br>
Steve Jobs
{% endcall %}
```

#### **codeblock**
Create a formated code block. It uses highlight.js for syntax highlighting.

**Function arguments:**
| Argument | Information | Example |
| ---------------- | ------ | ---- |
| lang | Programming languages short name | `rust` |

Template name: no templating.

**Usage:**
```jinja
{% call codeblock('bash') %}
xcode-select --install
{% endcall %}
```

#### **contacts**
Contact information block. It can be used in the footer or in the contact page. It uses fontawesome v5 icons. You should update **timug.yaml** file to use it.

**Function arguments:**
No arguments needed.


**Template arguments:**
| Argument | Information | Example |
| ---------------- | ------ | ---- |
| contacts | List of `icon: String`, `name: String`, `address: String`. It is not json format. | `[{'icon': 'fas fa-at', 'name': 'email', 'address': 'erhanbaris@gmail.com'}]` |

Template name: **contacts.html**
**Usage:**
```jinja
{% call contacts() %} {% endcall %}
```

#### **gist**
Github gist block. It can be used to embed a gist to the blog post.

**Function arguments:**
| Argument | Information | Example |
| ---------------- | ------ | ---- |
| path | GitHup gist path | `erhanbaris/bc6d9683a3e2d278851667e32759d585` |
| title | GitHup gist title | `vibration_test_output` |

Template name: no templating.

**Usage:**
```jinja
{% call gist('erhanbaris/bc6d9683a3e2d278851667e32759d585', 'vibration_test_output') %}{% endcall %}
```

#### **info**
Create an info block.

**Template arguments:**

| Argument | Information | Example | 
| ---------------- | ------ | ---- |
| content | Html body | `<b> Hello world </b>` |

Template name: **info.html**

**Usage:**
```jinja
{% call info() %}
Hello world
{% endcall %}
```
