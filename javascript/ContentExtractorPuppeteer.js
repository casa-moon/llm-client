// PuppeteerContentExtractor.js
const puppeteer = require("puppeteer-extra");
const StealthPlugin = require('puppeteer-extra-plugin-stealth');
const url = require("url");
const sharp = require("sharp");
const ContentExtractor = require('./ContentExtractor');

// Initialize variables
puppeteer.use(StealthPlugin());

class PuppeteerContentExtractor extends ContentExtractor {

  constructor(chatSession) {
    super(chatSession);
  }
  
  async extractText(path, depth) {
    if (this.visitedUrls.has(path)) {
      //console.log('Skipping already visited URL:', path);
      return null;
    }
    this.visitedUrls.add(path);

    const browser = await puppeteer.launch({ headless: 'new' });
    try {
      const page = await browser.newPage();
      page.setDefaultNavigationTimeout(60000); // 60 seconds
      await page.goto(path, { waitUntil: 'load' });

      let textContent = await page.evaluate(() => {
        return Array.from(document.querySelectorAll('p, h1, h2, h3, h4, h5, h6, span')).map(element => element.innerText).join('\n');
      });

      this.chatSession.appendMessageToFile('\n- ' + path + '\n');
      this.addMessageToTempMessageLog(path);
      this.addMessageToTempMessageLog(textContent);

      if (depth > 0) {
        const links = await page.evaluate(() =>
          Array.from(document.querySelectorAll('a')).map(a => a.href)
        );

        // only scrape links from the same domain
        const baseHostname = url.parse(path).hostname;
        const filteredLinks = links.filter(link => {
          const absoluteUrl = url.resolve(path, link);
          return url.parse(absoluteUrl).hostname === baseHostname;
        });

        // Process links concurrently
        const promises = filteredLinks.map(async link => await this.extractText(link, depth - 1));
        await Promise.all(promises);
      }
      }
      catch (error) {
        console.error(`Error processing URL ${path}: ${error}`);
      }
      finally {
        await browser.close();
      }
  }

  async extractImages(path, depth) {
    if (this.visitedUrls.has(path)) {
      //console.log('Skipping already visited URL:', path);
      return null;
    }
    this.visitedUrls.add(path);

    const browser = await puppeteer.launch({headless: 'new'});
    try {
      const page = await browser.newPage();
      page.setDefaultNavigationTimeout(60000); // 60 seconds
      await page.goto(path, { waitUntil: 'load' });

      let imageUrls = await page.evaluate(() =>
        Array.from(document.querySelectorAll('img')).map(img => img.src)
      );

      // Resolve relative URLs and filter out duplicates and google maps
      imageUrls = imageUrls
        .map(src => url.resolve(path, src))
        .filter((src, index, self) => self.indexOf(src) === index
          && !src.includes('maps.googleapis.com'));

      for (const imageUrl of imageUrls) {
        if (this.visitedUrls.has(imageUrl)) continue;
        this.visitedUrls.add(imageUrl);

        try {
          const imageBuffer = await page.evaluate(async (src) => {
            const response = await fetch(src);
            const buffer = await response.arrayBuffer();
            return Array.from(new Uint8Array(buffer));
          }, imageUrl);

          const metadata = await sharp(Uint8Array.from(imageBuffer)).metadata();

          if (metadata.width < 200 || metadata.height < 200) {
            continue; // Skip small images
          }

          this.chatSession.imageTokenCount += this.countImageTokens(metadata.width, metadata.height);
          this.chatSession.appendMessageToFile('\n- ' + imageUrl + '\n');

          if (metadata.format === 'svg') {
            const jpegBuffer = await sharp(Uint8Array.from(imageBuffer)).jpeg().toBuffer();
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
        const links = await page.evaluate(() =>
          Array.from(document.querySelectorAll('a')).map(a => a.href)
        );

        // only scrape links from the same domain
        const baseHostname = url.parse(path).hostname;
        const filteredLinks = links.filter(link => {
          const absoluteUrl = url.resolve(path, link);
          return url.parse(absoluteUrl).hostname === baseHostname;
        });

        // Process links concurrently
        const promises = filteredLinks.map(async link => await this.extractImages(link, depth - 1));
        await Promise.all(promises);
      }

    } catch (error) {
      console.error(`Error processing URL ${path}: ${error}`);
    } finally {
      await browser.close();
    }
  }
}

module.exports = PuppeteerContentExtractor;