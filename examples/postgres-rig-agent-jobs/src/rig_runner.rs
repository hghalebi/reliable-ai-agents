use async_trait::async_trait;

use crate::agent::{AgentError, AgentRunner, ProviderFailure};
use crate::agent_output::RawAgentOutput;
use crate::domain::{AgentPayload, AgentResult};

#[derive(Debug, Clone, Default)]
pub struct DeepSeekRigAgentRunner;

#[async_trait]
impl AgentRunner for DeepSeekRigAgentRunner {
    async fn run_agent(&self, payload: AgentPayload) -> Result<AgentResult, AgentError> {
        use rig::client::{CompletionClient, ProviderClient};
        use rig::completion::Prompt;
        use rig::providers::deepseek;

        let client =
            deepseek::Client::from_env().map_err(|error| {
                match ProviderFailure::permanent(format!(
                    "deepseek client configuration failed: {error}"
                )) {
                    Ok(error) => AgentError::from(error),
                    Err(error) => AgentError::InvalidOutput(error),
                }
            })?;

        let agent = client
            .agent(deepseek::DEEPSEEK_V4_FLASH)
            .preamble(
                "You are an operations agent. Return only a strict JSON object \
                 with keys summary, next_action, and approval_requirement. \
                 approval_requirement must be required or not_required. \
                 Do not wrap the JSON in markdown. Do not execute risky \
                 actions directly.",
            )
            .max_tokens(800)
            .build();

        let provider_output =
            agent
                .prompt(payload.instruction.as_str())
                .await
                .map_err(|error| {
                    match ProviderFailure::transient(format!(
                        "deepseek completion request failed: {error}"
                    )) {
                        Ok(error) => AgentError::from(error),
                        Err(error) => AgentError::InvalidOutput(error),
                    }
                })?;

        Ok(RawAgentOutput::new(provider_output)?.parse_agent_result()?)
    }
}
