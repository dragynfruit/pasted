# Pasted

## What is it?

**Pasted** is a free, privacy-respecting frontend for Pastebin, written in Rust. It is designed to work **without JavaScript** and aims to be as simple as possible while still maintaining a wide range of features.

## Screenshots

![Home](imgs/home.png)
![Paste](imgs/paste.png)

## Self-Hosting

We provide a Docker image! It's available at:
[**ghcr.io/dragynfruit/pasted\:latest**](https://github.com/dragynfruit/pasted/pkgs/container/pasted)

By default, it uses port `3000` and binds to `0.0.0.0`. You can override this by setting the `PORT` and `HOST` environment variables.

A premade `docker-compose.yml` file is available [here](docker-compose.yml).

## Privacy Policy

We do **not** collect any data. However, keep in mind that **Pastebin** might.
You can view their privacy policy here:
[https://pastebin.com/doc\_privacy\_statement](https://pastebin.com/doc_privacy_statement)

## Future Plans

Currently, Pasted is focused solely on providing a frontend for Pastebin. In the future, we plan to support other major Pastebin alternatives as well — all in a single interface.

## Todo

* [x] Read paste content
* [x] Create post
* [x] Parse paste and comments
* [x] Proxy user icons
* [x] Simple homepage and info page
* [x] Info API
* [x] Comments
* [x] Paste view page
* [x] Embed paste
* [x] Clone paste
* [x] Mobile layout
* [x] Error handling
* [x] Password-protected pastes
* [x] Burn on read
* [x] Icon cache
* [x] User page
* [x] Archive page
* [ ] Markdown paste support
* [ ] View deleted pastes
* [x] Last edited support
* [x] Persistent cache volume (Docker)
* [ ] Account support
