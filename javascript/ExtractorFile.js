// Load modules
const Extractor = require('./Extractor');
const fs = require('fs');

class FileExtractor extends Extractor {
  
  // Initialize the FileExtractor
  constructor(chatSession) {
    super(chatSession);
  }
  extract(path) {
    try {
      // Check if the file is a text file
      if (!this.isTextFile(path)) {
        console.log(`${path} is not a text file.`);
        return null;
      }
      
      // Extract the text from the file
      const data= fs.readFileSync(path, 'utf8');
      this.addMessageToTempMessageLog(data);
      return this.tempMessageLog;
    } catch (error) {
      throw new Error(`\nFailed to extract text from file: ${error.message}\n`);
    }
  }
}

module.exports = FileExtractor;