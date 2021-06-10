# How to use

Make sure you have docker and docker-compose set up and installed.

Then `cd docker-dev-env`, and `docker-compose up -d`.

Then type `docker-compose exec server bash` to get a shell inside the server, and do `cargo run --` and your arguments to test.

# Info about environment

One server with two peers connected:

- Server has ip `10.33.7.1`
- Peer 1 has ip `10.33.7.2`
- Peer 2 has ip `10.33.7.3`