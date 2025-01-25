# Miscellaneous

Command to generate certificates for docker compose:

```
rustls-cert-gen \
    --common-name=agdb \
    --ca-file-name=test_root_ca \
     --cert-file-name=test_cert \
     --country-name=CZ \
     --organization-name=Agnesoft \
     --san=localhost \
     --san=agdb0 \
     --san=agdb1 \
     --san=agdb2 \
     --output=.
```
