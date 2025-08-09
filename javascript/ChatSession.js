// Load modules
const fs = require("fs");
const path = require('path');
const os = require('os');

class ChatSession {
  // Initialize the ChatSession
  constructor(apiClient, model, messageLog) {
    this.apiClient = apiClient;
    this.messageLog = messageLog;
    this.model = model;
    this.imageTokenCount = 0;

    // termux support
    if (process.env.TERMUX_VERSION) {
      this.isTermux = true;
      this.homeDir = '/storage/emulated/0/Download'
      this.chatgptDir = 'chatgpt-client'
    }
    else {
      this.isTermux = false;
      this.homeDir = os.homedir();
      this.chatgptDir = '.chatgpt-client'
    }
    
    // Set the directories to save files
    this.dir = path.join(this.homeDir, this.chatgptDir, 'files');
    this.tempDir = path.join(this.homeDir, this.chatgptDir, 'temp');
    this.chatFileDir = path.join(this.homeDir, this.chatgptDir, 'chats');

    // Create the chatFileDir
    if (!fs.existsSync(this.chatFileDir)) {
      fs.mkdirSync(this.chatFileDir, { recursive: true });
    }
    
    // Create the files directory
    if (!fs.existsSync(this.dir)) {
      fs.mkdirSync(this.dir, { recursive: true });
    }
    
    // Create the temp directory
    if (!fs.existsSync(this.tempDir)) {
      fs.mkdirSync(this.tempDir, { recursive: true });
    }

    // Create the chat file path
    this.chatFileName = 'message-log-' + new Date(Date.now()).toISOString().replace(/:/g,'-') + '.md';
    this.chatFilePath = path.join(this.chatFileDir, this.chatFileName);
  }

  // Send a message to the API
  async sendMessageToApi() {
    // Send the message to the API
    return await this.apiClient.sendMessage(this.model);
  }

  appendMessageToFile(message) {
    fs.appendFileSync(this.chatFilePath, message, 'utf8');
  }

  cleanUp(save = false) {
    if (fs.existsSync(this.chatFilePath) && !save) {
      fs.unlinkSync(this.chatFilePath);
      console.log('\nChat transcript deleted.\n');
    }
    else if (fs.existsSync(this.chatFilePath) && save) {
      console.log('\nChat transcript saved to ' + this.chatFilePath + '\n');
    }

    // delete the temp dir
    if (fs.existsSync(this.tempDir)) {
      fs.rmdirSync(this.tempDir, { recursive: true});
    }
  }
}

module.exports = ChatSession;