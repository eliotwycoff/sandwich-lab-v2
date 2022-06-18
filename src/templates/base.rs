pub fn wrap_in_html(head_content: &str, body_content: &str, script_content: &str) -> String {
    let top = r##"
        <!DOCTYPE html>
        <!--[if lt IE 7]>      <html class="no-js lt-ie9 lt-ie8 lt-ie7"> <![endif]-->
        <!--[if IE 7]>         <html class="no-js lt-ie9 lt-ie8"> <![endif]-->
        <!--[if IE 8]>         <html class="no-js lt-ie9"> <![endif]-->
        <!--[if gt IE 8]>      <html class="no-js"> <!--<![endif]-->
        <html>
            <head>
                <meta charset="utf-8">
                <meta http-equiv="X-UA-Compatible" content="IE=edge">
                <title>{{title}}</title>
                <meta name="description" content="">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>                
                <link rel="preload" as="style" href="https://fonts.googleapis.com/css2?family=Dosis:wght@200;400;600&family=Special+Elite&display=swap">
                <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Dosis:wght@200;400;600&family=Special+Elite&display=swap" media="print" onload="this.media='all'">
                <link rel="stylesheet" href="/static/css/base.css">
                <script src="/static/js/client.js"></script>
        "##;

    // Links to static assets go here as "head_content".

    let body_top = r##"
            </head>
            <body>
                <!--[if lt IE 7]>
                    <p class="browsehappy">You are using an <strong>outdated</strong> browser. Please <a href="#">upgrade your browser</a> to improve your experience.</p>
                <![endif]-->
        
                <div class="container">
        "##;

    // Body content goes here as "body_content".

    let body_bottom = r##"
                </div>
                <script type="text/javascript">
                    window.onload = async () => {
        "##;

    // JavaScript can go here as "script_content".

    let bottom = r##"
                    };
                </script>
            </body>
            </html>
        "##;

    format!("{top}{head_content}{body_top}{body_content}{body_bottom}{script_content}{bottom}")
}