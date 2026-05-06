// Portions Copyright (c) 2026 FlintWithBlackCrown
// Copyright (c) 2020-2021 Anton Spitsyn
// SPDX-License-Identifier: MIT

use crate::errors::Result;
use crate::types::*;
use uuid::Uuid;

/// Sets the parameters for TDLib initialization. Works only when the current authorization state is authorizationStateWaitTdlibParameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetTdlibParameters {
    #[doc(hidden)]
    #[serde(rename(serialize = "@extra", deserialize = "@extra"))]
    extra: Option<String>,
    #[serde(rename(serialize = "@client_id", deserialize = "@client_id"))]
    client_id: Option<i32>,

    #[serde(default)]
    use_test_dc: bool,
    #[serde(default)]
    database_directory: String,
    #[serde(default)]
    files_directory: String,
    #[serde(default)]
    use_file_database: bool,
    #[serde(default)]
    use_chat_info_database: bool,
    #[serde(default)]
    use_message_database: bool,
    #[serde(default)]
    use_secret_chats: bool,
    #[serde(default)]
    api_id: i32,
    #[serde(default)]
    api_hash: String,
    #[serde(default)]
    system_language_code: String,
    #[serde(default)]
    device_model: String,
    #[serde(default)]
    system_version: String,
    #[serde(default)]
    application_version: String,
    #[serde(default)]
    enable_storage_optimizer: bool,
    #[serde(default)]
    ignore_file_names: bool,

    #[serde(rename(serialize = "@type"))]
    td_type: String,
}

impl RObject for SetTdlibParameters {
    #[doc(hidden)]
    fn extra(&self) -> Option<&str> {
        self.extra.as_deref()
    }
    #[doc(hidden)]
    fn client_id(&self) -> Option<i32> {
        self.client_id
    }
}

impl RFunction for SetTdlibParameters {}

impl SetTdlibParameters {
    pub fn from_json<S: AsRef<str>>(json: S) -> Result<Self> {
        Ok(serde_json::from_str(json.as_ref())?)
    }
    pub fn builder() -> SetTdlibParametersBuilder {
        let mut inner = SetTdlibParameters::default();
        inner.extra = Some(Uuid::new_v4().to_string());
        inner.td_type = "setTdlibParameters".to_string();
        SetTdlibParametersBuilder { inner }
    }

    pub fn parameters(&self) -> TdlibParameters {
        TdlibParameters::builder()
            .use_test_dc(self.use_test_dc)
            .database_directory(&self.database_directory)
            .files_directory(&self.files_directory)
            .use_file_database(self.use_file_database)
            .use_chat_info_database(self.use_chat_info_database)
            .use_message_database(self.use_message_database)
            .use_secret_chats(self.use_secret_chats)
            .api_id(self.api_id)
            .api_hash(&self.api_hash)
            .system_language_code(&self.system_language_code)
            .device_model(&self.device_model)
            .system_version(&self.system_version)
            .application_version(&self.application_version)
            .enable_storage_optimizer(self.enable_storage_optimizer)
            .ignore_file_names(self.ignore_file_names)
            .build()
    }
}

#[doc(hidden)]
pub struct SetTdlibParametersBuilder {
    inner: SetTdlibParameters,
}

#[deprecated]
pub type RTDSetTdlibParametersBuilder = SetTdlibParametersBuilder;

impl SetTdlibParametersBuilder {
    pub fn build(&self) -> SetTdlibParameters {
        self.inner.clone()
    }

    pub fn parameters<T: AsRef<TdlibParameters>>(&mut self, parameters: T) -> &mut Self {
        let p = parameters.as_ref();
        self.inner.use_test_dc = p.use_test_dc();
        self.inner.database_directory = p.database_directory().clone();
        self.inner.files_directory = p.files_directory().clone();
        self.inner.use_file_database = p.use_file_database();
        self.inner.use_chat_info_database = p.use_chat_info_database();
        self.inner.use_message_database = p.use_message_database();
        self.inner.use_secret_chats = p.use_secret_chats();
        self.inner.api_id = p.api_id();
        self.inner.api_hash = p.api_hash().clone();
        self.inner.system_language_code = p.system_language_code().clone();
        self.inner.device_model = p.device_model().clone();
        self.inner.system_version = p.system_version().clone();
        self.inner.application_version = p.application_version().clone();
        self.inner.enable_storage_optimizer = p.enable_storage_optimizer();
        self.inner.ignore_file_names = p.ignore_file_names();
        self
    }
}

impl AsRef<SetTdlibParameters> for SetTdlibParameters {
    fn as_ref(&self) -> &SetTdlibParameters {
        self
    }
}

impl AsRef<SetTdlibParameters> for SetTdlibParametersBuilder {
    fn as_ref(&self) -> &SetTdlibParameters {
        &self.inner
    }
}
