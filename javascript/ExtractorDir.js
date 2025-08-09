// Load modules
const Extractor = require('./Extractor');
const FileExtractor = require('./ExtractorFile');
const fs = require('fs');

class DirExtractor extends Extractor {
  
  constructor(chatSession) {
    super(chatSession);
  }
  
  extract(path, recursive = true) {
    try {
      const files = fs.readdirSync(path);
      for (const file of files) {
        if (
          (file !== 'node_modules') &&
          (file !== 'temp') &&
          (file !== 'archive') &&
          (file !== 'ideas') &&
          (file !== 'charts') &&
          (file !== 'chats') &&
          (file !== 'package-lock.json') &&
          (file !== '.idea') &&
          (file !== '.gitignore') &&
          (file !== '.git') &&
          (file !== '.terraform') &&
          (file !== '.editorconfig') &&
          (file !== '.pre-commit-config.yaml') &&
          (file !== '.releaserc.json') &&
          (file !== 'CHANGELOG.md') &&
          (file !== 'LICENSE')
        ) {
          const filePath = path + '/' + file;

          const stats = fs.statSync(filePath);
          if (stats.isDirectory() && recursive) {
            this.extract(filePath);
          } 
          else if (stats.isFile()) {
            const messageLog = new FileExtractor(this.chatSession).extract(filePath);
            
            // Push the file to the message log
            if (messageLog !== null) {
              this.addMessageToTempMessageLog(filePath);
              this.tempMessageLog.push(...messageLog);
              
              // Append message to chat file
              this.chatSession.appendMessageToFile('\n- ' + filePath + '\n');
            }
          }
        }
      }
      return this.tempMessageLog;
    } catch (error) {
      throw new Error(`\nFailed to extract text from directory: ${error.message}\n`);
    }
  }
}

module.exports = DirExtractor;