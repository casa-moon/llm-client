// Load modules
const Extractor = require('./Extractor');

class PdfExtractor extends Extractor {

  constructor(chatSession) {
    super(chatSession);
  }
  async extract(path, getImages = false) { 
    try {
      const pdfText = await this.extractTextFromPdf(path);

      // Add the text from the PDF to the message log
      this.addMessageToTempMessageLog(pdfText);
      
      if (getImages) {
        // Returns an array of images in Base64 format and the number of tokens for each image
        const pdfImages = await this.extractImagesFromPdf(path);

        // Append images to the message log and count tokens
        let imageIdx = 0;
        this.chatSession.imageTokenCount = 0;
        for (const pdfImage of pdfImages) {
          imageIdx += 1;
          this.addMessageToTempMessageLog(`image${imageIdx}`);
          this.addImageToTempMessageLog(pdfImage[0]);
          this.chatSession.imageTokenCount += pdfImage[1];
        }
      }
      return this.tempMessageLog;
    } catch (error) {
      throw new Error(`\nFailed to extract from pdf: ${error.message}\n`);
    }
  }
}

module.exports = PdfExtractor;