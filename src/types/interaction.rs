use crate::{expect_specific_api_response, expect_successful_api_response};
#[cfg(feature="extended-handler")]
use crate::{expect_successful_api_response_and_return};


use serde::{Deserialize, Serialize};

use serde_with::*;

use serde_repr::*;

use super::application::*;
use super::embed::*;
#[cfg(feature="extended-handler")]
use super::guild::*;
use super::user::*;
use super::HttpError;
use super::Snowflake;
use ::chrono::{DateTime, Utc};
#[cfg(feature = "handler")]
use log::{debug, error};
#[cfg(any(feature = "handler", feature = "extended-handler"))]
use reqwest::{Client, StatusCode, };

// ======================

#[cfg(any(feature = "handler", feature = "extended-handler"))]
#[derive(Clone)]
///
pub struct Context {
    client: Client,
    /// The [`Interaction`] sent by Discord.
    pub interaction: Interaction,
}
#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
/// The base Interaction structure. When Interactions are received, this structure is wrapped inside a [`Context`]
/// and can be used to get information about the Interaction.
pub struct Interaction {
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    /// The application id of your applicaton
    pub application_id: Option<Snowflake>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    /// Unique id identifying the interaction
    pub id: Option<Snowflake>,
    /// The type of interaction
    pub r#type: InteractionType,
    /// Interaction data, if applicable
    pub data: Option<ApplicationCommandInteractionData>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    /// The ID of the guild where the Interaction took place (None if in DM)
    pub guild_id: Option<Snowflake>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    /// The channel ID where the Interaction took place
    pub channel_id: Option<Snowflake>,
    /// The [`Member`] who invoked the command (None if in DM, use [`User`] instead)
    pub member: Option<Member>,
    /// The [`User`] who invoked the command (None if in guild, use [`Member`] instead)
    pub user: Option<User>,
    /// Unique token used for editing messages and managing follow-up messages
    pub token: Option<String>,
    /// Read-only. Always `1`
    pub version: Option<i8>,
}



#[derive(Clone, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
#[non_exhaustive]
/// Represents the type of interaction that comes in.
pub enum InteractionType {
    /// Discord requested a ping
    Ping = 1,
    /// A slash command
    ApplicationCommand = 2,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
/// Struct repesenting an Interaction response
///
/// This is used to respond to incoming interactions.
pub struct InteractionResponse {
    /// Type of response
    pub r#type: InteractionResponseType,

    /// Optional data field
    pub data: Option<InteractionApplicationCommandCallbackData>,
}

#[cfg(feature = "handler")]
#[derive(Clone, Debug)]
/// Builder for making a [`InteractionResponse`]

pub struct InteractionResponseBuilder {
    #[doc(hidden)]
    pub r#type: InteractionResponseType,
    #[doc(hidden)]
    pub data: Option<InteractionApplicationCommandCallbackData>,
}

impl InteractionResponse {
    /// Creates a new InteractionResponse
    pub fn new(
        rtype: InteractionResponseType,
        data: Option<InteractionApplicationCommandCallbackData>,
    ) -> InteractionResponse {
        InteractionResponse {
            r#type: rtype,
            data,
        }
    }
}

#[cfg(feature = "handler")]
impl Default for InteractionResponseBuilder {
    /// This will default to responding with the `InteractionResponseType::CHANNEL_MESSAGE_WITH_SOURCE` response type and no data.
    /// Adding data yourself is expected.
    fn default() -> Self {
        Self {
            r#type: InteractionResponseType::ChannelMessageWithSource,
            data: None,
        }
    }
}

#[cfg(feature = "handler")]
impl InteractionResponseBuilder {
    fn ret(self) -> InteractionResponse {
        InteractionResponse {
            r#type: self.r#type,
            data: self.data,
        }
    }

    /// Sets the [`InteractionResponseType`]
    pub fn respond_type(mut self, t: InteractionResponseType) -> Self {
        self.r#type = t;
        self
    }

    /// Fills the [`InteractionResponse`] with some [`InteractionApplicationCommandCallbackData`]
    /// This returns an `InteractionResponse` and consumes itself.
    pub fn data(mut self, d: InteractionApplicationCommandCallbackData) -> InteractionResponse {
        self.data = Some(d);
        self.ret()
    }

    /// Sets the Text-To-Speech value of this `InteractionResponse`.
    pub fn tts(mut self, enable: bool) -> Self {
        // Does data exist?
        if self.data.is_none() {
            let mut d = InteractionApplicationCommandCallbackData::new();
            d.tts = Some(enable);
            self.data = Some(d);
        } else {
            self.data.as_mut().unwrap().tts = Some(enable);
        }
        self
    }

