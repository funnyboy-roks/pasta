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

### `?password`

When creating a pasta, specifying `?password=<password>` will cause the
data to be encrypted.  NOTE: this does still send the original data to
the server, it is just encrypted before it's saved.  For true end-to-end
encryption, use the web UI or encrypt locally first.

The other direction works as well; if you add `?password=<password>` to
the GET request, it will decrypt the data (on the server) before
responding.  This has the same caveat as the above paragraph.

### Content-Type

When uploading content, set the `Content-Type` header to a value and
it will be returned when that item is fetched later.  The web ui uses
`text/<language>` to do syntax highlighting and
`application/aes256gcm-encrypted` for its encrypted content.

#### `application/link`

If the `Content-Type` header is set to `application/link`, then the
server will redirect any requests to get that resource to the underlying
link.  If the GET request is made with `?redirect=false`, then this
behaviour is disabled.

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

The content-type is set to `application/aes256gcm-encrypted`.

## Self Hosting

Pasta is very easy to self-host as it is a self-contained docker image!

Use this docker-compose for inspiration (or just copy-pasta it):

```yml
services:
  pasta:
    image: funnyboyroks/pasta
    ports:
      - 5000:3000
    environment:
      - API=https://p.fbr.dev # set this to the domain/location at which your server is running
    volumes:
      - data:/data
      - db:/db
    restart: unless-stopped
volumes:
  data: {}
  db: {}
```
