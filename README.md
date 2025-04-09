# DNS Server

# Running the Program

- If you would like to enable the added logging functionality, first set the `RUST_LOG` environment variable.
    - `export RUST_LOG=[trace | debug | info | warn]`
- Run `./run.sh` in one terminal session, and `dig @127.0.0.1 -p 2053 +noedns example.com`
  or some other network tool in another, where `example.com` is an example that we want to resolve.
    - The program returns a fixed arbitrary address as a solution.
- Run as `./run.sh --resolver <address>` to work in the forwarding DNS server mode.
    - `<address>` should be of the form `<ip>:<port>`.
    - A forwarding DNS server, also known as a DNS forwarder, is a DNS server that is configured to pass DNS queries it
      receives from clients to another DNS server for resolution, instead of directly resolving DNS queries by looking
      up the information in its own local cache or authoritative records.

# Running the Tests

```sh
cargo test conn
```