    /// This sets the `content` for an `InteractionResponse`
    pub fn content(mut self, c: impl ToString) -> Self {
        match self.data.as_mut() {
            None => {
                let mut d = InteractionApplicationCommandCallbackData::new();
                d.content = Some(c.to_string());
                self.data = Some(d);
            }
            Some(mut d) => {
                d.content = Some(c.to_string());
            }
        }
        self
    }

    /// Sets the `content` for an `InteractionResponse`. Alias for `content()`
    pub fn message(self, c: impl ToString) -> Self {
        self.content(c)
    }

    /// Add an [`Embed`] to the response.
    /// You can add up to 10 embeds.
    pub fn add_embed(mut self, e: Embed) -> Self {
        match self.data.as_mut() {
            None => {
                let mut d = InteractionApplicationCommandCallbackData::new();
                d.embeds = Some(vec![e]);
                self.data = Some(d);
            }
            Some(mut d) => {
                if d.embeds.is_none() {
                    d.embeds = Some(vec![e]);
                } else {
                    let v = d.embeds.as_mut().unwrap();
                    // Check if this will exceed the embed limit
                    if v.len() <= 9 {
                        v.push(e);
                    } else {
                        // Log an error for now.
                        error!("Tried to add embed while embed limit (max. 10 embeds) was already reached. Ignoring")
                    }
                }
            }
        }
        self
    }

    /// Returns an `InteractionResponse`, consuming itself.
    /// You can't use the builder anymore after you called this function.
    pub fn finish(self) -> InteractionResponse {
        self.ret()
    }
}

#[derive(Clone, Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]

/// Representing the type of response to an [`Interaction`]
pub enum InteractionResponseType {
    /// ACK a PING
    Pong = 1,
    /// Respond to an [`Interaction`] with a message
    ChannelMessageWithSource = 4,
    /// ACK an interaction and edit a response later, the user sees a loading state
    DefferedChannelMessageWithSource = 5,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
/// Representing the data used to respond to an [`Interaction`]
pub struct InteractionApplicationCommandCallbackData {
    tts: Option<bool>,
    content: Option<String>,
    embeds: Option<Vec<Embed>>,
    allowed_mentions: Option<AllowedMentions>,
    flags: Option<i32>,
}

impl InteractionApplicationCommandCallbackData {
    /// Creates a new [`InteractionApplicationCommandCallbackData`]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InteractionApplicationCommandCallbackData {
    fn default() -> Self {
        Self {
            tts: None,
            content: None,
            embeds: None,
            allowed_mentions: None,
            flags: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
/// Representing the allowed mention type
pub enum AllowedMentionTypes {
    /// Role mentions
    Roles,
    /// User mentions
    Users,
    /// @everyone mentions
    Everyone,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
/// Representing the AllowedMentions data model
pub struct AllowedMentions {
    parse: Vec<AllowedMentionTypes>,
    roles: Vec<Snowflake>,
    users: Vec<Snowflake>,
    replied_user: bool,
}
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Debug)]
/// Representing a webhook message
pub struct WebhookMessage {
    /// The message contents
    pub content: Option<String>,
    /// Embeds in the message (max 10)
    pub embeds: Option<Vec<Embed>>,
    /// Used for files.
    pub payload_json: Option<String>,
    allowed_mentions: Option<AllowedMentions>,
}
#[cfg(feature = "handler")]
impl WebhookMessage{
    /// Add text to this WebhookMessage
    pub fn content(mut self, content: impl ToString) -> Self{
        self.content = Some(content.to_string());
        self
    }
    
    /// Add an embed to this WebhookMessage
    pub fn add_embed(mut self, embed: Embed) -> Self{
        match self.embeds.as_mut() {
            None => {
                self.embeds = Some(vec![embed]);
            }
            Some(e) => {
                // Check if this will exceed the embed limit
                if e.len() <= 9 {
                    e.push(embed);
                } else {
                 // Log an error for now.
                    error!("Tried to add embed while embed limit (max. 10 embeds) was already reached. Ignoring")
                }
            }
        }
        self
    }
}
impl Default for WebhookMessage {
    fn default() -> Self {
        WebhookMessage {
            content: None,
            embeds: None,
            payload_json: None,
            allowed_mentions: None,
        }
    }
}

impl From<InteractionResponse> for WebhookMessage {
    fn from(o: InteractionResponse) -> WebhookMessage {
        let data = o.data.unwrap();

        WebhookMessage {
            content: data.content,
            embeds: data.embeds,
            payload_json: None,
            allowed_mentions: None,
        }
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
/// Reference to a message. Contains useful identifiers.
pub struct MessageReference {
    #[serde_as(as = "DisplayFromStr")]
    message_id: Snowflake,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    guild_id: Option<Snowflake>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    channel_id: Option<Snowflake>,
}

impl MessageReference {
    /// Get the message id of this message
    pub fn message_id(&self) -> Snowflake {
        self.message_id
    }
    /// Get the guild id of this message
    ///
    /// `None` if message is in DM
    pub fn guild_id(&self) -> Option<Snowflake> {
        self.guild_id
    }

    /// Get the channel ID of this message
    ///
    /// `None` if message is in DM
    pub fn channel_id(&self) -> Option<Snowflake> {
        self.channel_id
    }
}
#[cfg(feature = "handler")]
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Debug)]
/// Read-only struct representing a Followup message sent by some application.
pub struct FollowupMessage {
    #[serde_as(as = "DisplayFromStr")]
    id: Snowflake,
    r#type: u8,
    content: Option<String>,
    embeds: Vec<Embed>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    channel_id: Option<Snowflake>,
    author: Option<User>,
    tts: bool,
    #[serde_as(as = "DisplayFromStr")]
    timestamp: DateTime<Utc>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default)]
    edited_timestamp: Option<DateTime<Utc>>,
    flags: u32,
    #[serde_as(as = "DisplayFromStr")]
    application_id: Snowflake,
    #[serde_as(as = "DisplayFromStr")]
    webhook_id: Snowflake,
    message_reference: MessageReference,

