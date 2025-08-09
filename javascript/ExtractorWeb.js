// WebExtractor.js
const Extractor = require('./Extractor');
const AxiosContentExtractor = require('./ContentExtractorAxios');
const PuppeteerContentExtractor = require('./ContentExtractorPuppeteer');

class WebExtractor extends Extractor {
  constructor(chatSession) {
    super(chatSession);
  }

  async extract(path, depth = 0, getImages = false, useAxios = false) {
    try {
      const ora = (await import('ora')).default;
      const spinner = ora().start();
      
      const contentExtractor = useAxios ? new AxiosContentExtractor(this.chatSession) : new PuppeteerContentExtractor(this.chatSession);
      await contentExtractor.extractText(path, depth);

      if (getImages) {
        contentExtractor.visitedUrls.clear();
        await contentExtractor.extractImages(path, depth);
      }

      spinner.stop();
      return contentExtractor.tempMessageLog;
    } catch (error) {
      throw new Error(`Failed to extract from web: ${error.message}`);
    }
  }
}

module.exports = WebExtractor;