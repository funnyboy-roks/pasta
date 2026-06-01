# Pasta

Another pastebin-like service.

## API

Pasta's API is very simple, largely inspired by the [bytebin] API.

[bytebin]: https://github.com/lucko/bytebin/

```
PUT  /post   -+- Create a new pasta
POST /post    |
PUT  /        |
POST /       -*
GET  /{slug} --- Get uploaded data
```

### Content-Type

When uploading content, set the `Content-Type` header to a value and
it will be returned when that item is fetched later.  The web ui uses
`text/<language>` to do syntax highlighting and `application/aes256gcm`
for its encrypted content.

### Content-Encoding

If possible, please specify the `Content-Encoding` header as `gzip`
(currently the only encoding supported).  When fetching, use
`Accept-Encoding: gzip`, if possible.

This reduces resource usage by the service, which makes everyone's
experience better.

## Pasta UI

The Pasta UI is inspired by [paste] and supports end-to-end encryption
of content.  To encrypt content, simply insert a password and save the
paste.  The unencrypted content will never be sent to the server and
anyone who opens the pasta will require the original password in order
to access its content.

[paste]: https://github.com/lucko/paste

### Implementation Details

The encryption is done using
[AES-GCM](https://en.wikipedia.org/wiki/Galois/Counter_Mode) with a 256
bit key and stored as `base64(salt):base64(iv):base64(cipher)`.

The content-type is set to `application/aes256`.