    #[serde(skip)]
    interaction_token: String,
    #[serde(skip)]
    client: Client,
}
#[cfg(feature = "handler")]
/// Getter functions
impl FollowupMessage {
    /// Get the ID of this follow up
    pub fn id(&self) -> Snowflake {
        self.id
    }
    /// Get the type of message of this follow up
    pub fn get_type(&self) -> u8 {
        self.r#type
    }

    /// Get the embeds of this follow up
    pub fn embeds(&self) -> Vec<Embed> {
        self.embeds.clone()
    }

    /// Gets the contents of this followup message
    pub fn get_content(&self) -> Option<String> {
        self.content.clone()
    }
    /// Get the creation time of this followup message
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
    /// Get the time when this message was edited
    ///
    /// `None` if message was never edited
    pub fn edited_timestamp(&self) -> Option<DateTime<Utc>> {
        self.edited_timestamp
    }

    /// Get the message flags of this message
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Get the application id of the application that made this message
    pub fn app_id(&self) -> Snowflake {
        self.application_id
    }

    /// Get the webhook id associated with this message
    pub fn webhook_id(&self) -> Snowflake {
        self.webhook_id
    }

    /// Get the message reference of this message
    ///
    /// The [`MessageReference`] contains the message ID, aswell as the channel and guild id.
    pub fn message_reference(&self) -> MessageReference {
        self.message_reference.clone()
    }
}

#[cfg(feature = "handler")]
/// 'Do' functions
impl FollowupMessage {
    /// Edit this followup message
    pub async fn edit_message(&mut self, new_content: &WebhookMessage) -> Result<(), HttpError> {
        let url = format!(
            "/webhooks/{:?}/{:?}/messages/{:?}",
            self.application_id, self.interaction_token, self.id
        );

        let exec = self.client.post(&url).json(new_content).send().await;

        expect_successful_api_response!(exec, {
            // TODO: Update edited fields
            Ok(())
        })
    }

