const ApiClient = require('./ApiClient');
const Anthropic = require('@anthropic-ai/sdk');

class ApiClientAnthropic extends ApiClient {
  constructor(apiKey, messageLog) {
    super(apiKey, messageLog, 'anthropic');
    this.anthropic = new Anthropic({ apiKey: this.apiKey });
  }

  async sendMessage(model) {
    const rawMessageLog = this.messageLog.getRawMessageLog();
    const formattedMessageLog = this.transformMessageLog(rawMessageLog);
    
    // Display the formatted message log
    //console.log(JSON.stringify(formattedMessageLog, null, 2));

    // Import the ora module
    const ora = (await import('ora')).default;

    // Create a new ora instance
    const spinner = ora().start();

    const response = await this.anthropic.beta.messages.create({
      model: model,
      max_tokens: 4096,
      temperature: 0,
      messages: formattedMessageLog,
    });

    // Log the entire response
    let responseCopy = { ...response };
    delete responseCopy.content;
    console.log('\n\nAPI Response:', responseCopy);

    // Stop the spinner
    spinner.stop();

    // Return the response
    return response.content[0].text;
  }

  // message transformation handled by base class template
}

module.exports = ApiClientAnthropic;