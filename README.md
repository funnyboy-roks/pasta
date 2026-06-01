# Pasta

Another pastebin-like service.

## API

Pasta's API is very simple, largely inspired by the [bytebin] API.

```
PUT  /post   -+- Create a new pasta
POST /post    |
PUT  /        |
POST /       -+
GET  /{slug} --- Get uploaded data
```

When uploading, set the content-type header to a value and it will be
returned when you fetch the data again.

[bytebin]: https://github.com/lucko/bytebin/
