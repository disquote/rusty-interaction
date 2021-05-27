[![ci-img]][ci-link] [![cio-img]][cio-link] [![lic-img]][lic-link] [![doc-img]][doc-link]

# Rusty Interaction
This library provides types and helper functions for handling Discord's [Interactions](https://discord.com/developers/docs/interactions/slash-commands). It also provides an actix-web backend handler system to handle Interactions through your own API (instead of using the gateway).

**This library is in full development. Use for production not yet recommended**

Although it's usable and supports most of the features, it still requires work and things are subject to change. 
## Getting started
To install this library, add this dependency entry to your `Cargo.toml` file:
```toml
rusty_interaction = "0"
```
By default, this only exposes the types and security check function. If you want to use the handler, add the following to your `Cargo.toml`:
```toml
[dependencies.rusty_interaction]
version = "0"
features = ["handler"]
```
Take a look at the [documentation](https://docs.rs/rusty_interaction) and the [examples](/examples) to get yourself familiar with using the library.
## Contributing
More than welcome! :D

## What it has right now
- [x] - Data models exposure
- [x] - Interaction validation (`crate::security::verify_discord_message()`)
- [x] - Receive Interactions from Discord
- [x] - Bind interactions to a function (with the help of a macro)
- [x] - Properly respond to interactions from Discord
- [x] - Nice system to make follow-up messages.
- [ ] - Nice system to manage guild-specific commands.
- [x] - Support for components
- [ ] - Not a pile of code spaghetti that just works (oops...👀)


## Difference between receiving interactions through the gateway and your own endpoint
The gateway requires you to have a discord client where you receive interactions. 
Setting up your own endpoint makes Discord send the interactions to your own API endpoint (ex. `https://example.com/api/discord/interactions`).

If you already have an API that runs your service and you're looking to integrate with Discord, this way of receiving interactions can be really interesting.

### Ok, I want to receive interactions through the gateway. Does your library support that?
No. If you want to receive interactions through the gateway, you want to take a look at [Serenity](https://github.com/serenity-rs/serenity) or one of the [other libraries](https://discord.com/developers/docs/topics/community-resources#libraries-discord-libraries).

[ci-link]: https://github.com/hugopilot/rusty-interaction/actions
[ci-img]: https://img.shields.io/github/workflow/status/hugopilot/rusty-interaction/RustCI?style=flat-square
[cio-link]: https://crates.io/crates/rusty_interaction
[cio-img]: https://img.shields.io/crates/v/rusty-interaction?style=flat-square
[lic-link]: /LICENSE
[lic-img]: https://img.shields.io/crates/l/rusty-interaction?style=flat-square
[doc-link]: https://docs.rs/rusty_interaction
[doc-img]: https://img.shields.io/docsrs/rusty_interaction/latest?style=flat-square
