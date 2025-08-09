// Factory to create extractor instances based on command type
const FileExtractor = require('./ExtractorFile');
const DirExtractor = require('./ExtractorDir');
const PdfExtractor = require('./ExtractorPdf');
const ImageExtractor = require('./ExtractorImage');
const WebExtractor = require('./ExtractorWeb');

// Map user commands to extractor classes
const extractorMapping = {
  file: FileExtractor,
  f: FileExtractor,
  pdf: PdfExtractor,
  p: PdfExtractor,
  xlsx: DirExtractor,
  x: DirExtractor,
  image: ImageExtractor,
  i: ImageExtractor,
  dir: DirExtractor,
  d: DirExtractor,
  git: DirExtractor,
  g: DirExtractor,
  web: WebExtractor,
  w: WebExtractor
};

/**
 * Create an extractor instance for the given command
 * @param {string} command - The user command (e.g., 'file', 'pdf')
 * @param {object} chatSession - The active chat session
 * @returns {object} Instance of the appropriate extractor
 */
function createExtractor(command, chatSession) {
  const key = command.toLowerCase();
  const ExtractorClass = extractorMapping[key];
  if (!ExtractorClass) throw new Error(`Unsupported extractor command: ${command}`);
  return new ExtractorClass(chatSession);
}

module.exports = { createExtractor };