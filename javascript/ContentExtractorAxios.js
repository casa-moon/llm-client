// AxiosContentExtractor.js
const axios = require("axios");
const cheerio = require("cheerio");
const url = require("url");
const sharp = require("sharp");
const ContentExtractor = require('./ContentExtractor');

class AxiosContentExtractor extends ContentExtractor {

  constructor(chatSession) {
    super(chatSession);
  }
  
  async extractText(path, depth) {
    if (this.visitedUrls.has(path)) {
      //console.log('Skipping already visited URL:', path);
      return null;
    }
    this.visitedUrls.add(path);

    const response = await axios.get(path);
    const $ = cheerio.load(response.data);
    let textContent = '';
    $('p, h1, h2, h3, h4, h5, h6, span').each((i, elem) => {
      textContent += $(elem).text() + '\n';
    });

    this.chatSession.appendMessageToFile('\n- ' + path + '\n');
    this.addMessageToTempMessageLog(path);
    this.addMessageToTempMessageLog(textContent);

    // Recursively extract text from links if depth > 0
    if (depth > 0) {
      const links = [];
      const baseHostname = url.parse(path).hostname;
      $('a').each((i, link) => {
        const href = $(link).attr('href');
        if (href) {
          const absoluteUrl = url.resolve(path, href);
          
          // only scrape links from the same domain
          if (url.parse(absoluteUrl).hostname === baseHostname) {
            links.push(absoluteUrl);
          }
        }
      });

      // Process links concurrently
      const promises = links.map(async link => await this.extractText(link, depth - 1));
      await Promise.all(promises);
    }
  }

  async extractImages(path, depth) {
    if (this.visitedUrls.has(path)) {
      //console.log('Skipping already visited URL:', path);
      return null;
    }
    this.visitedUrls.add(path);

    const response = await axios.get(path);
    const $ = cheerio.load(response.data);
    let imageUrls = $('img').map((i, img) => $(img).attr('src')).get();

    // Resolve relative URLs and filter out duplicates and google maps
    imageUrls = imageUrls
      .map(src => url.resolve(path, src))
      .filter((src, index, self) => self.indexOf(src) === index
        && !src.includes('maps.googleapis.com'));
    
    for (const imageUrl of imageUrls) {
      if (this.visitedUrls.has(imageUrl)) continue;
      this.visitedUrls.add(imageUrl);

      try {
        const response = await axios.get(imageUrl, { responseType: 'arraybuffer' });
        const imageBuffer = Buffer.from(response.data);
        const metadata = await sharp(imageBuffer).metadata();

        if (metadata.width < 200 || metadata.height < 200) {
          continue; // Skip small images
        }

        this.chatSession.imageTokenCount += this.countImageTokens(metadata.width, metadata.height);
        this.chatSession.appendMessageToFile('\n- ' + imageUrl + '\n');

        if (metadata.format === 'svg') {
          const jpegBuffer = await sharp(imageBuffer).jpeg().toBuffer();
          const base64Image = `data:image/jpeg;base64,${jpegBuffer.toString('base64')}`;
          this.addMessageToTempMessageLog(imageUrl);
          this.addImageToTempMessageLog(base64Image);
        } else {
          this.addMessageToTempMessageLog(imageUrl);
          this.addImageToTempMessageLog(imageUrl);
        }
      } catch (error) {
        console.error(`Error processing image ${imageUrl}: ${error}`);
      }
    }
    
    if (depth > 0) {
      const links = [];
      const baseHostname = url.parse(path).hostname;
      $('a').each((i, link) => {
        const href = $(link).attr('href');
        if (href) {
          const absoluteUrl = url.resolve(path, href);
          
          // only scrape links from the same domain
          if (url.parse(absoluteUrl).hostname === baseHostname) {
            links.push(absoluteUrl);
          }
        }
      });

      // Process links concurrently
      const promises = links.map(async link => await this.extractImages(link, depth-1));
      await Promise.all(promises);
    }
  }
}

module.exports = AxiosContentExtractor;