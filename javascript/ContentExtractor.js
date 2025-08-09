const Extractor = require('./Extractor');

class ContentExtractor extends Extractor {

  constructor(chatSession) {
    super(chatSession);
  }
  
  async extractText(path, depth) {
    throw new Error('extractText() must be implemented by subclasses');
  }

  async extractImages(path, depth) {
    throw new Error('extractImages() must be implemented by subclasses');
  }
}

module.exports = ContentExtractor;