# Decisions

## Vanilla TypeScript

The interface uses plain TypeScript and one stylesheet. This keeps the small desktop workflow easy to inspect and avoids adding a component framework before the product needs one.

## JSON instead of SQLite

A single JSON file matches the brief, keeps the app portable, and is sufficient for a local question log. The Rust side owns all reads and writes so the frontend never handles filesystem paths.

## Rust owns the API call

The Anthropic request runs in Rust so the API key never enters the webview or frontend bundle. Missing keys and network or parse failures return visible command errors.

## Draft JSON contract

The model is asked for exactly a `draft` string and a `verify` array. Parsing that response before saving prevents an unstructured model response from being presented as a finished answer.