    /// Delete this followup message.
    ///
    /// If the deletion succeeded, you'll get an `Ok(())`. However, if this somehow fails, it will return `Err(Self)`.
    /// That means that if the deletion did not succeed, this reference does not go out of scope.
    ///
    /// Errors get printed using the [`::log::debug!`] macro
    pub async fn delete_message(self) -> Result<(), Self> {
        let url = format!(
            "{}/webhooks/{:?}/{}/messages/{:?}",
            crate::BASE_URL,
            self.application_id,
            self.interaction_token,
            self.id
        );

        let exec = self.client.delete(&url).send().await;

        match exec {
            Err(e) => {
                debug!("Discord API returned an error: {:#?}", e);
                Err(self)
            }
            Ok(r) => {
                if r.status() != StatusCode::NO_CONTENT {
                    let e = format!("{:#?}", r.text().await);
                    debug!(
                        "Discord API request did not return {}: {:#?}",
                        StatusCode::NO_CONTENT,
                        e
                    );
                    Err(self)
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(feature = "handler")]
impl Context {
    /// Creates a new [`Context`]
    pub fn new(c: Client, i: Interaction) -> Self {
        Self {
            client: c,
            interaction: i,
        }
    }

    /// Respond to an Interaction
    ///
    /// This returns an [`InteractionResponseBuilder`] which you can use to build an [`InteractionResponse`]
    ///
    /// # Example
    /// ```ignore
    /// let response = ctx.respond()
    ///                   .content("Example message")
    ///                   .tts(true)
    ///                   .finish();
    /// ```
    pub fn respond(&self) -> InteractionResponseBuilder {
        InteractionResponseBuilder::default()
    }

    /// Edit the original interaction response
    ///
    /// This takes an [`WebhookMessage`]. You can convert an [`InteractionResponse`] using [`WebhookMessage::from`].
    pub async fn edit_original(&self, new_content: &WebhookMessage) -> Result<(), HttpError> {
        let url = format!(
            "{}/webhooks/{:?}/{}/messages/@original",
            crate::BASE_URL,
            self.interaction.application_id.unwrap(),
            self.interaction.token.as_ref().unwrap().to_string()
        );
        let c = self.client.patch(&url).json(new_content).send().await;

        expect_successful_api_response!(c, Ok(()))
    }

    /// Delete the original interaction response
    pub async fn delete_original(&self) -> Result<(), HttpError> {
        let url = format!(
            "{}/webhooks/{:?}/{}/messages/@original",
            crate::BASE_URL,
            self.interaction.application_id.unwrap(),
            self.interaction.token.as_ref().unwrap().to_string()
        );
        let c = self.client.delete(&url).send().await;

        expect_specific_api_response!(c, StatusCode::NO_CONTENT, Ok(()))
    }

    /// Create a follow-up message
    pub async fn create_followup(
        &self,
        content: &WebhookMessage,
    ) -> Result<FollowupMessage, HttpError> {
        let url = format!(
            "{}/webhooks/{:?}/{}?wait=true",
            crate::BASE_URL,
            self.interaction.application_id.unwrap(),
            self.interaction.token.as_ref().unwrap().to_string()
        );

        let c = self.client.post(&url).json(content).send().await;

        match c {
            Err(e) => {
                debug!("Discord API request failed: {:#?}", e);
                Err(HttpError {
                    code: 0,
                    message: format!("{:#?}", e),
                })
            }
            Ok(r) => {
                let st = r.status();
                if !st.is_success() {
                    let e = format!("{:#?}", r.text().await);
                    debug!("Discord API returned an error: {:#?}", e);
                    Err(HttpError {
                        code: st.as_u16(),
                        message: e,
                    })
                } else {
                    let a: Result<FollowupMessage, serde_json::Error> =
                        serde_json::from_str(&r.text().await.unwrap());

                    match a {
                        Err(e) => {
                            debug!("Failed to decode response: {:#?}", e);
                            Err(HttpError {
                                code: 500,
                                message: format!("{:?}", e),
                            })
                        }
                        Ok(mut f) => {
                            f.interaction_token =
                                self.interaction.token.as_ref().unwrap().to_string();
                            Ok(f)
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature="extended-handler")]
/// Getter functions
impl Context{
    /// Get a [`Guild`] from an ID
    pub async fn get_guild<I: Into<Snowflake>>(&self, id: I) -> Result<Guild, HttpError>{
        let url = format!(
            "{}/guilds/{:?}?with_counts=true",
            crate::BASE_URL,
            id.into());

        let r = self.client.get(&url)
                    .send()
                    .await;
        expect_successful_api_response_and_return!(r, Guild, g, Ok(g))
    }

    /// Get a [`Member`] from a [`Guild`]
    pub async fn get_guild_member(&self, guild_id: impl Into<Snowflake>, user_id: impl Into<Snowflake>) -> Result<Member, HttpError>{
        let url = format!(
            "{}/guilds/{:?}/members/{:?}",
            crate::BASE_URL,
            guild_id.into(),
            user_id.into());

        let r = self.client.get(&url)
                    .send()
                    .await;
        expect_successful_api_response_and_return!(r, Member, m, Ok(m))
    }
}