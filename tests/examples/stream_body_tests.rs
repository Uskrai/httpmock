use httpmock::prelude::*;
use isahc::{prelude::*, Request};

#[test]
fn body_test() {
    // Arrange
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/books")
            .body("The Fellowship of the Ring")
            .body_contains("Ring")
            .body_matches(Regex::new("Fellowship").unwrap());
        then.status(201)
            .body_stream(|| futures_util::stream::once(async { Ok("The Lord of the Rings") }));
    });

    // Act: Send the request and deserialize the response to JSON
    let response = Request::post(&format!("http://{}/books", server.address()))
        .body("The Fellowship of the Ring")
        .unwrap()
        .send()
        .unwrap();

    // Assert
    m.assert();
    assert_eq!(response.status(), 201);
}

#[test]
fn delay_test() {
    // Arrange
    let server = MockServer::start();
    let time_start = std::time::Instant::now();
    let delay = std::time::Duration::from_secs(3);

    let m = server.mock(|when, then| {
        let delay = delay;
        when.method(POST)
            .path("/books")
            .body("The Fellowship of the Ring");
        then.status(201).body_stream(move || {
            futures_util::stream::unfold(0, move |state| async move {
                match state {
                    0 => Some((Ok("o"), 1)),
                    1 => Some((Ok("h"), 2)),
                    2 => Some((Ok("i"), 3)),
                    3 => {
                        tokio::time::sleep(delay).await;

                        Some((Ok("!"), 4))
                    }
                    _ => None,
                }
            })
        });
    });

    // Act: Send the request and deserialize the response to JSON
    let mut response = Request::post(&format!("http://{}/books", server.address()))
        .body("The Fellowship of the Ring")
        .unwrap()
        .send()
        .unwrap();

    m.assert();

    assert_eq!(response.status(), 201);
    assert_eq!(response.text().unwrap(), "ohi!");
    assert!(time_start.elapsed() > delay);
}
