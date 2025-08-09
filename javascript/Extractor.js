const fs = require("fs");
const isUtf8 = require('is-utf8');
const pdfjsLib = require('pdfjs-dist/legacy/build/pdf.mjs');
const { PDFDocument, PDFName, PDFRawStream } = require('pdf-lib');
const pathModule = require('path');
const pako = require('pako');
const sharp = require('sharp');

class Extractor {
  constructor(chatSession) {
    this.chatSession = chatSession;
    this.tempMessageLog = [];
    this.visitedUrls = new Set();
    this.chatSession.imageTokenCount = 0;
  }

  extract() {
    throw new Error('extract() must be implemented by subclasses');
  }

  addMessageToTempMessageLog(data) {
    this.tempMessageLog.push({
      role: "user",
      type: "text",
      content: data
    });
  }

  addImageToTempMessageLog(data) {
    this.tempMessageLog.push({
      role: "user",
      type: "image",
      content: data
    });
  }

  countImageTokens(width, height) {
    const h = Math.ceil(height / 512);
    const w = Math.ceil(width / 512);
    const n = w * h;
    return 85 + 170 * n;
  }

  isTextFile(path) {
    try {
      const data = fs.readFileSync(path);
      return isUtf8(data);
    } catch (error) {
      console.log('Could not determine the file type');
      return false;
    }
  }

  async extractTextFromPdf(path) {
    try {
      const data = new Uint8Array(fs.readFileSync(path));
      const pdf = await pdfjsLib.getDocument({ data }).promise;
      let text = '';
      const numPages = pdf.numPages;
      for (let i = 1; i <= numPages; i++) {
        const page = await pdf.getPage(i);
        const content = await page.getTextContent({ normalizeWhitespace: true });
        const strings = content.items.map(item => item.str);
        text += strings.join(' ') + '\n';
      }
      return text;
    } catch (error) {
      throw new Error(`\nFailed to extract text from PDF: ${error.message}\n`);
    }
  }

  async extractImagesFromPdf(path) {
    try {
      const pdfBytes = fs.readFileSync(path);
      const pdfDoc = await PDFDocument.load(pdfBytes);

      const enumeratedIndirectObjects = pdfDoc.context.enumerateIndirectObjects();
      const imagesInDoc = [];
      let objectIdx = 0;
      let imageIdx = 1;

      for (const [pdfRef, pdfObject] of enumeratedIndirectObjects) {
        try {
          //console.log(pdfRef);
          objectIdx += 1;

          if (!(pdfObject instanceof PDFRawStream)) continue;

          const {dict} = pdfObject;

          const subtype = dict.get(PDFName.of("Subtype"));
          if (subtype !== PDFName.of("Image")) continue;

          const filter = dict.get(PDFName.of("Filter"));
          //console.log('\n' + filter);
          let imageType = null;
          switch (filter) {
            case PDFName.of("DCTDecode"):
              imageType = "jpg";
              break;
            case PDFName.of("FlateDecode"):
              imageType = "png";
              break;
            //case PDFName.of("JPXDecode"):
            //imageType = "jpeg2000"; // JPX is typically used for JPEG2000 in PDFs
            //break;
            // ... Add more filters for other image formats like WebP, GIF, AVIF, TIFF, SVG etc.
            default:
              console.log(
                `Unsupported image format detected for ref: ${pdfRef}. Filter used: ${filter}`
              );
              continue;
          }
          //console.log(imageType);

          // Extract other image information
          //const smaskRef = dict.get(PDFName.of("SMask"));
          //const name = dict.get(PDFName.of("Name"));
          //const bitsPerComponent = dict.get(PDFName.of("BitsPerComponent")).numberValue;

          // Extract channels, width and height
          let channels = null;
          const colorSpace = dict.get(PDFName.of("ColorSpace")).toString();
          //console.log(colorSpace);
          if (colorSpace === '/DeviceRGB') channels = 3;
          else if (colorSpace === '/DeviceCMYK') channels = 4;
          else if (colorSpace === '/DeviceGray') continue;
          else if (colorSpace.includes('/ICCBased')) channels = 3;
          const width = dict.get(PDFName.of("Width")).numberValue;
          const height = dict.get(PDFName.of("Height")).numberValue;
          //console.log(channels);
          //console.log(width);
          //console.log(height);


          let buffer = Buffer.from(pdfObject.contents);
          if (imageType === 'png') {
            const inflated = pako.inflate(buffer);
            buffer = await sharp(inflated, {
              raw: {
                width: width,
                height: height,
                channels: channels
              }
            }).toFormat('jpeg').toBuffer();
            fs.writeFileSync(pathModule.join(this.chatSession.tempDir, 'image' + imageIdx + '.jpeg'), buffer);
            imageIdx += 1;
          } else {
            fs.writeFileSync(pathModule.join(this.chatSession.tempDir, 'image' + imageIdx + '.' + imageType), buffer);
            imageIdx += 1;
          }

          const tokens = this.countImageTokens(width, height);
          //console.log(tokens);

          const base64Image = "data:image/jpeg;base64," + buffer.toString("base64");
          imagesInDoc.push([base64Image, tokens]);
          //console.log(base64Image);
        } catch (error) {
          console.log(`Error extracting image from ref in for loop: ${pdfRef}. Error: ${error.message}`);
        }
      }
      return imagesInDoc;
    }
    catch (error) {
      throw new Error(`\nFailed to extract images from PDF: ${error.message}\n`);
    }
  }
}

module.exports = Extractor;