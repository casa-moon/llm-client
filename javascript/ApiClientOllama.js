const ApiClient = require('./ApiClient');
const axios = require('axios');

class ApiClientOllama extends ApiClient {
  constructor(apiKey, messageLog) {
    super(apiKey, messageLog, 'ollama');
    this.baseURL = process.env.OLLAMA_API_URL || 'http://127.0.0.1:11434';
    const headers = { 'Content-Type': 'application/json' };
    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }
    this.client = axios.create({ baseURL: this.baseURL });
  }

  async sendMessage(model) {
    const rawMessageLog = this.messageLog.getRawMessageLog();
    const formattedMessageLog = this.transformMessageLog(rawMessageLog);
    //console.log(JSON.stringify(formattedMessageLog, null, 2));

    const ora = (await import('ora')).default;
    const spinner = ora().start();

    const response = await this.client.post('/v1/chat/completions', {
      model,
      messages: formattedMessageLog
    });

    console.log('\n\nAPI Response:', response.data);
    spinner.stop();

    return response.data.choices[0].message.content;
  }

  // message transformation handled by base class template

  // Fetch available models from Ollama server
  async listModels() {
    const response = await this.client.get('/v1/models');
    // response.data.data contains the models array
    return response.data.data.map(m => m.id);
  }
}

module.exports = ApiClientOllama;