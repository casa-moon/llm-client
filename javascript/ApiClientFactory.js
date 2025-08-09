// Factory to create API client instances based on user selection
const ApiClientOpenAI = require('./ApiClientOpenAI');
const ApiClientGoogle = require('./ApiClientGoogle');
const ApiClientAnthropic = require('./ApiClientAnthropic');
const ApiClientPerplexity = require('./ApiClientPerplexity');
const ApiClientMistral = require('./ApiClientMistral');
const ApiClientOllama = require('./ApiClientOllama');

// Configuration for available API clients and models
const apiConfigs = [
  { name: '1', message: 'gpt-4.1', client: ApiClientOpenAI, apiKeyEnv: 'OPENAI_API_KEY', model: 'gpt-4.1' },
  { name: '2', message: 'o4-mini', client: ApiClientOpenAI, apiKeyEnv: 'OPENAI_API_KEY', model: 'o4-mini' },
  { name: '3', message: 'gemini-1.5-pro-latest', client: ApiClientGoogle, apiKeyEnv: 'GOOGLE_AI_API_KEY', model: 'gemini-1.5-pro-latest' },
  { name: '4', message: 'claude-3-5-sonnet-20240620', client: ApiClientAnthropic, apiKeyEnv: 'ANTHROPIC_API_KEY', model: 'claude-3-5-sonnet-20240620' },
  { name: '5', message: 'llama-3.1-sonar-large-128k-chat', client: ApiClientPerplexity, apiKeyEnv: 'PERPLEXITY_API_KEY', model: 'llama-3.1-sonar-large-128k-chat' },
  { name: '6', message: 'mistral-medium', client: ApiClientMistral, apiKeyEnv: 'MISTRAL_API_KEY', model: 'mistral-medium' },
  { name: '7', message: 'ollama', client: ApiClientOllama, apiKeyEnv: 'OLLAMA_API_KEY', model: 'ollama' }
];

/**
 * Get choices for inquirer prompt
 * @returns {Array<{name: string, value: string}>}
*/
function getChoices() {
  return apiConfigs.map(cfg => ({ name: cfg.message, value: cfg.name }));
}

/**
 * Create an API client and model based on the selected choice
 * @param {string} choiceName - The selected name/key
 * @param {MessageLog} messageLog - The message log instance
 * @returns {{ apiClient: object, model: string }}
 */
function createApiClient(choiceName, messageLog) {
  const cfg = apiConfigs.find(c => c.name === choiceName);
  if (!cfg) throw new Error(`Invalid API choice: ${choiceName}`);
  const apiKey = process.env[cfg.apiKeyEnv];
  if (!apiKey) throw new Error(`Missing API key for ${cfg.apiKeyEnv}`);
  const apiClient = new cfg.client(apiKey, messageLog);
  return { apiClient, model: cfg.model };
}

module.exports = { getChoices, createApiClient };