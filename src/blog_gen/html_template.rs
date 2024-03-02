pub fn get_index_page_template() -> &'static str {
    let index_page_template: &str = r###"
<!doctype html>
<html>
<head>
<style>
    html, body {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        background-color: #222;
        min-height: 100%;
    }

    .body {
        color: #fafafa;
    }

    .container {
        display: flex;
        justify-content: center;
        align-items: center;
        align-content: center;
        flex-direction: column;
        min-width: 500px;
        height: 80%;
        margin: 0;
        min-height: 80%;
    }

    .row-item {
        display: flex;
        position: relative;
        width: 100%;
        padding: 5px;
        align-items: center;
        justify-content: center;
    }
    
    a {
        text-decoration: none;
    }

    a, a:visited, a:hover, a:active {
        color: #fafafa;
    }

    a:hover {
        font-weight: bold;
    }
</style>
</head>

<body>
<div class="container">
    {% for page in pages -%}
        <div class="row-item"><a href="{{ page.url }}">{{ page.title }}</a></div>
    {%- endfor %}
</div>
</body>
</html>
"###;

    return index_page_template;
}

pub fn get_html_template() -> &'static str {
    let html_template: &str = r###"
<!doctype html>
<html>
<head>
<style>
{{ css_from_source }}

img {
    max-width: 200px;
}

</style>
</head>

<body>
{{ body_content }}
</body>

</html>
"###;

    return html_template;
}
