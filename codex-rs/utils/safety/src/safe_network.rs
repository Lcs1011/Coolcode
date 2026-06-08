use crate::safe_mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPurpose {
    ChatGPTAuth,
    ModelApi,
    Other,
}

impl NetworkPurpose {
    pub fn allowed_in_safe_mode(self) -> bool {
        matches!(self, NetworkPurpose::ChatGPTAuth | NetworkPurpose::ModelApi)
    }
}

pub fn ensure_allowed(purpose: NetworkPurpose) -> anyhow::Result<()> {
    if safe_mode::enabled() && !purpose.allowed_in_safe_mode() {
        anyhow::bail!("network request blocked by SafeMode: {purpose:?}");
    }

    Ok(())
}

pub async fn send(
    purpose: NetworkPurpose,
    request: reqwest::RequestBuilder,
) -> anyhow::Result<reqwest::Response> {
    ensure_allowed(purpose)?;
    Ok(request.send().await?)
}

pub fn blocking_send(
    purpose: NetworkPurpose,
    request: reqwest::blocking::RequestBuilder,
) -> anyhow::Result<reqwest::blocking::Response> {
    ensure_allowed(purpose)?;
    Ok(request.send()?)
}
