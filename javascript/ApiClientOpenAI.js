const ApiClient = require('./ApiClient');
const OpenAI = require('openai');

class ApiClientOpenAI extends ApiClient {
  constructor(apiKey, messageLog) {
    super(apiKey, messageLog, 'openai');
    this.openai = new OpenAI({ apiKey: this.apiKey });
  }

  async sendMessage(model) {
    const rawMessageLog = this.messageLog.getRawMessageLog();
    const formattedMessageLog = this.transformMessageLog(rawMessageLog);
    //console.log(JSON.stringify(formattedMessageLog, null, 2));
    
    // Import the ora module
    const ora = (await import('ora')).default;

    // Create a new ora instance
    const spinner = ora().start();
    
    let response;
    if (model === 'o4-mini') {
      response = await this.openai.chat.completions.create({
        model: model,
        max_tokens: 32768,
        temperature: 1,
        messages: formattedMessageLog,
      });
    }
    else if (model === 'gpt-4.1') {
      response = await this.openai.chat.completions.create({
        model: model,
        max_tokens: 32768,
        temperature: 0,
        messages: formattedMessageLog,
      });
    }
    
    // Log the entire response
    console.log('\n\nAPI Response:', response);

    // Stop the spinner
    spinner.stop();
    
    // Return the response
    return response.choices[0].message.content;
  }

  // message transformation handled by base class template
}

module.exports = ApiClientOpenAI;