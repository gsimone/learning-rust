use actix_web::{web, App, HttpResponse, HttpServer};

use serde::Deserialize;

fn main() {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });

    println!("Server started on http://localhost:3000...");

    server
        .bind("127.0.0.1:3000")
        .expect("Error binding server to address!")
        .run()
        .expect("Error running server!");
}

fn get_index() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        r#"
            <html>
                <body>
                    <form action="./gcd" method="POST">
                        <div>
                            <label> m 
                            <input type="number" name="m" />
                            </label>
                        </div>

                        <div>
                            <label> n
                            <input type="number" name="n" />
                            </label>
                        </div>

                        <button>Submit</button>
                    </form>
                </body>
            </html>
        "#,
    )
}

fn post_gcd(form: web::Form<GcdParams>) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Don't pass zero, come on.");
    }

    let response = format!("The gcd for {} and {} is <b>{}</b>", form.m, form.n, gcd(form.m, form.n));

    HttpResponse::Ok().content_type("text/html").body(response)
}

#[derive(Deserialize)]
struct GcdParams {
    m: u64,
    n: u64,
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }

        m = m % n;
    }

    n
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(4, 10), 2);
}
