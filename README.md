# Actix + Tonic

A sample project that shows how set up an [Actix
Web](https://github.com/actix/actix-web) server that communicates with a
"backend" gRPC server via [Tonic](https://github.com/hyperium/tonic).

## Actix Web

The initial code for the Actix Web part of this project was taken directly from
the [Getting Started](https://actix.rs/docs/getting-started/) documentation
page.

## Tonic

The initial code for Tonic comes from the [Getting
Started](https://github.com/hyperium/tonic/blob/master/examples/helloworld-tutorial.md)
hello world tutorial.

## Actix + Tonic

The example is completed by putting a Tonic client in the Actix Web server. 

The Actix Web server is started in another thread, based on the instructions
given in the [Actix server documentation](https://actix.rs/docs/server/). The
Tonic client is passed to the server as application data, see the
[Data](https://docs.rs/actix-web/3.3.2/actix_web/web/struct.Data.html) struct
documentation.

The Tonic client can be used concurrently. To achieve this we simply clone the
application state containing the client and use the clone as described in the
[client
documentation](https://docs.rs/tonic/0.5.2/tonic/client/index.html#concurrent-usage).

## Running the example

To run the example, you need three terminals (or whatever your preference is to
run three commands at the same time):
* Backend Server - Tonic gRPC Server
* Frontend Server - Actix Web HTTP Server
* HTTP Client - Curl

We start them in that order.

Start the backend (HTTP) server:

```sh
cargo run --bin backend-server-tonic
```

Start the frontend (gRPC) server:

```sh
cargo run --bin frontend-server-actix
```

Finally, make a request to the frontend (HTTP) server with `curl`:

```sh
curl -X POST -d "My name" 127.0.0.1:8080/echo
```

You should see the response:

```
Hello My name!
```

Of course, this doesn't show that this request made its way all the way up to
the Tonic server, but you can check the logs from the frontend and backend
servers to verify that.
