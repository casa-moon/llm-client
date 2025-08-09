// Load modules
const Extractor = require('./Extractor');

class ImageExtractor extends Extractor {
  
  constructor(chatSession) {
    super(chatSession);
  }
  
  extract(path) {
    try {
      if (path.startsWith('http')) {
        this.addImageToTempMessageLog(path);
      }
      else {
        console.log(`\n${path} is not a valid image path. Only http(s) paths are supported at this time.\n`);
      }
      return this.tempMessageLog;
    } catch (error) {
      throw new Error(`\nFailed to extract image: ${error.message}\n`);
    }
  }
}

module.exports = ImageExtractor;