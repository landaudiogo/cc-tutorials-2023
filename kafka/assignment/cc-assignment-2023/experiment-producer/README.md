# Execution

Example execution:
```bash
cargo run -p experiment-producer -- \
    --brokers 13.49.128.80:19093,13.49.128.80:29093,13.49.128.80:39093 \
    --topic group15
```

For the experiment producer to e able to produce data into your topic, the auth
folder should have your `ca.crt` and `kafka.keystore.pkcs12` files. These are
the files that provide your client authentication and authorization.

There are additional parameters you can configure. Run:
```bash
cargo run -p experiment-producer -- --help
```
for more information.
