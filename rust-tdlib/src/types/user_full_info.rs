// Portions Copyright (c) FlintWithBlackCrown
// Copyright (c) 2020-2021 Anton Spitsyn
// SPDX-License-Identifier: MIT

use crate::errors::Result;
use crate::types::*;
use uuid::Uuid;

/// Contains full information about a user
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserFullInfo {
    #[doc(hidden)]
    #[serde(rename(serialize = "@extra", deserialize = "@extra"))]
    extra: Option<String>,
    #[serde(rename(serialize = "@client_id", deserialize = "@client_id"))]
    client_id: Option<i32>,
    /// User profile photo; may be null
    photo: Option<ChatPhoto>,
    /// True, if the user is blocked by the current user

    #[serde(default)]
    is_blocked: bool,
    /// True, if the user can be called

    #[serde(default)]
    can_be_called: bool,
    /// True, if a video call can be created with the user

    #[serde(default)]
    supports_video_calls: bool,
    /// True, if the user can't be called due to their privacy settings

    #[serde(default)]
    has_private_calls: bool,
    /// True, if the user can't be linked in forwarded messages due to their privacy settings

    #[serde(default)]
    has_private_forwards: bool,
    /// True, if the current user needs to explicitly allow to share their phone number with the user when the method addContact is used

    #[serde(default)]
    need_phone_number_privacy_exception: bool,
    /// A short user bio

    #[serde(default)]
    bio: Option<FormattedText>,
    /// For bots, the text that is shown on the bot's profile page and is sent together with the link when users share the bot

    #[serde(default)]
    share_text: Option<FormattedText>,
    /// Contains full information about a user

    #[serde(default)]
    description: Option<FormattedText>,
    /// Number of group chats where both the other user and the current user are a member; 0 for the current user

    #[serde(default)]
    group_in_common_count: i32,
    /// For bots, list of the bot commands

    #[serde(default)]
    commands: Vec<BotCommand>,
}

impl RObject for UserFullInfo {
    #[doc(hidden)]
    fn extra(&self) -> Option<&str> {
        self.extra.as_deref()
    }
    #[doc(hidden)]
    fn client_id(&self) -> Option<i32> {
        self.client_id
    }
}

impl UserFullInfo {
    pub fn from_json<S: AsRef<str>>(json: S) -> Result<Self> {
        Ok(serde_json::from_str(json.as_ref())?)
    }
    pub fn builder() -> UserFullInfoBuilder {
        let mut inner = UserFullInfo::default();
        inner.extra = Some(Uuid::new_v4().to_string());

        UserFullInfoBuilder { inner }
    }

    pub fn photo(&self) -> &Option<ChatPhoto> {
        &self.photo
    }

    pub fn is_blocked(&self) -> bool {
        self.is_blocked
    }

    pub fn can_be_called(&self) -> bool {
        self.can_be_called
    }

    pub fn supports_video_calls(&self) -> bool {
        self.supports_video_calls
    }

    pub fn has_private_calls(&self) -> bool {
        self.has_private_calls
    }

    pub fn has_private_forwards(&self) -> bool {
        self.has_private_forwards
    }

    pub fn need_phone_number_privacy_exception(&self) -> bool {
        self.need_phone_number_privacy_exception
    }

    pub fn bio(&self) -> Option<&FormattedText> {
        self.bio.as_ref()
    }

    pub fn share_text(&self) -> Option<&FormattedText> {
        self.share_text.as_ref()
    }

    pub fn description(&self) -> Option<&FormattedText> {
        self.description.as_ref()
    }

    pub fn group_in_common_count(&self) -> i32 {
        self.group_in_common_count
    }

    pub fn commands(&self) -> &Vec<BotCommand> {
        &self.commands
    }
}

#[doc(hidden)]
pub struct UserFullInfoBuilder {
    inner: UserFullInfo,
}

#[deprecated]
pub type RTDUserFullInfoBuilder = UserFullInfoBuilder;

impl UserFullInfoBuilder {
    pub fn build(&self) -> UserFullInfo {
        self.inner.clone()
    }

    pub fn photo<T: AsRef<ChatPhoto>>(&mut self, photo: T) -> &mut Self {
        self.inner.photo = Some(photo.as_ref().clone());
        self
    }

    pub fn is_blocked(&mut self, is_blocked: bool) -> &mut Self {
        self.inner.is_blocked = is_blocked;
        self
    }

    pub fn can_be_called(&mut self, can_be_called: bool) -> &mut Self {
        self.inner.can_be_called = can_be_called;
        self
    }

    pub fn supports_video_calls(&mut self, supports_video_calls: bool) -> &mut Self {
        self.inner.supports_video_calls = supports_video_calls;
        self
    }

    pub fn has_private_calls(&mut self, has_private_calls: bool) -> &mut Self {
        self.inner.has_private_calls = has_private_calls;
        self
    }

    pub fn has_private_forwards(&mut self, has_private_forwards: bool) -> &mut Self {
        self.inner.has_private_forwards = has_private_forwards;
        self
    }

    pub fn need_phone_number_privacy_exception(
        &mut self,
        need_phone_number_privacy_exception: bool,
    ) -> &mut Self {
        self.inner.need_phone_number_privacy_exception = need_phone_number_privacy_exception;
        self
    }

    pub fn bio<T: AsRef<FormattedText>>(&mut self, bio: T) -> &mut Self {
        self.inner.bio = Some(bio.as_ref().clone());
        self
    }

    pub fn share_text<T: AsRef<FormattedText>>(&mut self, share_text: T) -> &mut Self {
        self.inner.share_text = Some(share_text.as_ref().clone());
        self
    }

    pub fn description<T: AsRef<FormattedText>>(&mut self, description: T) -> &mut Self {
        self.inner.description = Some(description.as_ref().clone());
        self
    }

    pub fn group_in_common_count(&mut self, group_in_common_count: i32) -> &mut Self {
        self.inner.group_in_common_count = group_in_common_count;
        self
    }

    pub fn commands(&mut self, commands: Vec<BotCommand>) -> &mut Self {
        self.inner.commands = commands;
        self
    }
}

impl AsRef<UserFullInfo> for UserFullInfo {
    fn as_ref(&self) -> &UserFullInfo {
        self
    }
}

impl AsRef<UserFullInfo> for UserFullInfoBuilder {
    fn as_ref(&self) -> &UserFullInfo {
        &self.inner
    }
}
