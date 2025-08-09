// Load modules
const path = require('path');
require('dotenv').config({ path: path.join(__dirname, '.env') });
const inquirer = require('inquirer');
const ChatSession = require('./ChatSession');
const { getChoices: getApiChoices, createApiClient } = require('./ApiClientFactory');
const MessageLog = require('./MessageLog');
const DataProcessor = require('./ProcessorData');

class Main {
  constructor() {
    this.messageLog = new MessageLog();
  }

  async run() {
    // Select API client and model via factory
    const { api } = await inquirer.prompt({
      type: 'list',
      name: 'api',
      message: 'Select API',
      choices: getApiChoices()
    });
    const { apiClient, model: initialModel } = createApiClient(api, this.messageLog);
    this.apiClient = apiClient;
    let selectedModel = initialModel;
    if (initialModel === 'ollama') {
      const availableModels = await this.apiClient.listModels();
      const modelChoices = availableModels.map(m => ({ name: m, value: m }));
      const { chosenModel } = await inquirer.prompt({
        type: 'list',
        name: 'chosenModel',
        message: 'Select Ollama model',
        choices: modelChoices,
        pageSize: modelChoices.length
      });
      selectedModel = chosenModel;
    }
    this.model = selectedModel;
    this.chatSession = new ChatSession(this.apiClient, this.model, this.messageLog);

    // Command selection loop using Inquirer list
    const commandChoices = [
      { name: 'Direct Chat Input', value: 'chat' },
      { name: 'Multi-line Input', value: 'multi' },
      { name: 'File', value: 'file' },
      { name: 'PDF', value: 'pdf' },
      { name: 'Excel (xlsx)', value: 'xlsx' },
      { name: 'Image', value: 'image' },
      { name: 'Directory', value: 'dir' },
      { name: 'Git Repository', value: 'git' },
      { name: 'Web Page', value: 'web' },
      { name: 'Save Chat Session', value: 'save' },
      { name: 'Exit Application', value: 'exit' }
    ];
    while (true) {
      const { command } = await inquirer.prompt({
        type: 'list', 
        name: 'command', 
        message: 'Select command', 
        choices: commandChoices, 
        pageSize: commandChoices.length
      });
      switch (command) {
        case 'save':
          this.chatSession.cleanUp(true);
          return;
        case 'exit':
          this.chatSession.cleanUp();
          return;
        case 'chat': {
          const { input } = await inquirer.prompt({ type: 'input', name: 'input', message: 'Chat input:' });
          await new DataProcessor(this.chatSession).process(input);
          break;
        }
        case 'multi': {
          const { input } = await inquirer.prompt({ type: 'editor', name: 'input', message: 'Multi-line input:' });
          await new DataProcessor(this.chatSession).process(input);
          break;
        }
        default:
          await new DataProcessor(this.chatSession).process(command);
      }
    }
  }
}

// Load the main class and run the application
new Main().run().then();